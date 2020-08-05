use crate::client_interface::ClientInterface;
use crate::types::ReadinessResponse;
use rocket::get;
use rocket::State;
use tokio::runtime::Runtime;

/*
* Trow readiness endpoint
* GET /readiness
*/

#[get("/readiness")]
pub fn readiness(ci: State<ClientInterface>) -> ReadinessResponse {
    let request = ci.is_ready();
    let mut rt = Runtime::new().unwrap();
    rt.block_on(request)
}
