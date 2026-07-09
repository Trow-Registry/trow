use std::sync::Arc;

use axum::extract::State;

use crate::TrowServerState;
use crate::services::health_service::HealthStatus;

pub async fn healthz(State(state): State<Arc<TrowServerState>>) -> HealthStatus {
    state.services.health.healthz()
}
