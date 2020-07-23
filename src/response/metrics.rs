use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};


use crate::types::MetricsResponse;

impl<'r> Responder<'r> for MetricsResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        match self.errored {
            false => {
                let text = self.metrics;
        
                Response::build()
                .header(ContentType::Plain)
                .sized_body(Cursor::new(text))
                .status(Status::Ok)
                .ok()
            },
            true => {
                let text = self.message;
        
                Response::build()
                .header(ContentType::Plain)
                .sized_body(Cursor::new(text))
                .status(Status::ServiceUnavailable)
                .ok()
            }   
        }
    }
}
#[cfg(test)]
mod test {
    use rocket::http::Status;
    use crate::types::{MetricsResponse};
    use crate::response::test_helper::test_route;
    
    fn build_metrics_response() -> MetricsResponse {
        MetricsResponse {
            message: String::from(""),
            errored: false,
            metrics: String::from("# HELP available_space ....")
        }
    }

    fn build_erroed_metrics_response() -> MetricsResponse  {
        MetricsResponse {
            message: String::from("Erroed out"),
            errored: true,
            metrics: String::from("")
        }
    }

    #[test]
    fn test_metrics_resp() {
        let response = test_route(build_metrics_response());
        assert_eq!(response.status(), Status::Ok);
      
    }

    #[test]
    fn test_errored_metrics_resp() {
        let response = test_route(build_erroed_metrics_response());
        assert_eq!(response.status(), Status::ServiceUnavailable);
      
    }
}