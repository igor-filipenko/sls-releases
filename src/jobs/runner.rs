use std::{sync::Arc, time::Duration};

use anyhow::Context;

use crate::{
    clients::github::ReleasesClient,
    domain::job::{AnyJob, Job, JobResult, JobStatus},
    persistence::{PersistenceError, Stores},
};

const LOOP_DELAY_SECS: u64 = 1;

pub async fn handle_next_job(
    _github: &Arc<dyn ReleasesClient>,
    stores: &Arc<Stores>,
) -> Result<(), anyhow::Error> {
    let job = match stores.jobs.get_next_job().await {
        Ok(job) => job,
        Err(PersistenceError::NotFound()) => return Ok(()),
        Err(e) => anyhow::bail!("failed to get next job: {e}"),
    };

    let job_id = job.id();
    let result = JobResult {
        id: job_id.clone(),
        status: JobStatus::Failed,
        error_code: Some("NOT_IMPLEMENTED".to_string()),
        error_detail: Some("Under construction".to_string()),
    };
    match job {
        Job::CreateRelease { .. } | Job::DeleteRelease { .. } => {
            stores
                .jobs
                .set_job_result(&job_id, result)
                .await
                .context("failed to persist job result")?;
        }
    }
    Ok(())
}

pub fn spawn_jobs_loop(github: Arc<dyn ReleasesClient>, stores: Arc<Stores>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(LOOP_DELAY_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = handle_next_job(&github, &stores).await {
                tracing::error!("failed to handle next job: {e}");
            }
        }
    });
}
