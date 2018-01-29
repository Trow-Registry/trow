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
pub mod errors;
mod test_helper;


/// take in a request and a struct to be serialised.
/// Return a response with the Json attached.
///
/// If one wants to continue modifying the response after attaching Json
///
/// ```
/// use rocket::http::Header;
/// let header = Header::new("Header", "Pizza");
/// Response::build_from(json_response(req, &repositories).unwrap_or_default())
///   .header(header)
///   .ok()
/// ```

pub fn json_response<T: serde::Serialize>(
    req: &Request,
    var: &T,
) -> Result<Response<'static>, Status> {
    use rocket_contrib;
    rocket_contrib::Json(var).respond_to(req)
}
