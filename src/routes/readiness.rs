use std::sync::Arc;

use axum::extract::State;

use crate::TrowServerState;
use crate::registry::api_types::ReadyStatus;

/*
* Trow readiness endpoint
* GET /readiness
*/
pub async fn readiness(State(_state): State<Arc<TrowServerState>>) -> ReadyStatus {
    ReadyStatus {
        message: "".to_string(),
        is_ready: true,
    }
}
