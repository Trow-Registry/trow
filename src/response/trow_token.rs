use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{FromRef, FromRequestParts};
use axum::headers::HeaderMapExt;
use axum::http::request::Parts;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{body, headers};
use base64::engine::general_purpose as base64_engine;
use base64::Engine as _;
use frank_jwt::{decode, encode, Algorithm, ValidationOptions};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{event, Level};
use uuid::Uuid;

use super::authenticate::Authenticate;
use super::get_base_url;
use crate::{TrowConfig, UserConfig};

const TOKEN_DURATION: u64 = 3600;
const AUTHORIZATION: &str = "authorization";

pub struct ValidBasicToken {
    user: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for ValidBasicToken
where
    TrowConfig: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, ());

    async fn from_request_parts(req: &mut Parts, config: &S) -> Result<Self, Self::Rejection> {
        let config = TrowConfig::from_ref(config);

        let user_cfg = match config.user {
            Some(ref user_cfg) => user_cfg,
            None => {
                event!(Level::WARN, "Attempted login, but no users are configured");
                return Err((StatusCode::UNAUTHORIZED, ()));
            }
        };

        // As Authorization is a standard header
        let auth_val = match req.headers.get(AUTHORIZATION) {
            Some(a) => a.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, ()))?,
            None => return Err((StatusCode::UNAUTHORIZED, ())),
        };

        // The value of the header is the type of the auth (Basic or Bearer), followed by an
        // encoded string, separate by whitespace.

        let auth_strings: Vec<String> = auth_val.split_whitespace().map(String::from).collect();
        if auth_strings.len() != 2 {
            //TODO: Should this be BadRequest?
            return Err((StatusCode::UNAUTHORIZED, ()));
        }
        // We're looking for a Basic token
        if auth_strings[0] != "Basic" {
            //TODO: This probably isn't right, maybe check if bearer?
            return Err((StatusCode::UNAUTHORIZED, ()));
        }

        match base64_engine::STANDARD.decode(&auth_strings[1]) {
            Ok(user_pass) => {
                if verify_user(user_pass, user_cfg) {
                    Ok(ValidBasicToken {
                        user: user_cfg.user.clone(),
                    })
                } else {
                    Err((StatusCode::UNAUTHORIZED, ()))
                }
            }
            Err(_) => Err((StatusCode::UNAUTHORIZED, ())),
        }
    }
}

/**
 * Sod the errors, just fail verification if there's an encoding problem.
 */
fn verify_user(user_pass: Vec<u8>, user_cfg: &UserConfig) -> bool {
    let mut user_pass = user_pass.split(|b| b == &b':');
    if let Some(user) = user_pass.next() {
        if let Some(pass) = user_pass.next() {
            if user_cfg.user.as_bytes() == user {
                if let Ok(v) = argon2::verify_encoded(&user_cfg.hash_encoded, pass) {
                    return v;
                }
            }
        }
    }
    false
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrowToken {
    pub user: String,
    pub token: String,
}

// Just using the default token claim stuff
// Could add scope stuff (which repos, what rights), but could also keep this in DB
// Mirroring Docker format would allow reuse of existing token server implementations
#[derive(Clone, Debug, Serialize, Deserialize)]
struct TokenClaim {
    // (Issuer) The issuer of the token, typically the fqdn of the authorization server.
    iss: String,

    // (Subject)The subject of the token; the name or id of the client which requested it.
    // This should be empty if the client did not authenticate.
    sub: String,

    // (Audience) The intended audience of the token;
    // The name or id of the service which will verify the token to authorize the client/subject.
    aud: String,

    // (Expiration) The token should only be considered valid up to this specified date and time.
    exp: u64,

    // (Not Before) The token should not be considered valid before this specified date and time.
    nbf: u64,

    // (Issued At) Specifies the date and time which the Authorization server generated this token.
    iat: u64,

    // (JWT ID) A unique identifier for this token.
    // Can be used by the intended audience to prevent replays of the token.
    jti: String,
}
/*
 * Create new jsonwebtoken.
 * Token consists of a string with 3 comma separated fields header, payload, signature
 */
pub fn new(vbt: ValidBasicToken, config: &TrowConfig) -> Result<TrowToken, frank_jwt::Error> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // build token from structure and return token string
    let token_claim = TokenClaim {
        iss: config.service_name.clone(),
        sub: vbt.user.clone(),
        aud: "Trow Registry".to_owned(),
        exp: current_time.add(Duration::new(TOKEN_DURATION, 0)).as_secs(),
        nbf: current_time.as_secs(),
        iat: current_time.as_secs(),
        jti: Uuid::new_v4().to_string(),
    };

    let header = json!({});
    let payload = serde_json::to_value(token_claim)?;

    //Use generated config here
    let token = encode(header, &config.token_secret, &payload, Algorithm::HS256)?;

    Ok(TrowToken {
        user: vbt.user,
        token,
    })
}
/*
 * Responder returns token as JSON body
 */
impl IntoResponse for TrowToken {
    fn into_response(self) -> Response {
        //TODO: would be better to use serde here
        let formatted_body = format!("{{\"token\": \"{}\"}}", self.token);
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, formatted_body.len())
            .status(StatusCode::OK)
            .body(body::Body::from(formatted_body))
            .unwrap()
            .into_response()
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for TrowToken
where
    TrowConfig: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Authenticate;

    async fn from_request_parts(req: &mut Parts, config: &S) -> Result<Self, Self::Rejection> {
        let config = &TrowConfig::from_ref(config);
        let base_url = get_base_url(&req.headers, config);

        if config.user.is_none() {
            //Authentication is not configured
            //TODO: Figure out how to create this only once
            let no_auth_token = TrowToken {
                user: "none".to_string(),
                token: "none".to_string(),
            };
            return Ok(no_auth_token);
        }
        let authorization = match req
            .headers
            .typed_get::<headers::Authorization<headers::authorization::Bearer>>()
        {
            Some(bt) => bt,
            None => return Err(Authenticate::new(base_url)),
        };
        let token = authorization.token();

        // parse for bearer token
        // TODO: frank_jwt is meant to verify iat, nbf etc, but doesn't.
        let dec_token = match decode(
            token,
            &config.token_secret,
            Algorithm::HS256,
            &ValidationOptions::default(),
        ) {
            Ok((_, payload)) => payload,
            Err(_) => {
                event!(Level::WARN, "Failed to decode user token");
                return Err(Authenticate::new(base_url));
            }
        };

        let trow_token = TrowToken {
            user: dec_token["sub"].to_string(),
            token: token.to_string(),
        };

        Ok(trow_token)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test() {}
}
