use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

use crate::registry_interface::MetricsResponse;

impl<'r> Responder<'r, 'static> for MetricsResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'static>, Status> {
        Response::build()
            .header(ContentType::Plain)
            .sized_body(None, Cursor::new(self.metrics))
            .status(Status::Ok)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use crate::registry_interface::MetricsResponse;
    use crate::response::test_helper::test_client;
    use rocket::http::Status;
    use rocket::response::Responder;

    fn build_metrics_response() -> MetricsResponse {
        MetricsResponse {
            metrics: String::from("# HELP available_space ...."),
        }
    }

    #[test]
    fn test_metrics_resp() {

        let cl = test_client();
        let req = cl.get("/");
        let response = build_metrics_response().respond_to(req.inner()).unwrap();
        assert_eq!(response.status(), Status::Ok);
    }
}
