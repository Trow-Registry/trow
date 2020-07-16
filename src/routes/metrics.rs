use crate::client_interface::ClientInterface;
use crate::types::MetricsResponse;

use rocket::State;
use tokio::runtime::Runtime;

/*
* Trow metrics endpoint
* GET /metrics
*/

#[get("/metrics")]
pub fn metrics(
    ci: State<ClientInterface>,
) -> MetricsResponse {
    let request = ci.get_metrics();
    let mut rt = Runtime::new().unwrap();
    
    rt.block_on(request)

}
