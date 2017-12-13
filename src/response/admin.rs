//! Administrative functions
use failure::Error;
use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use grpc::backend;
use config;

use response::json_response;

#[derive(Debug, Serialize)]
pub enum Admin {
    Uuids(Vec<String>),
}


impl Admin {
    pub fn get_uuids(handler: State<config::BackendHandler>) -> Result<Admin, Error> {
        let backend = handler.backend();
        let response = backend.get_uuids(backend::Empty::new())?;

        use std::iter::FromIterator;

        let uuids = response.get_uuids().iter().map(|wrapper| wrapper.get_uuid().to_owned()).collect::<Vec<String>>();
        debug!("Uuids: {:?}", uuids);
        Ok(Admin::Uuids(uuids))
    }
}

impl<'r> Responder<'r> for Admin {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        json_response(req, &self)
    }
}
