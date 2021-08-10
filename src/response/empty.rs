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
    use rocket::{http::Status, response::Responder};

    use crate::response::test_helper::test_client;

    #[test]
    fn empty_ok() {
        
        let cl = test_client();
        let req = cl.get("/");
        let response = Empty{}.respond_to(req.inner()).unwrap();
        assert_eq!(response.status(), Status::Ok);
    }
}
