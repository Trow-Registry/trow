use failure::{Error};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::http::{Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use crypto::sha2::Sha256;
use chrono::Local;
use jwt::{Header as TokenHeader, Token};

const AUTHORISATION_SECRET: &str = "Bob Marley Rastafaria";
const EXPIRY_TIME: u64 = 3600;

#[derive(Debug, Serialize)]
pub struct TrowToken;

#[derive(Debug, Serialize, RustcEncodable, RustcDecodable)]
pub struct BearerToken {
    user_id: String,
    client_id: String,
    scope: String,
    iat: u64,
    exp: u64,
}

fn encode_token() -> Result<String, Error> {
    // hard coded values for token
    let username = "admin";
    let client_id = "docker";
    let scope = "push/pull";
    let now = SystemTime::now();
    let current_time = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // build token from structure and return token string
    let token_claims = BearerToken {
        user_id: username.to_string(),
        client_id: client_id.to_string(),
        scope: scope.to_string(),
        iat: current_time.as_secs(),
        exp: EXPIRY_TIME,
    };
    let token_header: TokenHeader = Default::default();
    let bearer_token = Token::new(token_header, token_claims);

    let token_enum = bearer_token.signed(AUTHORISATION_SECRET.as_bytes(), Sha256::new()).ok();
    let mut token_string = String::new();
    /*
    match token_enum {
        Some(token_enum) => token_string = token_enum,
        _ => (),
    };
     */
    if let Some(token_enum) = token_enum { token_string = token_enum };
    Ok(token_string)
}

impl<'r> Responder<'r> for TrowToken {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
        // create token string and put it in response header
        let token_string = encode_token();
        match token_string {
            Ok(token_string) => {
                let date = Local::now();
                let formatted_date =  date.format("%Y-%m-%dT%H:%M:%SZ");
                let formatted_string=format!("{{\"token\":\"{}\",\"expires_in\":{},\"issued_at\":\"{}\"}}/n", token_string, EXPIRY_TIME, formatted_date);
                let formatted_body = Cursor::new(formatted_string);
                Response::build()
                    .status(Status::Ok)
                    .header(ContentType::JSON)
                    .sized_body(formatted_body)
                    .ok()
            }
            _ => {
                debug!("CATCHALL!");
                Response::build().status(Status::Unauthorized).ok()
            }
        }
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
        let headers = response.headers();
        assert_eq!(response.status(), Status::Ok);
//        assert_eq!(headers.contains(""))
    }
}
