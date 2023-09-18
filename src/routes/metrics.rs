use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;

use crate::registry_interface::{Metrics, MetricsResponse};
use crate::response::errors::Error;
use crate::TrowServerState;

/*
* Trow metrics endpoint
* GET /metrics
*/
pub async fn metrics(State(state): State<Arc<TrowServerState>>) -> Result<MetricsResponse, Error> {
    state
        .client
        .get_metrics()
        .await
        .map_err(|_| Error::InternalError)
}
