use std::sync::Arc;

use axum::extract::State;

use crate::registry_interface::metrics::Metrics;
use crate::types::HealthResponse;
use crate::TrowServerState;

/*
* Trow health endpoint
* GET /healthz
*/

pub async fn healthz(State(state): State<Arc<TrowServerState>>) -> HealthResponse {
    HealthResponse {
        message: "".to_string(),
        is_healthy: state.client.is_healthy().await,
    }
}
