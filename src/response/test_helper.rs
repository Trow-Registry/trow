#[cfg(test)]
use crate::GrpcConfig;
#[cfg(test)]
use crate::NetAddr;
#[cfg(test)]
use crate::TrowConfig;
#[cfg(test)]
#[cfg(test)]
use rocket::local::blocking::Client;

#[cfg(test)]
pub fn test_client() -> Client {
    let trow_config = TrowConfig {
        data_dir: "".to_string(),
        addr: NetAddr {
            host: "trow".to_string(),
            port: 51000,
        },
        tls: None,
        grpc: GrpcConfig {
            listen: "trow:51000".to_owned(),
        },
        proxy_hub: true,
        proxy_registry_config_dir: "./proxy-registry-config".to_string(),
        hub_user: None,
        hub_pass: None,
        host_names: vec![],
        allow_prefixes: vec![],
        allow_images: vec![],
        deny_prefixes: vec![],
        deny_images: vec![],
        dry_run: false,
        max_manifest_size: 1,
        max_blob_size: 100,
        token_secret: "secret".to_string(),
        user: None,
        cors: false,
        log_level: "error".to_string(),
    };
    let rocket = rocket::Rocket::build()
        .manage(trow_config)
        .mount("/", vec![]);
    Client::tracked(rocket).expect("valid rocket instance")
}
