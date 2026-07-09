use std::sync::Arc;

use axum::extract::State;

use crate::TrowServerState;
use crate::services::health_service::ReadyStatus;

pub async fn readiness(State(state): State<Arc<TrowServerState>>) -> ReadyStatus {
    state.services.health.readiness().await
}
