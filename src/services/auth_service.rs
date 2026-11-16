//! Authentication and session token issuing.
//!
//! Registry clients exchange basic credentials for a JWT session token. This
//! module owns the credential check (argon2) and the token lifecycle (sign and
//! verify); the routes layer only extracts headers and renders responses.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::UserConfig;
use crate::services::Error;

/// Lifetime of a session token, in seconds.
const TOKEN_DURATION: u64 = 3600;
const TOKEN_AUDIENCE: &str = "Trow Registry";

/// Identity of an authenticated client. Produced by `AuthService`; the routes
/// layer carries it through its extractors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub user: String,
    pub token: String,
}

/// Just using the default token claim stuff. Could add scope stuff (which
/// repos, what rights), but could also keep this in DB. Mirroring the Docker
/// format would allow reuse of existing token server implementations.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TokenClaim {
    /// (Issuer) The issuer of the token, typically the fqdn of the authorization server.
    iss: String,
    /// (Subject) The name or id of the client which requested the token.
    sub: String,
    /// (Audience) The service which will verify the token.
    aud: String,
    /// (Expiration) Up to when the token is valid.
    exp: u64,
    /// (Not Before) From when the token is valid.
    nbf: u64,
    /// (Issued At) When the token was issued.
    iat: u64,
    /// (JWT ID) Unique identifier for this token.
    jti: String,
}

#[derive(Debug)]
pub struct AuthService {
    /// Issuer name written into the `iss` claim.
    service_name: String,
    /// HMAC secret used to sign and verify session tokens.
    token_secret: Vec<u8>,
    /// The single configured user, if authentication is enabled.
    user: Option<UserConfig>,
}

impl AuthService {
    pub fn new(user: Option<UserConfig>, service_name: String, token_secret: Vec<u8>) -> Self {
        Self {
            service_name,
            token_secret,
            user,
        }
    }

    /// Whether the registry requires clients to authenticate.
    pub fn requires_auth(&self) -> bool {
        self.user.is_some()
    }

    /// Verifies a decoded `user:password` pair against the configured user and
    /// returns the user name.
    pub fn verify_credentials(&self, user_pass: &[u8]) -> Result<String, Error> {
        let user_cfg = self
            .user
            .as_ref()
            .ok_or_else(|| Error::Unauthorized("no user is configured".to_string()))?;
        let sep = user_pass
            .iter()
            .position(|b| *b == b':')
            .ok_or_else(|| Error::Unauthorized("malformed basic credentials".to_string()))?;
        let (user, pass) = (&user_pass[..sep], &user_pass[sep + 1..]);
        if user_cfg.user.as_bytes() != user {
            return Err(Error::Unauthorized("unknown user".to_string()));
        }
        match argon2::verify_encoded(&user_cfg.hash_encoded, pass) {
            Ok(true) => Ok(user_cfg.user.clone()),
            Ok(false) => Err(Error::Unauthorized("wrong password".to_string())),
            Err(e) => Err(Error::Internal(format!("could not verify password: {e}"))),
        }
    }

    /// Issues a session token for an authenticated user.
    pub fn issue_token(&self, user: &str) -> Result<String, Error> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Internal("system clock is before the unix epoch".to_string()))?;

        let claim = TokenClaim {
            iss: self.service_name.clone(),
            sub: user.to_string(),
            aud: TOKEN_AUDIENCE.to_owned(),
            exp: current_time
                .checked_add(Duration::new(TOKEN_DURATION, 0))
                .ok_or_else(|| Error::Internal("token expiry overflow".to_string()))?
                .as_secs(),
            nbf: current_time.as_secs(),
            iat: current_time.as_secs(),
            jti: Uuid::new_v4().to_string(),
        };

        encode(
            &Header::default(),
            &claim,
            &EncodingKey::from_secret(&self.token_secret),
        )
        .map_err(|e| Error::Internal(format!("could not sign token: {e}")))
    }

    /// Verifies a bearer token and returns the identity it describes.
    pub fn verify_token(&self, token: &str) -> Result<AuthenticatedUser, Error> {
        let mut validation = Validation::default();
        validation.set_audience(&[TOKEN_AUDIENCE]);
        let decoded = decode::<TokenClaim>(
            token,
            &DecodingKey::from_secret(&self.token_secret),
            &validation,
        )
        .map_err(|e| Error::Unauthorized(format!("could not decode token: {e}")))?;

        Ok(AuthenticatedUser {
            user: decoded.claims.sub,
            token: token.to_string(),
        })
    }

    /// Identity used when no authentication is configured at all.
    pub fn anonymous() -> AuthenticatedUser {
        AuthenticatedUser {
            user: "none".to_string(),
            token: "none".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service(user: Option<(&str, &str)>) -> AuthService {
        let user = user.map(|(name, pass)| {
            let mut hash_config = argon2::Config::rfc9106();
            hash_config.mem_cost = 4066;
            hash_config.time_cost = 3;
            let hash_encoded =
                argon2::hash_encoded(pass.as_bytes(), Uuid::new_v4().as_bytes(), &hash_config)
                    .unwrap();
            UserConfig {
                user: name.to_string(),
                hash_encoded,
            }
        });
        AuthService::new(user, "http://trow.test".to_string(), vec![7u8; 32])
    }

    #[test]
    fn requires_auth_reflects_configuration() {
        assert!(!service(None).requires_auth());
        assert!(service(Some(("admin", "hunter2"))).requires_auth());
    }

    #[test]
    fn valid_credentials_are_accepted() {
        let auth = service(Some(("admin", "hunter2")));
        let user = auth.verify_credentials(b"admin:hunter2").unwrap();
        assert_eq!(user, "admin");
    }

    #[test]
    fn wrong_password_is_rejected() {
        let auth = service(Some(("admin", "hunter2")));
        assert!(matches!(
            auth.verify_credentials(b"admin:wrong"),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn wrong_user_is_rejected() {
        let auth = service(Some(("admin", "hunter2")));
        assert!(matches!(
            auth.verify_credentials(b"root:hunter2"),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn credentials_without_colon_are_rejected() {
        let auth = service(Some(("admin", "hunter2")));
        assert!(matches!(
            auth.verify_credentials(b"adminhunter2"),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn credentials_are_rejected_when_no_user_is_configured() {
        assert!(matches!(
            service(None).verify_credentials(b"admin:hunter2"),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn issued_token_round_trips() {
        let auth = service(Some(("admin", "hunter2")));
        let token = auth.issue_token("admin").unwrap();
        let identity = auth.verify_token(&token).unwrap();
        assert_eq!(identity.user, "admin");
        assert_eq!(identity.token, token);
    }

    #[test]
    fn token_from_another_secret_is_rejected() {
        let auth = service(Some(("admin", "hunter2")));
        let other = AuthService::new(None, "http://trow.test".to_string(), vec![9u8; 32]);
        let token = auth.issue_token("admin").unwrap();
        assert!(matches!(
            other.verify_token(&token),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn garbage_token_is_rejected() {
        let auth = service(Some(("admin", "hunter2")));
        assert!(matches!(
            auth.verify_token("not-a-jwt"),
            Err(Error::Unauthorized(_))
        ));
    }

    #[test]
    fn password_containing_colon_is_accepted() {
        let auth = service(Some(("admin", "hun:ter2")));
        assert_eq!(auth.verify_credentials(b"admin:hun:ter2").unwrap(), "admin");
    }
}
