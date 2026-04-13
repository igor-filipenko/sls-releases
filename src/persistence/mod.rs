mod sqlite;

pub use sqlite::SqliteReleasesStore;

use std::future::Future;
use std::pin::Pin;

use crate::domain::release::{Release, Version};

#[derive(Debug, thiserror::Error)]
pub enum PersistenceError {
    #[error("database error: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("invalid version_kind in database: {0}")]
    InvalidVersionKind(String),
}

pub trait ReleasesStore: Send + Sync {
    fn get_all_releases<'a>(
        &'a self,
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

    fn replace_all_releases<'a>(
        &'a self,
        releases: Vec<Release>,
    ) -> Pin<Box<dyn Future<Output = Result<(), PersistenceError>> + Send + 'a>> {
        Box::pin(async move { SqliteReleasesStore::replace_all_releases(self, releases).await })
    }
}

pub(crate) fn version_parts(
    v: &Version,
) -> (
    &'static str,
    i32,
    i32,
    i32,
    Option<i32>,
) {
    match v {
        Version::Release {
            major,
            minor,
            patch,
        } => ("release", *major, *minor, *patch, None),
        Version::Candidate {
            major,
            minor,
            patch,
            number,
        } => ("candidate", *major, *minor, *patch, Some(*number)),
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
        "release" => Ok(Version::Release {
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
        // Legacy rows (if any); treat like release for ordering parity with old Milestone.
        "milestone" => Ok(Version::Release {
            major,
            minor,
            patch,
        }),
        other => Err(PersistenceError::InvalidVersionKind(other.to_string())),
    }
}
