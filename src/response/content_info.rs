use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use log::warn;

use crate::registry_interface::blob_storage::ContentInfo;
use crate::response::errors::Error;

/**
 * ContentInfo should always be wrapped an Option in routes to avoid failure returns.
 */
#[axum::async_trait]
impl<S> FromRequestParts<S> for ContentInfo
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Error);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let length = match parts.headers.get("Content-Length") {
            Some(l) => match l.to_str().map(|s| s.parse::<u64>()) {
                Ok(Ok(i)) => i,
                _ => {
                    warn!("Received request with invalid Content-Length header");
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Error::BlobUploadInvalid("Invalid Content-Length".to_string()),
                    ));
                }
            },
            None => {
                // This probably just means we don't have ContentInfo
                // Should be caught by an option in the RequestGuard
                return Err((
                    StatusCode::BAD_REQUEST,
                    Error::BlobUploadInvalid("Expected Content-Length header".to_string()),
                ));
            }
        };

        if let Some(r) = parts.headers.get("Content-Range") {
            if let Ok(range) = r.to_str() {
                let parts: Vec<&str> = range.split('-').collect();
                if parts.len() == 2 {
                    if let Ok(l) = parts[0].parse::<u64>() {
                        if let Ok(r) = parts[1].parse::<u64>() {
                            return Ok(ContentInfo {
                                length,
                                range: (l, r),
                            });
                        }
                    }
                }
            }
        }
        warn!("Received request with invalid Content-Range header");
        Err((
            StatusCode::BAD_REQUEST,
            Error::BlobUploadInvalid("Invalid Content-Range".to_string()),
        ))
    }
}
