use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use crate::types::HealthResponse;

impl<'r> Responder<'r> for HealthResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {

        let json = serde_json::to_string(&self).unwrap_or_default();

        match self.is_healthy {
            true => {
                Response::build()
                    .header(ContentType::JSON)
                    .sized_body(Cursor::new(json))
                    .status(Status::Ok)
                    .ok()
            },
            false => {
                Response::build()
                    .header(ContentType::JSON)
                    .sized_body(Cursor::new(json))
                    .status(Status::ServiceUnavailable)
                    .ok()
            }

        }
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use crate::types::{HealthResponse};
    use crate::response::test_helper::test_route;
    
    fn build_response() -> HealthResponse {
        HealthResponse {
            status: String::from("OK"),
            message: String::from("Healthy"),
            is_healthy: true
        }
    }

    #[test]
    fn test_resp() {
        let response = test_route(build_response());
        assert_eq!(response.status(), Status::Ok);
      
    }
}
