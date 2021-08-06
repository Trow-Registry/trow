use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

#[derive(Debug, Serialize)]
pub struct Empty;

impl<'r> Responder<'r, 'static> for Empty {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        Response::build().ok()
    }
}

#[cfg(test)]
mod test {
    use crate::response::empty::Empty;
    use rocket::http::Status;

    use crate::response::test_helper::test_route;

    #[test]
    fn empty_ok() {
        let response = test_route(Empty);
        assert_eq!(response.status(), Status::Ok);
    }
}
