use std::sync::Arc;

use axum::extract::State;

use crate::registry_interface::Metrics;
use crate::types::ReadinessResponse;
use crate::TrowServerState;

/*
* Trow readiness endpoint
* GET /readiness
*/
pub async fn readiness(State(state): State<Arc<TrowServerState>>) -> ReadinessResponse {
    ReadinessResponse {
        message: "".to_string(),
        is_ready: state.client.is_ready().await,
    }
}
