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
use rocket::response::Responder;

#[cfg(test)]
pub fn test_route<'r, A: Responder<'r, 'static>>(handler: A) -> rocket::Response<'static> {
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
        hub_user: None,
        hub_pass: None,
        host_names: vec![],
        allow_prefixes: vec![],
        allow_images: vec![],
        deny_prefixes: vec![],
        deny_images: vec![],
        dry_run: false,
        token_secret: "secret".to_string(),
        user: None,
        cors: false,
    };
    let rocket = rocket::Rocket::build().manage(trow_config);
    let client = Client::tracked(rocket).expect("valid rocket instance");
    let request = client.get("/");
    let request = request.inner();

    handler.respond_to(&request).unwrap()
}
