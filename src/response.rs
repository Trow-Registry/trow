use rocket;
use rocket_contrib::Json;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket::response::status::NotFound;

use errors;

#[derive(Serialize, Debug)]
pub struct EmptyResponse {}

#[derive(Debug)]
pub struct RegistryResponse<R>(pub R);

impl<'r, R: Responder<'r>> Responder<'r> for RegistryResponse<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        debug!("In the Registry Response");
        let header = rocket::http::Header::new("Docker-Distribution-API-Version", "registry/2.0");
        Response::build()
            .header(header)
            .merge(self.0.respond_to(req)?)
            .ok()
    }
}

/**
This is the response for a valid Uuid Generation.
*/
#[derive(Debug, Serialize)]
pub struct UuidResponse {
    pub uuid: String
}

impl<'r> Responder<'r> for UuidResponse {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        let header = rocket::http::Header::new("Docker-Upload-UUID", self.uuid);
        Response::build()
            .header(header)
            .ok()
    }
}

pub type LycaonResponse<A> = RegistryResponse<Json<A>>;
pub type MaybeResponse<A> = RegistryResponse<Result<Json<A>, NotFound<Json<errors::Error>>>>;


impl<A> MaybeResponse<A>  {
    pub fn ok(val: A) -> Self {
        RegistryResponse(Ok((Json(val))))
    }

    pub fn err(error: errors::Error) -> Self {
        RegistryResponse(Err(NotFound(Json(error))))
    }
}
