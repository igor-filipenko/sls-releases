pub mod sqlite;

use std::sync::Arc;

use crate::domain::job::{Job, JobResult};
use crate::domain::module::Module;
use crate::domain::release::{Release, Version};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("invalid version_kind in database: {0}")]
    InvalidVersionKind(String),
    #[error("invalid job status in database: {0}")]
    InvalidJobStatus(String),
    #[error("not found")]
    NotFound(),
}

#[derive(Debug, thiserror::Error)]
pub enum PersistenceConnectionError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

/// Controls which persisted rows are returned by read queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Include {
    pub candidates: bool,
    pub milestones: bool,
}

impl Include {
    pub const fn all() -> Self {
        Self {
            candidates: true,
            milestones: true,
        }
    }
}

/// Persistence interface for fetching and storing releases.
///
/// ## Ordering contract
/// Implementations must return releases **ordered by `name` (ascending)** in `get_all_releases`.
/// Callers rely on this for stable output ordering without doing additional sorting.
#[async_trait]
pub trait ReleasesStore: Send + Sync {
    async fn get_all_releases(&self, include: &Include) -> Result<Vec<Release>, PersistenceError>;

    /// Returns all releases for a single module.
    ///
    /// The returned list is expected to be stable; callers may apply their own version ordering.
    async fn get_releases_by_name(
        &self,
        name: &str,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError>;

    async fn replace_all_releases(&self, releases: Vec<Release>) -> Result<(), PersistenceError>;

    /// Lists modules from the `modules` table, ordered by `name` ascending.
    /// When `name` is `Some`, returns at most one row with that exact primary key.
    async fn list_modules(&self, name: Option<&str>) -> Result<Vec<Module>, PersistenceError>;

    /// Returns the release for a given version, if exists.
    async fn get_release(&self, version: &Version) -> Result<Release, PersistenceError>;
}

/// Persistence interface for creating jobs.
#[async_trait]
pub trait JobsStore: Send + Sync {
    /// Creates a new job.
    /// Returns the job ID.
    async fn create_job(&self, job: &Job) -> Result<(), PersistenceError>;

    /// Gets a job by ID.
    async fn get_job(&self, id: &str) -> Result<JobResult, PersistenceError>;
}

#[derive(Clone)]
pub struct Stores {
    pub releases: Arc<dyn ReleasesStore>,
    pub jobs: Arc<dyn JobsStore>,
}
