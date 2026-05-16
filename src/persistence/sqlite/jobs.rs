use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::domain::job::{AnyJob, Job, JobResult, JobStatus};
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

        let job_id = job.id();
        sqlx::query("INSERT INTO jobs (id) VALUES (?)")
            .bind(&job_id)
            .execute(&mut *tx)
            .await?;

        match job {
            Job::CreateRelease {
                id,
                milestone,
                candidate,
                description,
            } => {
                sqlx::query("INSERT INTO create_release_jobs (id, milestone, candidate, description) VALUES (?, ?, ?, ?)")
                .bind(id)
                .bind(milestone)
                .bind(candidate)
                .bind(description)
                .execute(&mut *tx)
                .await?;
            }
            Job::DeleteRelease { id, tag } => {
                sqlx::query("INSERT INTO delete_release_jobs (id, tag) VALUES (?, ?)")
                    .bind(id)
                    .bind(tag)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_job(&self, id: &str) -> Result<JobResult, PersistenceError> {
        let row = sqlx::query(
            r#"
          SELECT id, status, error_code, error_detail FROM jobs WHERE id = ?
        "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => PersistenceError::NotFound(),
            e => PersistenceError::Sql(e),
        })?;
        Ok(JobResult {
            id: row.try_get("id")?,
            status: to_status(row.try_get("status")?)?,
            error_code: row.try_get("error_code")?,
            error_detail: row.try_get("error_detail")?,
        })
    }

    async fn get_next_job(&self) -> Result<Job, PersistenceError> {
        let mut tx = self.pool.begin().await?;

        let id_row = sqlx::query(
            r#"
            SELECT id
            FROM jobs
            WHERE status = 'pending'
            ORDER BY created_at, id
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(id_row) = id_row else {
            return Err(PersistenceError::NotFound());
        };
        let id: String = id_row.try_get("id")?;

        let updated =
            sqlx::query("UPDATE jobs SET status = 'running' WHERE id = ? AND status = 'pending'")
                .bind(&id)
                .execute(&mut *tx)
                .await?;

        if updated.rows_affected() == 0 {
            return Err(PersistenceError::NotFound());
        }

        if let Ok(detail) = sqlx::query(
            r#"
            SELECT milestone, candidate, description
            FROM create_release_jobs
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&mut *tx)
        .await
        {
            tx.commit().await?;
            return Ok(Job::CreateRelease {
                id,
                milestone: detail.try_get("milestone")?,
                candidate: detail.try_get("candidate")?,
                description: detail.try_get("description")?,
            });
        }

        let detail = sqlx::query(
            r#"
            SELECT tag
            FROM delete_release_jobs
            WHERE id = ?
            "#,
        )
        .bind(&id)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(Job::DeleteRelease {
            id,
            tag: detail.try_get("tag")?,
        })
    }

    async fn set_job_result(&self, id: &str, result: JobResult) -> Result<(), PersistenceError> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = ?, error_code = ?, error_detail = ?
            WHERE id = ?
            "#,
        )
        .bind(from_status(result.status))
        .bind(&result.error_code)
        .bind(&result.error_detail)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if updated.rows_affected() == 0 {
            return Err(PersistenceError::NotFound());
        }
        Ok(())
    }
}

fn from_status(status: JobStatus) -> &'static str {
    match status {
        JobStatus::Pending => "pending",
        JobStatus::Running => "running",
        JobStatus::Ok => "ok",
        JobStatus::Failed => "failed",
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
