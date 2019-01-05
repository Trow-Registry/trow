//use rocket::http::Status;
use rocket::http::{Header, Status};
use rocket::http::{ContentType};
use rocket::request::Request;
use rocket::response::{Responder, Response};

#[derive(Debug, Serialize)]
pub struct Empty;

impl<'r> Responder<'r> for Empty {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
//        Response::build().ok()
        debug!("hijacked empty response");
        let authenticate_header = Header::new("Www-Authenticate:","Bearer realm=https://0.0.0.0:8080/tokens");
        Response::build()
            .status(Status::Unauthorized)
            .header(authenticate_header)
            .header(ContentType::JSON)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use response::empty::Empty;
    use rocket::http::Status;

    use response::test_helper::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Empty);
        assert_eq!(response.status(), Status::Ok);
    }
}
