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
//        let authenticate_header = Header::new("www-authenticate:","Bearer realm=https://0.0.0.0:8080/token");

//        let authenticate_header = Header::new("www-authenticate","Bearer realm=\"http://0.0.0.0:8080/token\",service=\"demo_registry\",scope=\"registry:catalog:*\"");
        let authenticate_header = Header::new("www-authenticate","Basic realm=\"https://0.0.0.0:8443/token\",service=\"trow_registry\",scope=\"wanker\"");
        
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
