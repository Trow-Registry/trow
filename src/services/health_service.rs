use std::sync::Arc;

use serde_derive::{Deserialize, Serialize};

use crate::file_storage::FileStorage;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub message: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadyStatus {
    pub is_ready: bool,
    pub message: String,
}

#[derive(Debug)]
pub struct HealthService {
    storage: Arc<FileStorage>,
}

impl HealthService {
    pub fn new(storage: Arc<FileStorage>) -> Self {
        Self { storage }
    }

    pub fn healthz(&self) -> HealthStatus {
        HealthStatus {
            message: String::new(),
            is_healthy: true,
        }
    }

    pub async fn readiness(&self) -> ReadyStatus {
        match self.storage.is_ready().await {
            Ok(()) => ReadyStatus {
                message: String::new(),
                is_ready: true,
            },
            Err(e) => ReadyStatus {
                message: e.to_string(),
                is_ready: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::file_storage::FileStorage;
    use crate::services::health_service::HealthService;
    use crate::test_utilities::test_temp_dir;

    #[test]
    fn healthz_returns_healthy() {
        let dir = test_temp_dir!();
        let storage = Arc::new(FileStorage::new(dir.as_path_untracked().to_owned()).unwrap());
        let svc = HealthService::new(storage);
        let status = svc.healthz();
        assert!(status.is_healthy);
        assert_eq!(status.message, "");
    }

    #[tokio::test]
    async fn readiness_returns_ready() {
        let dir = test_temp_dir!();
        let storage = Arc::new(FileStorage::new(dir.as_path_untracked().to_owned()).unwrap());
        let svc = HealthService::new(storage);
        let status = svc.readiness().await;
        assert!(status.is_ready);
        assert_eq!(status.message, "");
    }
}
