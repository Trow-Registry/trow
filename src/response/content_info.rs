use crate::registry_interface::blob_storage::ContentInfo;
use crate::response::errors::Error;
use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};

/**
 * ContentInfo should always be wrapped an Option in routes to avoid failure returns.
 */
#[rocket::async_trait]
impl<'r> FromRequest<'r> for ContentInfo {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Error> {
        let length = match request.headers().get_one("Content-Length") {
            Some(l) => match l.parse::<u64>() {
                Ok(i) => i,
                Err(_) => {
                    warn!("Received request with invalid Content-Length header");
                    return Outcome::Failure((
                        Status::BadRequest,
                        Error::BlobUploadInvalid("Invalid Content-Length".to_string()),
                    ));
                }
            },
            None => {
                // This probably just means we don't have ContentInfo
                // Should be caught by an option in the RequestGuard
                return Outcome::Failure((
                    Status::BadRequest,
                    Error::BlobUploadInvalid("Expected Content-Length header".to_string()),
                ));
            }
        };

        if let Some(r) = request.headers().get_one("Content-Range") {
            let parts: Vec<&str> = r.split('-').collect();
            if parts.len() == 2 {
                if let Ok(l) = parts[0].parse::<u64>() {
                    if let Ok(r) = parts[1].parse::<u64>() {
                        return Outcome::Success(ContentInfo {
                            length,
                            range: (l, r),
                        });
                    }
                }
            }
        }
        warn!("Received request with invalid Content-Range header");
        Outcome::Failure((
            rocket::http::Status::BadRequest,
            Error::BlobUploadInvalid("Invalid Content-Range".to_string()),
        ))
    }
}
