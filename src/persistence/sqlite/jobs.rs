use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::domain::job::{Job,JobResult, JobStatus};
use crate::persistence::{JobsStore, PersistenceError};

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
        let mut tx = self.pool.begin().await?;

        let job_id = match job {
            Job::CreateRelease { id, .. } => id.clone(),
        };
        sqlx::query("INSERT INTO jobs (id) VALUES (?)")
            .bind(&job_id)
            .execute(&mut *tx)
            .await?;

        match job {
            Job::CreateRelease { id, milestone, candidate, description } => {
                sqlx::query("INSERT INTO create_release_jobs (id, milestone, candidate, description) VALUES (?, ?, ?, ?)")
                .bind(&id)
                .bind(&milestone)
                .bind(&candidate)
                .bind(&description)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_job(&self, id: &str) -> Result<JobResult, PersistenceError> {
        let row = sqlx::query(r#"
          SELECT id, status, error_code, error_detail FROM jobs WHERE id = ?
        "#)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(JobResult {
            id: row.try_get("id")?,
            status: to_status(row.try_get("status")?)?,
            error_code: row.try_get("error_code")?,
            error_detail: row.try_get("error_detail")?,
        })
    }
}

fn to_status(status: &str) -> Result<JobStatus, PersistenceError> {
    match status {
        "pending" => Ok(JobStatus::Pending),
        "running" => Ok(JobStatus::Running),
        "ok" => Ok(JobStatus::Ok),
        "failed" => Ok(JobStatus::Failed),
        _ => Err(PersistenceError::InvalidJobStatus(status.to_string())),
    }
}