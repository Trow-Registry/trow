use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use crate::types::ReadinessResponse;

impl<'r> Responder<'r> for ReadinessResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        let json = serde_json::to_string(&self).unwrap_or_default();

        match self.is_ready {
            true => Response::build()
                .header(ContentType::JSON)
                .sized_body(Cursor::new(json))
                .status(Status::Ok)
                .ok(),
            false => Response::build()
                .header(ContentType::JSON)
                .sized_body(Cursor::new(json))
                .status(Status::ServiceUnavailable)
                .ok(),
        }
    }
}
#[cfg(test)]
mod test {
    use crate::response::test_helper::test_route;
    use crate::types::ReadinessResponse;
    use rocket::http::Status;

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
        let response = test_route(build_ready_response());
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    fn test_not_ready_resp() {
        let response = test_route(build_not_ready_response());
        assert_eq!(response.status(), Status::ServiceUnavailable);
    }
}
