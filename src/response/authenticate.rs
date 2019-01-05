use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use types::Authenticate;
/*
WWW-Authenticate: Basic

WWW-Authenticate: Basic realm="Access to the staging site", charset="UTF-8"
*/
#[derive(Debug, Serialize)]
pub struct Authenticate;

impl<'r> Responder<'r> for Authenticate {
    fn respond_to(self, _req: &Request) -> response::Result<'r> {
        debug!("authenticate response"); 
        Response::build()
            let authenticate_header = Header::new("Www-Authenticate: Bearer realm=https://0.0.0.0:8080/tokens")
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
    use response::empty::Empty;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Empty);
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
