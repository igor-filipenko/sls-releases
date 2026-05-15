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
    #[serde(rename = "errorCode")]
    pub error_code: Option<String>,
    #[serde(rename = "errorDetail")]
    pub error_detail: Option<String>,
}
