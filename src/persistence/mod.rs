pub mod migrations;
mod sqlite;

pub use sqlite::SqliteReleasesStore;

use crate::domain::module::Module;
use crate::domain::release::{Release, ReleaseKind, Version};
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("invalid version_kind in database: {0}")]
    InvalidVersionKind(String),
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
}

#[async_trait]
impl ReleasesStore for SqliteReleasesStore {
    async fn get_all_releases(&self, include: &Include) -> Result<Vec<Release>, PersistenceError> {
        SqliteReleasesStore::get_all_releases(self, include).await
    }

    async fn get_releases_by_name(
        &self,
        name: &str,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        SqliteReleasesStore::get_releases_by_name(self, name, include).await
    }

    async fn replace_all_releases(&self, releases: Vec<Release>) -> Result<(), PersistenceError> {
        SqliteReleasesStore::replace_all_releases(self, releases).await
    }

    async fn list_modules(&self, name: Option<&str>) -> Result<Vec<Module>, PersistenceError> {
        SqliteReleasesStore::list_modules(self, name).await
    }
}

pub(crate) fn version_parts(r: &Release) -> (ReleaseKind, i32, i32, i32, Option<i32>) {
    let kind = r.kind;
    match &r.version {
        Version::Release {
            major,
            minor,
            patch,
        } => (kind, *major, *minor, *patch, None),
        Version::Candidate {
            major,
            minor,
            patch,
            number,
        } => (
            ReleaseKind::Candidate,
            *major,
            *minor,
            *patch,
            Some(*number),
        ),
    }
}

pub(crate) fn version_kind_db_str(kind: ReleaseKind) -> &'static str {
    match kind {
        ReleaseKind::Milestone => "milestone",
        ReleaseKind::Production => "production",
        ReleaseKind::Candidate => "candidate",
    }
}

pub(crate) fn version_from_row(
    kind: &str,
    major: i32,
    minor: i32,
    patch: i32,
    rc: Option<i32>,
) -> Result<Version, PersistenceError> {
    match kind {
        "production" | "milestone" | "release" => Ok(Version::Release {
            major,
            minor,
            patch,
        }),
        "candidate" => Ok(Version::Candidate {
            major,
            minor,
            patch,
            number: rc.ok_or_else(|| {
                PersistenceError::InvalidVersionKind("candidate without rc_number".into())
            })?,
        }),
        other => Err(PersistenceError::InvalidVersionKind(other.to_string())),
    }
}
