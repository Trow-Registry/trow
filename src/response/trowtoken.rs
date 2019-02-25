use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::http::{Status};
use rocket::http::{ContentType};
use rocket::request::{self, FromRequest, Request};
use rocket::response::{Responder, Response};
use rocket::Outcome;
use crypto::sha2::Sha256;
use chrono::Local;
use routes::AuthorisedUser;
use frank_jwt::{Header, Payload, encode, decode, Algorithm};

const AUTHORISATION_SECRET: &str = "Bob Marley Rastafaria";
const EXPIRY_TIME: &str = "3600";

#[derive(Debug, Serialize, RustcEncodable, RustcDecodable)]
pub struct TrowToken {
    user_id: String,
    client_id: String,
    scope: String,
    issued_at: u64,
    expires: String,
    signed_token: String
}

#[derive(Clone, Debug, Serialize, RustcEncodable, RustcDecodable)]
struct TokenClaim {
    user_id: String,
    client_id: String,
    scope: String,
    issued_at: u64,
    expires: String,
}
/*
 * Create new jsonwebtoken.
 * Token consists of a string with 3 comma separated fields header, payload, signature
 */
pub fn new(auth_user: AuthorisedUser) -> Result<TrowToken, frank_jwt::Error> {
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
        issued_at: current_time.as_secs(),
        expires: EXPIRY_TIME,
    };
    let token_header = Header::new(Algorithm::HS256);
    let mut payload = json!({
        "username", username.to_string(),
        "iss": "trow",
        "sub": "docker",
        "iat": current_time.as_secs().to_string(),
        "exp": EXPIRY_TIME,
        "scope": "push/pull"
    })
    let bearer_token_string = encode(token_header, AUTHORISATION_SECRET, payload.clone())?;

    Ok(TrowToken{
            user_id: username.to_string(),
            client_id: "docker",
            scope: "push/pull",
            issued_at: current_time.as_secs(),
            expires: EXPIRY_TIME,
            signed_token: bearer_token_string
    })
}
/*
 * Responder returns valid bearer token
 */
impl<'r> Responder<'r> for TrowToken {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        // duplication - want to use time from token struct
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
/*
 * 
 */
impl<'a, 'r> FromRequest<'a, 'r> for TrowToken {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<TrowToken, ()> {
        // Look in headers for an Authorization header
        let keys: Vec<_> = req.headers().get("Authorization").collect();
        if keys.len() != 1 { // no key return false in auth structure
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        // split the header on white space
        let auth_strings: Vec<String> = keys[0].to_string().split_whitespace().map(String::from).collect();
        if auth_strings.len() !=2 {
            //TODO: Maybe BadRequest?
            return Outcome::Failure((Status::Unauthorized, ()));
        }

        // Basic token is a base64 encoded user/pass
        if auth_strings[0] != "Bearer" {
            //TODO: Maybe should forward or something on Basic
            return Outcome::Failure((Status::Unauthorized, ()));
        } 
        // parse for bearer token and verify it
//        let token = Token::<Header, Registered>::parse(&auth_strings[1]).unwrap();
        /*
        if token.verify(AUTHORISATION_SECRET.as_bytes(), Sha256::new()) {
            let ttoken = TrowToken {
                username: token.header.
                authorized: true,
            };
        }
        */
        Outcome::Success(auth_user)
    }
}


#[cfg(test)]
mod test {
    use response::trowtoken;
    use rocket::http::Status;
    use routes::AuthorisedUser;

    use response::test_helper::test_route;

    #[test]
    fn token_ok() {
        let user = AuthorisedUser {
            username: "admin".to_string(),
            authorized: true,
        };
        let response = test_route(trowtoken::new(user));
        assert_eq!(response.status(), Status::Ok);
    }
}
