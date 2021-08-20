use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use crate::types::ReadinessResponse;

impl<'r> Responder<'r, 'static> for ReadinessResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'static>, Status> {
        let json = serde_json::to_string(&self).unwrap_or_default();

        match self.is_ready {
            true => Response::build()
                .header(ContentType::JSON)
                .sized_body(None, Cursor::new(json))
                .status(Status::Ok)
                .ok(),
            false => Response::build()
                .header(ContentType::JSON)
                .sized_body(None, Cursor::new(json))
                .status(Status::ServiceUnavailable)
                .ok(),
        }
    }
}
#[cfg(test)]
mod test {
    use crate::response::test_helper::test_client;
    use crate::types::ReadinessResponse;
    use rocket::http::Status;
    use rocket::response::Responder;

    fn build_ready_response() -> ReadinessResponse {
        ReadinessResponse {
            message: String::from("Ready"),
            is_ready: true,
        }
    }

    fn build_not_ready_response() -> ReadinessResponse {
        ReadinessResponse {
            message: String::from("Not Ready"),
            is_ready: false,
        }
    }

    #[test]
    fn test_ready_resp() {
        let cl = test_client();
        let req = cl.get("/");
        let response = build_ready_response().respond_to(req.inner()).unwrap();

        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_not_ready_resp() {
        let cl = test_client();
        let req = cl.get("/");
        let response = build_not_ready_response().respond_to(req.inner()).unwrap();
        assert_eq!(response.status(), Status::ServiceUnavailable);
    }
}
