use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Internal metrics error")]
    Internal,
}

/*
Could just use a string here, but later on we probably want more structure.
*/
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricsResponse {
    pub metrics: String,
}

#[axum::async_trait]
pub trait Metrics {
    async fn is_healthy(&self) -> bool;
    async fn is_ready(&self) -> bool;
    async fn get_metrics(&self) -> Result<MetricsResponse, MetricsError>;
}
