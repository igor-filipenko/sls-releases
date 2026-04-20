//! Types for release HTTP handlers.

use serde::Serialize;

use crate::domain::release::{ModuleRelease, Release, ReleaseKind, Version};

#[derive(Debug, serde::Deserialize)]
pub struct ReleasesQuery {
    pub rc: Option<String>,
    pub ms: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReleaseRow {
    pub name: String,
    pub localized_name: String,
    pub kind: ReleaseKind,
    pub version: Version,
    pub url: String,
    pub date_time: String,
    pub closed: bool,
}

impl From<&Release> for ReleaseRow {
    fn from(value: &Release) -> Self {
        Self {
            name: value.name.clone(),
            localized_name: value.localized_name.clone(),
            kind: value.kind,
            version: value.version.clone(),
            url: value.url.clone(),
            date_time: value.date_time.clone(),
            closed: value.closed,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ModuleReleaseRow {
    pub version: Version,
    pub url: String,
    pub date_time: String,
}

impl From<&ModuleRelease> for ModuleReleaseRow {
    fn from(value: &ModuleRelease) -> Self {
        Self {
            version: value.version.clone(),
            url: value.url.clone(),
            date_time: value.date_time.clone(),
        }
    }
}
