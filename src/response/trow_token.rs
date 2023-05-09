use crate::TrowConfig;
use crate::UserConfig;
use base64::{engine::general_purpose as base64_engine, Engine as _};
use frank_jwt::{decode, encode, Algorithm, ValidationOptions};
use log::warn;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{Responder, Response};
use rocket::{outcome::Outcome, State};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Cursor;
use std::ops::Add;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const TOKEN_DURATION: u64 = 3600;
const AUTHORIZATION: &str = "authorization";

pub struct ValidBasicToken {
    user: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ValidBasicToken {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<ValidBasicToken, ()> {
        let config = req
            .guard::<&rocket::State<TrowConfig>>()
            .await
            .expect("TrowConfig not present!");

        let user_cfg = match config.user {
            Some(ref user_cfg) => user_cfg,
            None => {
                warn!("Attempted login, but no users are configured");
                return Outcome::Failure((Status::Unauthorized, ()));
            }
        };

        // As Authorization is a standard header
        let auth_val = match req.headers().get_one(AUTHORIZATION) {
            Some(a) => a,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        // The value of the header is the type of the auth (Basic or Bearer), followed by an
        // encoded string, separate by whitespace.
        let auth_strings: Vec<String> = auth_val.split_whitespace().map(String::from).collect();
        if auth_strings.len() != 2 {
            //TODO: Should this be BadRequest?
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        // We're looking for a Basic token
        if auth_strings[0] != "Basic" {
            //TODO: This probably isn't right, maybe check if bearer?
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        match base64_engine::STANDARD_NO_PAD.decode(&auth_strings[1]) {
            Ok(user_pass) => {
                if verify_user(user_pass, user_cfg) {
                    Outcome::Success(ValidBasicToken {
                        user: user_cfg.user.clone(),
                    })
                } else {
                    Outcome::Failure((Status::Unauthorized, ()))
                }
            }
            Err(_) => Outcome::Failure((Status::Unauthorized, ())),
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
pub fn new(vbt: ValidBasicToken, tc: &State<TrowConfig>) -> Result<TrowToken, frank_jwt::Error> {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // build token from structure and return token string
    let token_claim = TokenClaim {
        iss: tc.service_name.clone(),
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
    let token = encode(header, &tc.token_secret, &payload, Algorithm::HS256)?;

    Ok(TrowToken {
        user: vbt.user,
        token,
    })
}
/*
 * Responder returns token as JSON body
 */
impl<'r> Responder<'r, 'static> for TrowToken {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        //TODO: would be better to use serde here
        let formatted_body = Cursor::new(format!("{{\"token\": \"{}\"}}", self.token));
        Response::build()
            .status(Status::Ok)
            .header(ContentType::JSON)
            .sized_body(None, formatted_body)
            .ok()
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for TrowToken {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> request::Outcome<TrowToken, ()> {
        let config = req
            .guard::<&rocket::State<TrowConfig>>()
            .await
            .expect("TrowConfig not present!");

        if config.user.is_none() {
            //Authentication is not configured
            //TODO: Figure out how to create this only once
            let no_auth_token = TrowToken {
                user: "none".to_string(),
                token: "none".to_string(),
            };
            return Outcome::Success(no_auth_token);
        }
        let auth_val = match req.headers().get_one("Authorization") {
            Some(a) => a,
            None => return Outcome::Failure((Status::Unauthorized, ())),
        };

        // Check header handling - isn't there a next?
        // split the header on white space
        let auth_strings: Vec<String> = auth_val.split_whitespace().map(String::from).collect();
        if auth_strings.len() != 2 {
            return Outcome::Failure((Status::BadRequest, ()));
        }
        // We're looking for a Bearer token
        //TODO: Maybe should forward or something on Basic
        if auth_strings[0] != "Bearer" {
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        // parse for bearer token
        // TODO: frank_jwt is meant to verify iat, nbf etc, but doesn't.

        let dec_token = match decode(
            &auth_strings[1],
            &config.token_secret,
            Algorithm::HS256,
            &ValidationOptions::default(),
        ) {
            Ok((_, payload)) => payload,
            Err(_) => {
                warn!("Failed to decode user token");
                return Outcome::Failure((Status::Unauthorized, ()));
            }
        };

        let trow_token = TrowToken {
            user: dec_token["sub"].to_string(),
            token: auth_strings[1].clone(),
        };

        Outcome::Success(trow_token)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test() {}
}
