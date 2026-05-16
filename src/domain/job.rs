/// A trait for any job.
pub trait AnyJob {
    fn id(&self) -> String;
}

/// Background work enqueued by the API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Job {
    /// Create a new release from a milestone tag (and optional RC).
    CreateRelease {
        id: String,
        milestone: String,
        candidate: bool,
        description: Option<String>,
    },
    /// Delete an existing candidate release by GitHub tag.
    DeleteRelease { id: String, tag: String },
}

impl AnyJob for Job {
    fn id(&self) -> String {
        match self {
            Job::CreateRelease { id, .. } | Job::DeleteRelease { id, .. } => id.clone(),
        }
    }
}

/// Lifecycle state of a persisted job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Ok,
    Failed,
}

/// Snapshot of a job returned by the store after creation or lookup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobResult {
    pub id: String,
    pub status: JobStatus,
    pub error_code: Option<String>,
    pub error_detail: Option<String>,
}
