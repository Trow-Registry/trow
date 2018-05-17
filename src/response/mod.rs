use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use serde;

pub mod empty;
pub mod errors;
pub mod html;
pub mod manifest_upload;
mod test_helper;
pub mod upload_info;
pub mod uuidaccept;

pub fn json_response<T: serde::Serialize>(
    req: &Request,
    var: &T,
) -> Result<Response<'static>, Status> {
    use rocket_contrib;
    rocket_contrib::Json(var).respond_to(req)
}
