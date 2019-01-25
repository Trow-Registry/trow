//use rocket::http::Status;
//use rocket::request::Request;
//use rocket::response::{Responder, Response};
//use types::Token;
use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
/*
WWW-Authenticate: Basic

WWW-Authenticate: Basic realm="Access to the staging site", charset="UTF-8"
*/
#[derive(Debug, Serialize)]
pub struct Token;
/*
example token
eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IklQNU06RDM0UDpXSURROkVJWkw6RzVZRzpKRlhFOkVUTTM6TFlYMjpTWk1WOk5UNks6V1NGTTpZTFM2In0A.eyJzdWIiOiIiLCJpc3MiOiJkZW1vX29hdXRoIiwiYWNjZXNzIjpbeyJ0eXBlIjoicmVnaXN0cnkiLCJuYW1lIjoiY2F0YWxvZyIsImFjdGlvbnMiOlsiKiJdfV0sImV4cCI6MTU0Njg2Njg0NiwiaWF0IjoxNTQ2ODYzMjQ2LCJuYmYiOjE1NDY4NjMyMTYsImF1ZCI6ImRlbW9fcmVnaXN0cnkifQ.pi-Ua_P6bH6zur0Czsqv-_1_kgl7uVkM1aw2HSpu04P1Q6OMeob4eqh_koktpMlS9rcLgl7EAiPIBlkgBrD5OIVOHbIodPk1YuqrO3ZfVB2pkrwYi6ZttI6t3ehLBsk6e5p8Deam_EhYux7wtcwWwMU11_qj94_-vbBO215JsjkJlCuui3Vv_zpeH3j_tN4XfBtyRKMNjfMsCCRmTdHRYt5I8ZqN_XwlXtSyK-wfvM1__a6R7HgOMlBaTaEtAAHO64u7iPlMTOsA-lQahE-T5sb4N4I1YWg1-aLLWpsYN
n7cQZF1jDskZZNZhotlPp7Uc3PL7eFL3t2y7hEAg4Bzxg

*/
impl<'r> Responder<'r> for Token {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        println!("   ");
        debug!("token response"); 
        println!("-----------------------------------------------------------------------------");
        let token_header = Header::new(
            "www-authenticate",
            "Basic realm=\"http://0.0.0.0:8443/basic\",service=\"trow_registry\",scope=\"registry:catalog:*\"");
        Response::build()
            .status(Status::Unauthorized)
            .header(token_header)
            .header(ContentType::JSON)
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
