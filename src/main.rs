#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;

use rocket_contrib::{Json};
use rocket::http::{Status};
use rocket::response::{Response, Responder};
use rocket::request::Request;

#[derive(Serialize)]
struct V2AvailableRoutes {
}

#[derive(Debug)]
struct RegistryResponse<R>(pub R);

impl <'r, R: Responder<'r>> Responder<'r> for RegistryResponse<R> {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let header = rocket::http::Header::new("Docker-Distribution-API-Version", "registry/2.0");
        let mut build = Response::build();

        {
        let _x = self.0.respond_to(req)?;
        // return _x;
        build
            .header(header)
            .merge(_x)
            .ok()
        }
    }
}

type LycaonResponse<A> = RegistryResponse<Json<A>>;

#[get("/v2")]
fn v2root() -> LycaonResponse<V2AvailableRoutes> {
    RegistryResponse::<Json<V2AvailableRoutes>> ((Json(V2AvailableRoutes {})))
}

fn main () {
    rocket::ignite().mount("/", routes![v2root]).launch();
}
