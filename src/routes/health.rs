use crate::client_interface::ClientInterface;
use crate::types::HealthResponse;

use rocket::State;
use tokio::runtime::Runtime;

/*
* Trow health endpoint
* GET /healthz
*/

#[get("/healthz")]
pub fn healthz(
    ci: State<ClientInterface>,
) -> HealthResponse {
    let request = ci.is_healthy();
    let mut rt = Runtime::new().unwrap();
    
    rt.block_on(request)
}
