use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use response::RegistryTrait;

#[derive(Debug, Serialize)]
pub struct Empty;
DummyResponder!(Empty);

impl RegistryTrait for Empty {
    fn ok<'r>(&self, _req: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .ok()
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::empty::Empty;

    use test::test_helpers::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Ok(Empty));
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn empty_err() {
        let response = test_route(Err(Empty));
        assert_eq!(response.status(), Status::NotFound);
    }
}
