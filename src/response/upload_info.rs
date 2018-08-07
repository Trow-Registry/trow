use failure::Error;
use response::{errors,get_base_url};
use rocket::State;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};

use backend as bh;
use grpc::backend;
use types::Layer;

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: String,
    repo_name: String,
    range: (u32, u32),
}

pub fn create_upload_info(
    uuid: String,
    repo_name: String,
    range: (u32, u32),
) -> UploadInfo {
    UploadInfo {
        uuid,
        repo_name,
        range,
    }
}

impl UploadInfo {
    //TODO: Move this
    pub fn uuid_exists(handler: State<bh::ClientInterface>, layer: &Layer) -> Result<bool, Error> {
        let backend = handler.backend();
        let mut req = backend::Layer::new();
        req.set_repo_name(layer.repo_name.to_owned());
        req.set_digest(layer.digest.to_owned());

        let response = backend.uuid_exists(&req)?;
        debug!("UuidExists: {:?}", response.get_success());
        if response.get_success() {
            Ok(true)
        } else {
            Err(errors::Error::DigestInvalid.into())
        }
    }
}


impl<'r> Responder<'r> for UploadInfo {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        match self {
            UploadInfo {
                ref uuid,
                ref repo_name,
                ref range,
            } => {
                debug!("Uuid Ok");
                let location_url = format!(
                    "{}/v2/{}/blobs/uploads/{}",
                    get_base_url(req),
                    repo_name,
                    uuid
                );
                let &(left, right) = range;
                let upload_uuid = Header::new("Docker-Upload-UUID", uuid.clone());
                let range = Header::new("Range", format!("{}-{}", left, right));
                let length = Header::new("X-Content-Length", format!("{}", right - left));
                let location = Header::new("Location", location_url);

                debug!("Range: {}-{}, Length: {}", left, right, right - left);
                Response::build()
                    .header(upload_uuid)
                    .header(location)
                    .header(range)
                    .header(length)
                    // TODO: move into the type so it is better encoded?...
                    .status(Status::Accepted)
                    .ok()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use response::upload_info::UploadInfo;
    use rocket::http::Status;

    use response::test_helper::test_route;
    fn build_response() -> UploadInfo {
        UploadInfo {
            uuid: String::from("whatever"),
            repo_name: String::from("moredhel/test"),
            range: (0, 0),
        }
    }

    #[test]
    fn uuid_uuid() {
        let response = test_route(build_response());
        let headers = response.headers();
        assert_eq!(response.status(), Status::Accepted);
        assert!(headers.contains("Docker-Upload-UUID"));
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
    }

}
