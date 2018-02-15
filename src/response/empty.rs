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

#[derive(Debug, Serialize)]
pub struct Created;

impl<'r> Responder<'r> for Created {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .status(Status::Created)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::empty::Empty;

    use response::test_helper::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Empty);
        assert_eq!(response.status(), Status::Ok);
    }


    #[test]
    fn accepted_ok() {
        let response = test_route(Created);
        assert_eq!(response.status(), Status::Created);
    }
}
