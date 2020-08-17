use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use crate::types::MetricsResponse;

impl<'r> Responder<'r> for MetricsResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .header(ContentType::Plain)
            .sized_body(Cursor::new(self.metrics))
            .status(Status::Ok)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use crate::response::test_helper::test_route;
    use crate::types::MetricsResponse;
    use rocket::http::Status;

    fn build_metrics_response() -> MetricsResponse {
        MetricsResponse {
            metrics: String::from("# HELP available_space ...."),
        }
    }

    #[test]
    fn test_metrics_resp() {
        let response = test_route(build_metrics_response());
        assert_eq!(response.status(), Status::Ok);
    }
}
