use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::persistence::{Job, JobResult, JobsStore, PersistenceError};

pub struct SqliteJobsStore {
    pool: SqlitePool,
}

impl SqliteJobsStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobsStore for SqliteJobsStore {
    async fn create_job(&self, job: &Job) -> Result<(), PersistenceError> {
        /*
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query("INSERT INTO jobs (id, status, error_code, error_detail) VALUES (?, ?, ?, ?)")
            .bind(&job.id)
            .bind(&job.status)
            .bind(job.error_code.clone())
            .bind(job.error_detail.clone())
            .execute(&mut tx)
            .await?;
        tx.commit().await?;
        */
        Ok(())
    }

    async fn get_job(&self, id: &str) -> Result<JobResult, PersistenceError> {
        /*
        let result = sqlx::query_as("SELECT id, status, error_code, error_detail FROM jobs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;
        Ok(JobResult {
            id: result.id,
            status: result.status,
            error_code: result.error_code,
            error_detail: result.error_detail,
        })
        */
        Err(PersistenceError::NotFound())
    }
}
