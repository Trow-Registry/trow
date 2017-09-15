use rocket;
use rocket_contrib::Json;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket::response::status::NotFound;

use errors;

#[derive(Serialize, Debug)]
pub enum Responses {
    Empty {},
    Uuid {
        uuid: String,
        name: String,
        repo: String
    },

}

impl<'r> Responder<'r> for Responses {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        match self {
            Responses::Empty {} => Response::build().ok(),
            Responses::Uuid {uuid, name, repo} => {
                let location_url = format!("/v2/{}/{}/blobs/uploads/{}", name, repo, uuid);
                let header = Header::new("Docker-Upload-UUID", uuid);
                let location = Header::new("Location", location_url);
                Response::build()
                    .header(header)
                    .header(location)
                    .ok()
            },
        }
    }
}

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

pub type MaybeResponse<A> = RegistryResponse<Result<A, NotFound<errors::Error>>>;

impl<A> MaybeResponse<A>  {
    pub fn ok(val: A) -> Self {
        RegistryResponse(Ok(val))
    }

    pub fn err(error: errors::Error) -> Self {
        RegistryResponse(Err(NotFound(error)))
    }
}
