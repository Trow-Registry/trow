#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;

use rocket_contrib::Json;
use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use rocket::response::status::NotFound;

mod errors;

#[derive(Serialize, Debug)]
struct V2AvailableRoutes {}

#[derive(Debug)]
struct RegistryResponse<R>(pub R);

impl<'r, R: Responder<'r>> Responder<'r> for RegistryResponse<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let header = rocket::http::Header::new("Docker-Distribution-API-Version", "registry/2.0");
        Response::build()
            .header(header)
            .merge(self.0.respond_to(req)?)
            .ok()
    }
}

type LycaonResponse<A> = RegistryResponse<Json<A>>;
type MaybeResponse<A, E> = Result<Json<A>, NotFound<Json<E>>>;


#[get("/v2")]
fn get_v2root() -> LycaonResponse<V2AvailableRoutes> {
    RegistryResponse::<Json<V2AvailableRoutes>>((Json(V2AvailableRoutes {})))
}

#[get("/v2/<_name>/manifests/<reference>")]
fn get_manifest(
    _name: String,
    reference: String,
) -> MaybeResponse<V2AvailableRoutes, errors::Errors> {
    println!("Getting Manifest");
    let errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    match reference.as_str() {
        "good" => Ok(Json(V2AvailableRoutes {})),
        _ => Err(NotFound(Json(errors))),
    }
}

#[get("/v2/<_name>/blobs/<digest>")]
fn get_blob(_name: String, digest: String) -> MaybeResponse<V2AvailableRoutes, V2AvailableRoutes> {
    println!("Getting Blob");
    match digest.as_str() {
        "good" => Ok(Json(V2AvailableRoutes {})),
        _ => Err(NotFound(Json(V2AvailableRoutes {}))),
    }
}

fn main() {
    rocket::ignite()
        .mount("/", routes![get_v2root, get_manifest, get_blob])
        .launch();
}
