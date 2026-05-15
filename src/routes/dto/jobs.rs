use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum JobStatusDto {
  Pending,
  Running,
  Ok,
  Failed,
}

#[derive(Debug, Serialize)]
pub struct JobDto {
  pub id: String,
  pub status: JobStatusDto,
  pub errorCode: Option<String>,
  pub errorDetail: Option<String>,
}