use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};
/*
WWW-Authenticate: Basic

WWW-Authenticate: Basic realm="Access to the staging site", charset="UTF-8"
*/
#[derive(Debug, Serialize)]
pub struct Authenticate;

impl<'r> Responder<'r> for Authenticate {
    fn respond_to(self, _: &Request)  -> Result<Response<'r>, Status> {
        println!("   ");
        debug!("www-authenticate response"); 
        println!("-----------------------------------------------------------------------------");
        let authenticate_header = Header::new("www-authenticate","Bearer realm=\"https://0.0.0.0:8443/token\",service=\"trow_registry\",scope=\"unused\"");
        Response::build()
            .status(Status::Unauthorized)
            .header(authenticate_header)
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

#[cfg(test)]
mod test {
    use response::token::Token;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn authenticate_ok() {
        let response = test_route(Token);
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
