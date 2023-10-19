use std::sync::Arc;

use axum::extract::State;

use crate::registry_interface::metrics::Metrics;
use crate::TrowServerState;
use crate::trow_server::api_types::HealthStatus;
/*
* Trow health endpoint
* GET /healthz
*/

pub async fn healthz(State(state): State<Arc<TrowServerState>>) -> HealthStatus {
    HealthStatus {
        message: "".to_string(),
        is_healthy: state.client.is_healthy().await,
    }
}
