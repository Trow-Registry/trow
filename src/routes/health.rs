use crate::client_interface::ClientInterface;
use crate::registry_interface::metrics::Metrics;
use crate::types::HealthResponse;

use rocket::State;

/*
* Trow health endpoint
* GET /healthz
*/

#[get("/healthz")]
pub fn healthz(ci: State<ClientInterface>) -> HealthResponse {
    HealthResponse {
        message: "".to_string(),
        is_healthy: ci.is_healthy(),
    }
}
