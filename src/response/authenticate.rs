use crate::response::get_base_url;
use rocket::http::ContentType;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

/*
 * Generate a WWW-Authenticate header
 */
#[derive(Debug, Serialize)]
pub struct Authenticate {}

impl<'r> Responder<'r, 'static> for Authenticate {
    fn respond_to(self, req: &Request) -> response::Result<'static> {
        let realm = get_base_url(req);
        let authenticate_header = Header::new(
            "www-authenticate",
            format!(
                "Bearer realm=\"{}/login\",service=\"trow_registry\",scope=\"push/pull\"",
                realm
            ),
        );
        Response::build()
            .status(Status::Unauthorized)
            .header(authenticate_header)
            .header(ContentType::JSON)
            .ok()
    }
}
