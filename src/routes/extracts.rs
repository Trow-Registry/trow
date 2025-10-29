use std::sync::Arc;

use axum::RequestPartsExt;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum_extra::extract::Host;
use serde::Deserialize;

use crate::TrowServerState;

#[derive(Deserialize)]
pub struct ImageNamespace {
    pub ns: Option<String>,
}

pub struct AlwaysHost(pub String);

impl<S> FromRequestParts<S> for AlwaysHost
where
    Arc<TrowServerState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, ());

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = &Arc::from_ref(state).config;

        let maybe_host = parts.extract::<Result<Host, _>>().await.unwrap();
        if let Ok(Host(host)) = maybe_host {
            // Check if we have an upstream load balancer doing TLS termination
            let scheme = if let Some(proto) = parts.headers.get("X-Forwarded-Proto") {
                proto.to_str().unwrap_or("http")
            } else if config.uses_tls {
                "https"
            } else {
                "http"
            };
            if let (Some(forwarded_host_header), Some(forwarded_port_header)) = (
                parts.headers.get("X-Forwarded-Host"),
                parts.headers.get("X-Forwarded-Port"),
            ) {
                let forwarded_host = forwarded_host_header.to_str().unwrap_or("");
                let forwarded_port = forwarded_port_header.to_str().unwrap_or("");
                let res = format!("{scheme}://{forwarded_host}");
                if forwarded_port.is_empty() {
                    return Ok(AlwaysHost(res));
                } else {
                    return Ok(AlwaysHost(format!("{res}:{forwarded_port}")));
                }
            };

            return Ok(AlwaysHost(format!("{scheme}://{host}")));
        }

        Ok(AlwaysHost(String::new()))
    }
}
