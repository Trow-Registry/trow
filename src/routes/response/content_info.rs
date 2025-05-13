use axum::extract::{FromRequestParts, OptionalFromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;

use crate::registry::api_types::ContentInfo;
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
        let err = |msg: &str, optional: bool| {
            if optional {
                Err((StatusCode::BAD_REQUEST, Error::NotFound))
            } else {
                Err((
                    StatusCode::BAD_REQUEST,
                    Error::BlobUploadInvalid(msg.to_string()),
                ))
            }
        };

        let length = match parts.headers.get("Content-Length") {
            Some(l) => match l.to_str().map(|s| s.parse::<u64>()) {
                Ok(Ok(i)) => i,
                _ => return err("Invalid Content-Length", false),
            },
            None => return err("Expected Content-Length header", true),
        };

        let range = match parts.headers.get("Content-Range") {
            Some(r) => match r.to_str().map(|head| {
                head.split_once('-')
                    .map(|(start, end)| (start.parse(), end.parse()))
            }) {
                Ok(Some((Ok(start), Ok(end)))) => (start, end),
                _ => return err("Invalid Content-Range", false),
            },
            None => return err("Expected Content-Range header", true),
        };
        if length != range.1 - range.0 + 1 {
            return err("Content-Length and Content-Range don't match", false);
        }

        Ok(Self { length, range })
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
        match <Self as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(ci) => Ok(Some(ci)),
            Err((_, Error::NotFound)) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
