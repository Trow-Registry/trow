use axum::http::header::HeaderMap;
use tracing::{event, Level};

use crate::TrowConfig;

pub mod accepted_upload;
pub mod authenticate;
pub mod blob_deleted;
pub mod blob_reader;
pub mod content_info;
pub mod empty;
pub mod errors;
pub mod health;
pub mod html;
pub mod manifest_deleted;
pub mod manifest_history;
pub mod manifest_reader;
pub mod metrics;
pub mod readiness;
pub mod repo_catalog;
pub mod tag_list;
pub mod trow_token;
pub mod upload;
pub mod upload_info;
pub mod verified_manifest;

/// Gets the base URL e.g. <http://registry:8000> using the HOST value from the request header.
/// Falls back to hostname if it doesn't exist.
///
/// Move this.
pub fn get_base_url(headers: &HeaderMap, config: &TrowConfig) -> String {
    let host = headers
        .get("Host")
        .expect("No host header")
        .to_str()
        .expect("invalid host header");

    // Check if we have an upstream load balancer doing TLS termination
    match headers.get("X-Forwarded-Proto").map(|h| h.to_str()) {
        Some(Ok(proto)) => {
            if proto == "http" {
                event!(Level::WARN, "Security issue! Upstream proxy is using HTTP");
            }
            format!("{}://{}", proto, host)
        }
        _ => match config.tls {
            None => format!("http://{}", host),
            Some(_) => format!("https://{}", host),
        },
    }
}
