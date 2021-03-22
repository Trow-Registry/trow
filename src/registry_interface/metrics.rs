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

pub trait Metrics {
    fn is_healthy(&self) -> bool;
    fn is_ready(&self) -> bool;
    fn get_metrics(&self) -> Result<MetricsResponse, MetricsError>;
}
