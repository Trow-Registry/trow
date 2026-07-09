//! Service layer: orchestration and business rules.
//!
//! Services own Repositories and FileStorage handles. Controllers call
//! services; services never call controllers.

pub mod admission_service;
pub mod blob_service;
pub mod blob_upload_service;
pub mod catalog_service;
pub mod error;
pub mod gc_service;
pub mod health_service;
pub mod manifest_service;
pub mod proxy_service;
pub mod referrers_service;

use std::sync::Arc;

use self::admission_service::AdmissionService;
use self::blob_service::BlobService;
use self::blob_upload_service::BlobUploadService;
use self::catalog_service::CatalogService;
pub use self::error::Error;
use self::gc_service::GcService;
use self::health_service::HealthService;
use self::manifest_service::ManifestService;
use self::proxy_service::ProxyService;
use self::referrers_service::ReferrersService;
use crate::TrowConfig;
use crate::file_storage::FileStorage;
use crate::repositories::Repositories;

impl std::fmt::Debug for Services {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Services").finish_non_exhaustive()
    }
}

pub struct Services {
    pub blob: BlobService,
    pub blob_upload: BlobUploadService,
    pub manifest: ManifestService,
    pub catalog: CatalogService,
    pub referrers: ReferrersService,
    pub proxy: Arc<ProxyService>,
    pub gc: Arc<GcService>,
    pub admission: AdmissionService,
    pub health: HealthService,
    #[doc(hidden)]
    repos_shared: Arc<Repositories>,
    #[doc(hidden)]
    storage_shared: Arc<FileStorage>,
}

impl Services {
    pub fn new(
        repos: Arc<Repositories>,
        storage: Arc<FileStorage>,
        config: Arc<TrowConfig>,
    ) -> Self {
        let proxy = Arc::new(ProxyService::new(repos.clone(), storage.clone()));
        Self {
            blob: BlobService::new(repos.clone(), storage.clone()),
            blob_upload: BlobUploadService::new(repos.clone(), storage.clone()),
            manifest: ManifestService::new(repos.clone(), config.clone(), proxy.clone()),
            catalog: CatalogService::new(repos.clone()),
            referrers: ReferrersService::new(repos.clone()),
            proxy,
            gc: Arc::new(GcService::new(
                repos.clone(),
                storage.clone(),
                config.clone(),
            )),
            admission: AdmissionService::new(config.clone()),
            health: HealthService::new(storage.clone()),
            repos_shared: repos.clone(),
            storage_shared: storage.clone(),
        }
    }

    #[doc(hidden)]
    pub fn repos(&self) -> &Arc<Repositories> {
        &self.repos_shared
    }

    #[doc(hidden)]
    pub fn storage(&self) -> &Arc<FileStorage> {
        &self.storage_shared
    }
}
