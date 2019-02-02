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
/*
example token
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IklQNU06RDM0UDpXSURROkVJWkw6RzVZRzpKRlhFOkVUTTM6TFlYMjpTWk1WOk5UNks6V1NGTTpZTFM2In0A.eyJzdWIiOiIiLCJpc3MiOiJkZW1vX29hdXRoIiwiYWNjZXNzIjpbeyJ0eXBlIjoicmVnaXN0cnkiLCJuYW1lIjoiY2F0YWxvZyIsImFjdGlvbnMiOlsiKiJdfV0sImV4cCI6MTU0Njg2Njg0NiwiaWF0IjoxNTQ2ODYzMjQ2LCJuYmYiOjE1NDY4NjMyMTYsImF1ZCI6ImRlbW9fcmVnaXN0cnkifQ.pi-Ua_P6bH6zur0Czsqv-_1_kgl7uVkM1aw2HSpu04P1Q6OMeob4eqh_koktpMlS9rcLgl7EAiPIBlkgBrD5OIVOHbIodPk1YuqrO3ZfVB2pkrwYi6ZttI6t3ehLBsk6e5p8Deam_EhYux7wtcwWwMU11_qj94_-vbBO215JsjkJlCuui3Vv_zpeH3j_tN4XfBtyRKMNjfMsCCRmTdHRYt5I8ZqN_XwlXtSyK-wfvM1__a6R7HgOMlBaTaEtAAHO64u7iPlMTOsA-lQahE-T5sb4N4I1YWg1-aLLWpsYN
n7cQZF1jDskZZNZhotlPp7Uc3PL7eFL3t2y7hEAg4Bzxg

*/
/*
token fields;
access_token: token_string
token_type: bearer
expires_in: expiry_time
refresh_token: refresh_token
scope: scope

token headers:
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
encode in toekn

user_id
client_id
scope
iat
exp
scope
*/
fn encode_token() -> Result<(), Error> {
    /*
    use jwt::{Header, Token};
    let claims = SigningManifest {
        fsLayers: vec!(BlobSummary { blobSum: digest.to_owned() }),
        ..Default::default()
    };

    let manifest = Manifest::from_signing_manifest(&claims);
    let token = Token::new(header, claims);


    println!("{:?}", manifest);
    let signed = token.signed(b"secret_key", Sha256::new()).ok();
    println!("{:?}", signed);
*/
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
    Ok(())
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
        encode_token();
        debug!("current time is {:?}", current_time);
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
