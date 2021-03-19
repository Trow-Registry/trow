use crate::client_interface::ClientInterface;
use crate::types::MetricsResponse;

use crate::registry_interface::Metrics;
use crate::response::errors::Error;

use rocket::State;

/*
* Trow metrics endpoint
* GET /metrics
*/

#[get("/metrics")]
pub fn metrics(ci: State<ClientInterface>) -> Result<MetricsResponse, Error> {
    ci.get_metrics().map_err(|_| Error::InternalError)
}
