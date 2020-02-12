use hostname;
use rocket::request::Request;
use crate::TrowConfig;

pub mod accepted_upload;
pub mod authenticate;
pub mod blob_reader;
pub mod blob_deleted;
pub mod manifest_deleted;
pub mod empty;
pub mod errors;
pub mod html;
pub mod manifest_reader;
pub mod repo_catalog;
pub mod tag_list;
mod test_helper;
pub mod trow_token;
pub mod upload_info;
pub mod verified_manifest;

/// Gets the base URL e.g. <http://registry:8000> using the HOST value from the request header.
/// Falls back to hostname if it doesn't exist.
///
/// Move this.
fn get_base_url(req: &Request) -> String {
    let host = get_domain_name(req);

    let config = req
        .guard::<rocket::State<TrowConfig>>()
        .expect("TrowConfig not present!");

    // Check if we have an upstream load balancer doing TLS termination
    match req.headers().get("X-Forwarded-Proto").next() {
        None => {
            match config.tls {
                None => format!("http://{}", host),
                Some(_) => format!("https://{}", host),
            }        
        }
        Some(proto) => {
            if proto == "http" {
                warn!("Security issue! Upstream proxy is using HTTP");
            }
            format!("{}://{}", proto, host)
        }
    }
}

fn get_domain_name(req: &Request) -> String {
    match req.headers().get("HOST").next() {
        None => {
            hostname::get_hostname().expect("Server has no name; cannot give clients my address")
        }
        Some(shost) => shost.to_string(),
    }
}
