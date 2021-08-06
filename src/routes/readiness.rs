use crate::client_interface::ClientInterface;
use crate::registry_interface::Metrics;
use crate::types::ReadinessResponse;
use rocket::get;
use rocket::State;

/*
* Trow readiness endpoint
* GET /readiness
*/

#[get("/readiness")]
pub fn readiness(ci: &State<ClientInterface>) -> ReadinessResponse {
    ReadinessResponse {
        message: "".to_string(),
        is_ready: ci.is_ready(),
    }
}
