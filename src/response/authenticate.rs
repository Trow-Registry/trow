use rocket::http::ContentType;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};

/*
 * Generate a WWW-Authenticate header
 */
#[derive(Debug, Serialize)]
pub struct Authenticate {
    pub username: String,
}

impl<'r> Responder<'r> for Authenticate {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        //TODO: Change hard-coded host
        let authenticate_header = Header::new("www-authenticate","Bearer realm=\"https://0.0.0.0:8443/login\",service=\"trow_registry\",scope=\"push/pull\"");
        Response::build()
            .status(Status::Unauthorized)
            .header(authenticate_header)
            .header(ContentType::JSON)
            .ok()
    }
}
