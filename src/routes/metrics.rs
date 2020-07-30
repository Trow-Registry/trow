use crate::client_interface::ClientInterface;
use crate::types::MetricsResponse;

use crate::response::errors::Error;

use rocket::State;
use tokio::runtime::Runtime;

/*
* Trow metrics endpoint
* GET /metrics
*/

#[get("/metrics")]
pub fn metrics(
    ci: State<ClientInterface>,
) -> Result<MetricsResponse, Error> {
    let request = ci.get_metrics();
    let mut rt = Runtime::new().unwrap();
    
    match rt.block_on(request) {
        Ok(metrics) => {
            Ok(metrics)
        }
        Err(_) => {
            Err(Error::InternalError)
        }
    }

}
