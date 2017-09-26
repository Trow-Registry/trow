use rocket;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket_contrib::Json;
use serde_json;

use errors;

// TODO refactor this out in place of dynamic updates
const BASE_URL: &str = "http://localhost:8000";

// one possible solution
pub type MaybeResponse2<A: MyTrait> = RegistryResponse<A>;

impl<A> MaybeResponse2<A>  {
    pub fn ok2(val: A) -> Self
    where A: MyTrait{
        // RegistryResponse(Ok(A(Json)))
        RegistryResponse(Ok(val))
    }

    pub fn err2(val: A) -> Self
    where A: MyTrait {
        RegistryResponse(Err(val))
    }
}

pub trait MyTrait {}


impl MyTrait for Empty {}

#[derive(Debug)]
pub struct Empty;

impl<'r> Responder<'r> for Empty {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .ok()
    }
}

#[derive(Debug, Serialize)]
pub enum UuidResponse {
    Uuid {
        uuid: String,
        name: String,
        repo: String,
        left: u32,
        right: u32,
    },
    Empty,
}
impl MyTrait for UuidResponse {}

impl<'r> Responder<'r> for UuidResponse {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        {
            debug!("{:?}", &self);
            debug!("{}", serde_json::to_string(&self).unwrap());
        }

        if let UuidResponse::Uuid {uuid, name, repo, left, right} = self {
            let location_url = format!("{}/v2/{}/{}/blobs/uploads/{}?query=true", BASE_URL, name, repo, uuid);
            let upload_uuid = Header::new("Docker-Upload-UUID", uuid);
            let range = Header::new("Range", format!("{}-{}", left, right));
            debug!("Range: {}-{}, Length: {}", left, right, right-left);
            let length = Header::new("Content-Length", format!("{}", right - left));
            let location = Header::new("Location", location_url);
            Response::build()
                .header(upload_uuid)
                .header(location)
                .header(range)
                .header(length)
                // TODO: move into the type so it is better encoded?...
                .status(Status::Accepted)
                .ok()
        } else {
            Response::build()
                .status(Status::NotFound)
                .ok()
        }
    }
}


/// An enum of possible responses
#[derive(Serialize, Debug)]
pub enum Responses {
    Accept,
    Empty,
    Uuid {
        uuid: String,
        name: String,
        repo: String,
        left: u32,
        right: u32,
    },
    UuidAccept,
}

impl<'r> Responder<'r> for Responses {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        match self {
            Responses::Accept => Response::build().status(Status::Accepted).ok(),
            Responses::Empty => Response::build().ok(),
            Responses::Uuid {uuid, name, repo, left, right} => {
                let location_url = format!("{}/v2/{}/{}/blobs/uploads/{}?query=true", BASE_URL, name, repo, uuid);
                let upload_uuid = Header::new("Docker-Upload-UUID", uuid);
                let range = Header::new("Range", format!("{}-{}", left, right));
                debug!("Range: {}-{}, Length: {}", left, right, right-left);
                let length = Header::new("Content-Length", format!("{}", right - left));
                let location = Header::new("Location", location_url);
                Response::build()
                    .header(upload_uuid)
                    .header(location)
                    .header(range)
                    .header(length)
                    // TODO: move into the type so it is better encoded?...
                    .status(Status::Accepted)
                    .ok()
            },
            Responses::UuidAccept => {
                Response::build()
                    .status(Status::Created)
                    .ok()
            }
        }
    }
}

#[derive(Debug)]
pub struct RegistryResponse<R>(pub Result<R, R>);

impl<'r, R: Responder<'r>> Responder<'r> for RegistryResponse<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        match self.0 {
            Ok(r) => info!("all going according to plan!"),
            Err(_) => info!("Something happened"),
        };
        debug!("In the Registry Response");
        let header = rocket::http::Header::new("Docker-Distribution-API-Version", "registry/2.0");
        // let sub_response = self.0.respond_to(req)?;
        // debug!("{:?}", sub_response);
        debug!("Exit Registry Response");
        Response::build()
            .header(header)
            // .merge(sub_response)
            .ok()
    }
}

impl<'r> Responder<'r> for errors::Error {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        // TODO add a body to the response
        Response::build()
            .status(Status::NotFound)
            .ok()
    }
}

pub type MaybeResponse<A> = RegistryResponse<A>;

impl<A> MaybeResponse<A>  {
    pub fn ok(val: A) -> Self {
        RegistryResponse(Ok(val))
    }

    pub fn err(error: A) -> Self {
        RegistryResponse(Err(error))
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }

    #[test]
    fn panicify() {
        panic!("something happened");
    }
}
