use axum::extract::{ FromRequestParts, Host};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;

pub struct AlwaysHost (pub String);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AlwaysHost
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, ());

    async fn from_request_parts(req: &mut Parts, _config: &S) -> Result<Self, Self::Rejection> {
        let maybe_host = req.extract::<Option<Host>>().await.unwrap();
        let host = match maybe_host {
            Some(Host(host)) => AlwaysHost(host),
            None => AlwaysHost(String::new())
        };
        Ok(host)
    }
}
