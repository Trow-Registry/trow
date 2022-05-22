use crate::client_interface::ClientInterface;

use crate::registry_interface::{Metrics, MetricsResponse};
use crate::response::errors::Error;

use anyhow::Result;
use rocket::get;
use rocket::State;

/*
* Trow metrics endpoint
* GET /metrics
*/

#[get("/metrics")]
pub async fn metrics(ci: &State<ClientInterface>) -> Result<MetricsResponse, Error> {
    ci.get_metrics().await.map_err(|_| Error::InternalError)
}
