use failure::{format_err, Error};
use std::io::Cursor;
use std::time::{SystemTime, UNIX_EPOCH};
use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use crypto::sha2::Sha256;
use jwt::{Header as TokenHeader, Token};

#[derive(Debug, Serialize)]
pub struct TrowToken;
#[derive(Debug, Serialize, RustcEncodable, RustcDecodable)]
pub struct TroutToken {
    user_id: String,
    client_id: String,
    scope: String,
    iat: u64,
    exp: u64,
}

const AUTHORISATION_SECRET: &'static str = "Bob Marley Rastafaria";

fn encode_token() -> Result<String, Error> {
    // hard coded values for token
    let username = "admin";
    let client_id = "docker";
    let scope = "push/pull";
    let now = SystemTime::now();
    let current_time = now.duration_since(UNIX_EPOCH);
    let expiry_time = 3600;

    // build token from structure and return token string
    let token_claims = TroutToken {
        user_id: username.to_string(),
        client_id: client_id.to_string(),
        scope: scope.to_string(),
        iat: 234523456, // now.as_secs(),
        exp: expiry_time,
    };
    let token_header: TokenHeader = Default::default();
    let bearer_token = Token::new(token_header, token_claims);

    let token_enum = bearer_token.signed(AUTHORISATION_SECRET.as_bytes(), Sha256::new()).ok();
    let mut token_string = String::new();
    match token_enum {
        Some(token_enum) => token_string = token_enum,
        _ => (),
    };
    println!("return string is {}", token_string);
    Ok(token_string)
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
        let expiry_time=3600;
// create token string and put it in response header
        let token_string = encode_token();
        println!("token string is {:?}", token_string);
//        let header_string = String::new;
//        let formatted_string = format!("test {}", token_string)
        match token_string {
            Ok(token_string) => {
                let formatted_string=format!("{{\"token\":\"{}\",\"expires_in\":{},\"issued_at\":\"2019-01-31T09:05:33.678171359Z\"}}/n", token_string, expiry_time);
                println!("formatted estring is {:?}", formatted_string);
                let formatted_body = Cursor::new(formatted_string);
                println!("formatted body is {:?}", formatted_body);
                Response::build()
                    .status(Status::Ok)
                //            .header(token_header)
                    .header(ContentType::JSON)
                    .sized_body(formatted_body)
                    .ok()
            }
            _ => {
                println!("CATCHALL!");
                Response::build().status(Status::Unauthorized).ok()
            }
        }
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
