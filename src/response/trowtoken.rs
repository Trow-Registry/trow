//extern crate data_encoding;
//extern crate ring;

use failure::{format_err, Error};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
//use rocket::http::Status;
//use rocket::request::Request;
//use rocket::response::{Responder, Response};
//use types::Token;
use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};

//use data_encoding::HEXUPPER;
//use ring::{digest, pbkdf2, rand};
//use ring::rand::SecureRandom;

#[derive(Debug, Serialize)]
pub struct TrowToken;
#[derive(Debug, Serialize, RustcEncodable, RustcDecodable)]
pub struct TroutToken {
    userId: String,
    clientId: String,
    scope: String,
    iat: u64,
    exp: u64,
}
static AUTHORISATION_SECRET: &'static str = "Bob Marley Rastafaria";
/*
example token fields;
access_token: token_string
token_type: bearer
expires_in: expiry_time
refresh_token: refresh_token
scope: scope

example token headers:
Cache-Control: no-store
Pragma: no-cache

Example:
HTTP/1.1 200 OK
Content-Type: application/json
Cache-Control: no-store
Pragma: no-cache
 
{
  "access_token":"MTQ0NjJkZmQ5OTM2NDE1ZTZjNGZmZjI3",
  "token_type":"bearer",
  "expires_in":3600,
  "refresh_token":"IwOGYzYTlmM2YxOTQ5MGE3YmNmMDFkNTVk",
  "scope":"create"
}

Bad Request example:
HTTP/1.1 400 Bad Request
Content-Type: application/json;charset=UTF-8
Cache-Control: no-store
Pragma: no-cache
 
{
  "error": "invalid_request",
  "error_description": "Request was missing the 'redirect_uri' parameter.",
  "error_uri": "See the full API docs at https://authorization-server.com/docs/access_token"
}
 */

/*
{
   "typ":"JWT",
   "alg":"HS256”
 }

{
  "sub": 1000,
  "iss": "https://authorization-server.com",
  "cid": "https://example-app.com",
  "iat": 1470002703,
  "exp": 1470009903,
  "scope": "read write"
}

Base64-encoding the first two components

calculate a hash of the two strings along with a secret,

concatenate all three strings together separated by periods
*/

/*
encode in token

user_id
client_id
scope
iat
exp
scope
*/
fn encode_token() -> Result<String, Error> {
    use crypto::sha2::Sha256;
    use jwt::{Header, Token};

    let username = "admin";
    let client_id = "docker";
    let scope = "push/pull";
    let now = SystemTime::now();
    let current_time = now.duration_since(UNIX_EPOCH);
    let expiry_time = 3600;

    let claims = TroutToken {
        userId: username.to_string(),
        clientId: client_id.to_string(),
        scope: scope.to_string(),
        iat: 234523456, // now.as_secs(),
        exp: expiry_time,
    };
    let header: Header = Default::default();
    let token = Token::new(header, claims);


    println!("{:?}", token);
    /*
    Token { raw: None, header: Header { typ: Some(JWT), kid: None, alg: HS256 }, claims: TroutToken { userId: "admin", clientId: "docker", scope: "push/pull", iat: 234523456, exp: 3600 } }
    */
    // let signed =
    let token_string = token.signed(AUTHORISATION_SECRET.as_bytes(), Sha256::new()).ok();
//    println!("{:?}", token_string);
    /*
    Some("eyJ0eXAiOiJKV1QiLCJraWQiOm51bGwsImFsZyI6IkhTMjU2In0.eyJ1c2VySWQiOiJhZG1pbiIsImNsaWVudElkIjoiZG9ja2VyIiwic2NvcGUiOiJwdXNoL3B1bGwiLCJpYXQiOjIzNDUyMzQ1NiwiZXhwIjozNjAwfQ.MKDAir42OCVyHOlC7fH1f9iVnvz7e3/IzCiV1gBVUzY")
    */
//    println!("{:?}", Some(token_string));
    /*
    Some(Some("eyJ0eXAiOiJKV1QiLCJraWQiOm51bGwsImFsZyI6IkhTMjU2In0.eyJ1c2VySWQiOiJhZG1pbiIsImNsaWVudElkIjoiZG9ja2VyIiwic2NvcGUiOiJwdXNoL3B1bGwiLCJpYXQiOjIzNDUyMzQ1NiwiZXhwIjozNjAwfQ.MKDAir42OCVyHOlC7fH1f9iVnvz7e3/IzCiV1gBVUzY"))
    */
    match token_string {
        Some(token_string) => Ok(token_string),
//        Some(token_string) => println!("token string is {}", token_string),
        _ => Err(Error::InternalError)
    }
    /*
    trow::response::trowtoken[DEBUG] token response
-----------------------------------------------------------------------------
trow::response::trowtoken[DEBUG] current time is SystemTime { tv_sec: 1550048621, tv_nsec: 65616687 }
trow::response::trowtoken[DEBUG] self TrowToken

        */

    // let test_string=Some(token_string);

//    println!("test string is {}", test_string);
//    format!("{}", token_string);
    /*
    let testy_this = token.signed(AUTHORISATION_SECRET.as_bytes(), Sha256::new()).ok();
    match testy_this {
        Ok(testy_this) => println!("pass string is {}", testy_this),
        Err(_) => println!("error of some prescription")
    };
    return Ok(token_string);
    */
    Ok("test".to_string())
    /*
  const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
  const N_ITER: u32 = 100_000;
  let rng = rand::SystemRandom::new();

  let mut salt = [0u8; CREDENTIAL_LEN];
  rng.fill(&mut salt)?;

  let password = "Guess Me If You Can!";
  let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
  pbkdf2::derive(
      &digest::SHA512,
      N_ITER,
      &salt,
      password.as_bytes(),
      &mut pbkdf2_hash,
  );
  println!("Salt: {}", HEXUPPER.encode(&salt));
  println!("PBKDF2 hash: {}", HEXUPPER.encode(&pbkdf2_hash));

  let should_succeed = pbkdf2::verify(
      &digest::SHA512,
      N_ITER,
      &salt,
      password.as_bytes(),
      &pbkdf2_hash,
  );
  let wrong_password = "Definitely not the correct password";
  let should_fail = pbkdf2::verify(
      &digest::SHA512,
      N_ITER,
      &salt,
      wrong_password.as_bytes(),
      &pbkdf2_hash,
  );

  assert!(should_succeed.is_ok());
  assert!(!should_fail.is_ok());
*/
//    signed
}

impl<'r> Responder<'r> for TrowToken {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
//        let token_header = Header::new(("{\"token\":\"randomtokenstring09876543210960987654321\",\"expires_in\":300,\"issued_at\":\"2019-01-31T09:05:33.678171359Z\"}/n"));
//            "Authorization","Bearer randomtokenstring1234567890");
//        let token_body=Cursor::new("{\"token\":\"randomtokenstring09876543210960987654321\",\"expires_in\":300,\"issued_at\":\"2019-01-31T09:05:33.678171359Z\"}/n");
        let token_body = Cursor::new("{\"token\":\"randomtokenstring09876543210960987654321\",\"expires_in\":300,\"issued_at\":\"2019-01-31T09:05:33.678171359Z\"}/n");
        println!("   ");
        debug!("token response"); 
        println!("-----------------------------------------------------------------------------");
        let current_time = SystemTime::now();
        let token_string = encode_token();
        debug!("current time is {:?}", current_time);
        debug!("self {:?}", self);
//        debug!("Request {:?}", Request);
        Response::build()
            .status(Status::Ok)
//            .header(token_header)
            .header(ContentType::JSON)
            .sized_body(token_body)
            .ok()
    }
}

/*
impl Responder<'static> for String {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        Response::build()
            .header(ContentType::Plain)
            .sized_body(Cursor::new(self))
            .ok()
    }
}
*/
/*
#[cfg(test)]
mod test {
    use response::token::Token;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn token_ok() {
        let response = test_route(Token);
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
*/
