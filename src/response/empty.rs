use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;

#[derive(Debug, Serialize)]
pub struct Empty;

impl<'r> Responder<'r> for Empty {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        Response::build().ok()
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::empty::Empty;

    use test::test_helpers::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Empty);
        assert_eq!(response.status(), Status::Ok);
    }
}
