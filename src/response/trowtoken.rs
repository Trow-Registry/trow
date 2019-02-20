use failure::{Error};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::http::{Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use crypto::sha2::Sha256;
use chrono::Local;
use routes::AuthorisedUser;
use jwt::{Header as TokenHeader, Token};

const AUTHORISATION_SECRET: &str = "Bob Marley Rastafaria";
const EXPIRY_TIME: u64 = 3600;

#[derive(Debug, Serialize, RustcEncodable, RustcDecodable)]
pub struct TrowToken {
    user_id: String,
    client_id: String,
    scope: String,
    iat: u64,
    exp: u64,
    signed_token: String
}

#[derive(Clone, Debug, Serialize, RustcEncodable, RustcDecodable)]
struct TokenClaim {
    user_id: String,
    client_id: String,
    scope: String,
    iat: u64,
    exp: u64,
}

pub fn new(auth_user: AuthorisedUser) -> Result<TrowToken, jwt::Error> {
    // hard coded values for token
    let username = auth_user.username;
    let client_id = "docker";
    let scope = "push/pull";
    let now = SystemTime::now();
    let current_time = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // build token from structure and return token string
    let token_claims = TokenClaim {
        user_id: username.to_string(),
        client_id: client_id.to_string(),
        scope: scope.to_string(),
        iat: current_time.as_secs(),
        exp: EXPIRY_TIME,
    };
    let token_header: TokenHeader = Default::default();
    let bearer_token = Token::new(token_header, token_claims.clone());

    let sig = bearer_token.signed(AUTHORISATION_SECRET.as_bytes(), Sha256::new())?;
  
    Ok(TrowToken{
            user_id: token_claims.user_id,
            client_id: token_claims.client_id,
            scope: token_claims.scope,
            iat: token_claims.iat,
            exp: token_claims.exp,
            signed_token: sig
    })
}

impl<'r> Responder<'r> for TrowToken {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        let date = Local::now();
        let formatted_date =  date.format("%Y-%m-%dT%H:%M:%SZ");
        let formatted_string=format!("{{\"token\":\"{}\",\"expires_in\":{},\"issued_at\":\"{}\"}}/n", self.signed_token, EXPIRY_TIME, formatted_date);
        let formatted_body = Cursor::new(formatted_string);
        Response::build()
            .status(Status::Ok)
            .header(ContentType::JSON)
            .sized_body(formatted_body)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use response::trowtoken::TrowToken;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn token_ok() {
        let response = test_route(TrowToken);
        let _headers = response.headers();
        assert_eq!(response.status(), Status::Ok);
//        assert_eq!(headers.contains(""))
    }
}
