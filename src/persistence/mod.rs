mod sqlite;
pub mod migrations;

pub use sqlite::SqliteReleasesStore;

use std::future::Future;
use std::pin::Pin;

use crate::domain::release::{Release, ReleaseKind, Version};

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("invalid version_kind in database: {0}")]
    InvalidVersionKind(String),
}

/// Persistence interface for fetching and storing releases.
///
/// ## Ordering contract
/// Implementations must return releases **ordered by `name` (ascending)** in `get_all_releases`.
/// Callers rely on this for stable output ordering without doing additional sorting.
pub trait ReleasesStore: Send + Sync {
    fn get_all_releases<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>>;

    /// Returns all releases for a single module.
    ///
    /// The returned list is expected to be stable; callers may apply their own version ordering.
    fn get_releases_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>>;

    fn replace_all_releases<'a>(
        &'a self,
        releases: Vec<Release>,
    ) -> Pin<Box<dyn Future<Output = Result<(), PersistenceError>> + Send + 'a>>;
}

impl ReleasesStore for SqliteReleasesStore {
    fn get_all_releases<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>> {
        Box::pin(async move { SqliteReleasesStore::get_all_releases(self).await })
    }

    fn get_releases_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>> {
        Box::pin(async move { SqliteReleasesStore::get_releases_by_name(self, name).await })
    }

    fn replace_all_releases<'a>(
        &'a self,
        releases: Vec<Release>,
    ) -> Pin<Box<dyn Future<Output = Result<(), PersistenceError>> + Send + 'a>> {
        Box::pin(async move { SqliteReleasesStore::replace_all_releases(self, releases).await })
    }
}

pub(crate) fn version_parts(
    r: &Release,
) -> (
    ReleaseKind,
    i32,
    i32,
    i32,
    Option<i32>,
) {
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
        } => (ReleaseKind::Candidate, *major, *minor, *patch, Some(*number)),
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
