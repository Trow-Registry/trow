use std::sync::Arc;

use axum::extract::State;

use crate::registry::api_types::HealthStatus;
use crate::TrowServerState;
/*
* Trow health endpoint
* GET /healthz
*/

pub async fn healthz(State(state): State<Arc<TrowServerState>>) -> HealthStatus {
    HealthStatus {
        message: "".to_string(),
        is_healthy: state.registry.is_healthy().await,
    }
}
