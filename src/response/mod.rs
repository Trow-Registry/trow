use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use serde;

use errors;

/// Exporting all routes for the project
pub mod admin;
pub mod catalog;
pub mod empty;
pub mod html;
pub mod layers;
pub mod uuid;
pub mod uuidaccept;
mod test_helper;

/// Encapsulate a response from the registry
/// Currently the full type definition is not possible (26 Sept 2017),
/// The hope is that we can restrict the following to Responder<'r> in
/// the future.
///
/// We pass in a Struct/Enum into the type-constructor, and as
/// long as we have implemented the Responder<'r> trait, then
/// everything just works.
///
/// @Deprecated, move everything directly onto the RegistryResponse
pub type MaybeResponse<A> = RegistryResponse<A>;

/// Testing new MaybeResponse
pub type MaybeResponse2<A> = RegistryResponse<Result<A, errors::Client>>;

/// Two constructors to ease sending a success/fail response.
impl<'r, A: Responder<'r>> MaybeResponse<A> {
    pub fn build(val: A) -> Self
    where
        A: Responder<'r>,
    {
        RegistryResponse(val)
    }

    pub fn ok(val: A) -> Self
    where
        A: Responder<'r>,
    {
        warn!("Deprecated, please use build");
        RegistryResponse(val)
    }

    pub fn err(val: A) -> Self
    where
        A: Responder<'r>,
    {
        warn!("Deprecated, please use build");
        RegistryResponse(val)
    }
}

/// Performs runtime dispatch to return call success or failure.
/// TODO: add lifetime and change to restrict to Responder
#[derive(Debug)]
pub struct RegistryResponse<R>(pub R);

impl<'r, R> Responder<'r> for RegistryResponse<R>
where
    R: Responder<'r>,
{
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let response = self.0.respond_to(req)?;
        Ok(response)
    }
}

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
