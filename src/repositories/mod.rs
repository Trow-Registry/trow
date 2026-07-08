//! Repository layer: database metadata access.
//!
//! All `sqlx::query!` calls live here. Services call repositories;
//! repositories never call services.

pub mod blob_repository;
pub mod blob_upload_repository;
pub mod manifest_repository;
pub mod models;
pub mod repo_blob_assoc_repository;
pub mod tag_repository;

use sqlx::migrate::MigrateError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};

pub use self::blob_repository::BlobRepository;
pub use self::blob_upload_repository::BlobUploadRepository;
pub use self::manifest_repository::ManifestRepository;
pub use self::repo_blob_assoc_repository::RepoBlobAssocRepository;
pub use self::tag_repository::TagRepository;

/// Aggregate of every repository. Constructed once at startup, shared as
/// `Arc<Repositories>` by services.
pub struct Repositories {
    pub blob: BlobRepository,
    pub blob_upload: BlobUploadRepository,
    pub manifest: ManifestRepository,
    pub tag: TagRepository,
    pub repo_blob_assoc: RepoBlobAssocRepository,
}

impl std::fmt::Debug for Repositories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repositories").finish_non_exhaustive()
    }
}

impl Repositories {
    /// Opens (or creates) the SQLite database at `db_file`, runs migrations,
    /// and returns a ready-to-use `Repositories` instance.
    pub async fn new(db_file: &str) -> Result<Self, MigrateError> {
        let options = SqliteConnectOptions::new()
            .filename(db_file)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .foreign_keys(true);

        let db_rw = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options.clone().create_if_missing(true))
            .await?;

        let db_ro = SqlitePoolOptions::new()
            .connect_with(options.read_only(true))
            .await?;

        sqlx::migrate!().run(&db_rw).await?;

        Ok(Self {
            blob: BlobRepository::new(db_ro.clone(), db_rw.clone()),
            blob_upload: BlobUploadRepository::new(db_ro.clone(), db_rw.clone()),
            manifest: ManifestRepository::new(db_ro.clone(), db_rw.clone()),
            tag: TagRepository::new(db_ro.clone(), db_rw.clone()),
            repo_blob_assoc: RepoBlobAssocRepository::new(db_ro, db_rw),
        })
    }

    /// Construct from pre-built pools (used by tests with in-memory SQLite).
    pub fn from_pools(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self {
            blob: BlobRepository::new(db_ro.clone(), db_rw.clone()),
            blob_upload: BlobUploadRepository::new(db_ro.clone(), db_rw.clone()),
            manifest: ManifestRepository::new(db_ro.clone(), db_rw.clone()),
            tag: TagRepository::new(db_ro.clone(), db_rw.clone()),
            repo_blob_assoc: RepoBlobAssocRepository::new(db_ro, db_rw),
        }
    }

    #[doc(hidden)]
    pub fn db_rw(&self) -> &SqlitePool {
        self.blob.db_rw()
    }

    #[doc(hidden)]
    pub fn db_ro(&self) -> &SqlitePool {
        self.blob.db_ro()
    }
}
