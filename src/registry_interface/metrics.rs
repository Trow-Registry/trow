use thiserror::Error;
use crate::trow_server::api_types::MetricsResponse;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Internal metrics error")]
    Internal,
}

#[axum::async_trait]
pub trait Metrics {
    async fn is_healthy(&self) -> bool;
    async fn is_ready(&self) -> bool;
    async fn get_metrics(&self) -> Result<MetricsResponse, MetricsError>;
}
