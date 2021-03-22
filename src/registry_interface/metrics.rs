use crate::types::MetricsResponse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Internal metrics error")]
    Internal,
}

pub trait Metrics {
    fn is_healthy(&self) -> bool;
    fn is_ready(&self) -> bool;
    fn get_metrics(&self) -> Result<MetricsResponse, MetricsError>;
}
