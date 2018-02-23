use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use serde;

/// Exporting all routes for the project
pub mod admin;
pub mod catalog;
pub mod empty;
pub mod html;
pub mod layers;
pub mod uuid;
pub mod uuidaccept;
pub mod manifest_upload;
pub mod errors;
mod test_helper;

pub fn json_response<T: serde::Serialize>(
    req: &Request,
    var: &T,
) -> Result<Response<'static>, Status> {
    use rocket_contrib;
    rocket_contrib::Json(var).respond_to(req)
}
