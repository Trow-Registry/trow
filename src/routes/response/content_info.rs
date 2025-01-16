use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use tracing::{event, Level};

use crate::registry::blob_storage::ContentInfo;
use crate::routes::response::errors::Error;

/**
 * ContentInfo should always be wrapped an Option in routes to avoid failure returns.
 */
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
                    event!(
                        Level::WARN,
                        "Received request with invalid Content-Length header"
                    );
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
        event!(
            Level::WARN,
            "Received request with invalid Content-Range header"
        );
        Err((
            StatusCode::BAD_REQUEST,
            Error::BlobUploadInvalid("Invalid Content-Range".to_string()),
        ))
    }
}

impl<S> OptionalFromRequestParts<S> for ContentInfo
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Error);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Option<Self>, Self::Rejection> {
        // TODO: better handle this
        Ok(
            <Self as FromRequestParts<S>>::from_request_parts(parts, state)
                .await
                .ok(),
        )
    }
}
