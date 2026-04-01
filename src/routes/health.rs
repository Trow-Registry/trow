use std::sync::Arc;

use axum::extract::State;

use crate::TrowServerState;
use crate::registry::api_types::HealthStatus;
/*
* Trow health endpoint
* GET /healthz
*/

pub async fn healthz(State(_state): State<Arc<TrowServerState>>) -> HealthStatus {
    HealthStatus {
        message: "".to_string(),
        is_healthy: true,
    }
}
