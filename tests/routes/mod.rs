use std::sync::Arc;

use async_trait::async_trait;
use axum::response::Response;
use http_body_util::BodyExt;

use sls_releases::persistence::{Job, JobResult, JobsStore, PersistenceError, Stores};

struct NoopJobsStore;

#[async_trait]
impl JobsStore for NoopJobsStore {
    async fn create_job(&self, _job: &Job) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn get_job(&self, _id: &str) -> Result<JobResult, PersistenceError> {
        Err(PersistenceError::NotFound())
    }
}

pub fn stores_with_releases(releases: Arc<dyn sls_releases::persistence::ReleasesStore>) -> Stores {
    Stores {
        releases,
        jobs: Arc::new(NoopJobsStore),
    }
}

pub async fn body_string(resp: Response) -> String {
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("body collect")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf-8")
}

pub fn csv_non_empty_line_count(s: &str) -> usize {
    s.lines().filter(|l| !l.is_empty()).count()
}

#[path = "modules.rs"]
mod modules;
#[path = "releases.rs"]
mod releases;
#[path = "transactions.rs"]
mod transactions;
