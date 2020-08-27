use rocket::request::{self, Request, FromRequest};
use rocket::State;
// use tracing::{Level, event, instrument};
use uuid::Uuid;
use failure::Error;

#[derive(Debug, Clone)]
pub struct TrowInstance {
    pub id: String
}

impl TrowInstance {
    pub fn new() -> Result<Self, Error> {
        Ok(TrowInstance{
            id: format!("{}-{}", "trow", Uuid::new_v4().to_string())
        })
    }
}

#[derive(Debug)]
pub struct TrowRequest {
    pub request_id: String,
    pub trow_instance: TrowInstance 
}

impl TrowRequest {
    pub fn new(trow_instance: TrowInstance) -> Result<Self, Error> {
        Ok(TrowRequest { 
            request_id:  Uuid::new_v4().to_string(),
            trow_instance:  trow_instance
        })
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a TrowRequest {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<&'a TrowRequest, ()> {
        let trow_instance_state = request.guard::<State<TrowInstance>>()?;

        request::Outcome::Success(request.local_cache(|| {
            TrowRequest::new(trow_instance_state.clone()).unwrap()
        }))
    }
}
