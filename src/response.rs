use rocket;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use errors;

const BASE_URL: &str = "http://localhost:8000";

// pub type Response<T: MyResponder>(T);
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
pub struct RegistryResponse<R>(pub R);

impl<'r, R: Responder<'r>> Responder<'r> for RegistryResponse<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        debug!("In the Registry Response");
        let header = rocket::http::Header::new("Docker-Distribution-API-Version", "registry/2.0");
        let sub_response = self.0.respond_to(req)?;
        debug!("{:?}", sub_response);
        debug!("Exit Registry Response");
        Response::build()
            .header(header)
            .merge(sub_response)
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

pub type MaybeResponse<A> = RegistryResponse<Result<A, errors::Error>>;

impl<A> MaybeResponse<A>  {
    pub fn ok(val: A) -> Self {
        RegistryResponse(Ok(val))
    }

    pub fn err(error: errors::Error) -> Self {
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
