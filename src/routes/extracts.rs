use axum::extract::{FromRef, FromRequestParts, Host};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::RequestPartsExt;

use crate::TrowConfig;

pub struct AlwaysHost(pub String);

#[axum::async_trait]
impl<S> FromRequestParts<S> for AlwaysHost
where
    TrowConfig: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, ());

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = TrowConfig::from_ref(state);

        let maybe_host = parts.extract::<Option<Host>>().await.unwrap();
        if let Some(Host(host)) = maybe_host {
            // Check if we have an upstream load balancer doing TLS termination
            let scheme = if let Some(proto) = parts.headers.get("X-Forwarded-Proto") {
                proto.to_str().unwrap_or("http")
            } else {
                if config.uses_tls {
                    "https"
                } else {
                    "http"
                }
            };

            return Ok(AlwaysHost(format!("{}://{}", scheme, host)));
        }

        Ok(AlwaysHost(String::new()))
    }
}
