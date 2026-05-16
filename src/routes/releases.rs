use std::collections::BTreeMap;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};

use crate::domain::job::{Job, JobStatus};
use crate::domain::release::{ModuleRelease, Release, ReleaseKind, parse_tag};
use crate::persistence::{Include, PersistenceError, Stores};
use crate::routes::dto::jobs::{JobDto, JobStatusDto};
use crate::routes::dto::releases::{CreateReleaseQuery, ReleaseRow, ReleasesQuery};
use crate::routes::{map_store_error, render};

#[derive(Clone)]
pub struct ReleasesState {
    pub store: Stores,
}

pub fn router(state: ReleasesState) -> Router {
    Router::new()
        .route("/sls/releases", get(list_latest))
        .route("/sls/releases/{module}", get(list_module))
        .route("/sls/releases", post(create_release))
        .route("/sls/jobs/{id}", get(get_job))
        .with_state(state)
}

async fn list_latest(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    uri: Uri,
    Query(q): Query<ReleasesQuery>,
) -> Result<Response, StatusCode> {
    let use_rc = to_boolean(q.rc.as_deref());
    let use_ms = to_boolean(q.ms.as_deref());

    let include = releases_query_include(use_rc, use_ms);

    let all = state
        .store
        .releases
        .get_all_releases(&include)
        .await
        .map_err(|e| map_store_error("/sls/releases", e))?;

    let mut by_name: BTreeMap<String, Release> = BTreeMap::new();
    for r in all {
        by_name
            .entry(r.name.clone())
            .and_modify(|cur| {
                if r > *cur {
                    *cur = r.clone();
                }
            })
            .or_insert(r);
    }

    let latest: Vec<Release> = by_name.into_values().collect();

    if accepts_html(&headers) {
        let base_url = uri.to_string();
        Ok((
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            render::releases_table_html(&base_url, use_rc, use_ms, &latest),
        )
            .into_response())
    } else if accepts_json(&headers) {
        let body: Vec<ReleaseRow> = latest.iter().map(ReleaseRow::from).collect();
        Ok(Json(body).into_response())
    } else {
        Ok((
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            render::releases_csv(&latest),
        )
            .into_response())
    }
}

async fn list_module(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    Path(module): Path<String>,
    Query(q): Query<ReleasesQuery>,
) -> Result<Response, StatusCode> {
    let use_rc = to_boolean(q.rc.as_deref());
    let use_ms = to_boolean(q.ms.as_deref());

    let include = releases_query_include(use_rc, use_ms);

    let all = state
        .store
        .releases
        .get_releases_by_name(&module, &include)
        .await
        .map_err(|e| map_store_error("/sls/releases/{module}", e))?;

    let mut releases = all;
    releases.sort_by(|a, b| b.version.cmp(&a.version));

    if accepts_html(&headers) {
        let module_views = releases_as_module_views(&releases);
        Ok((
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            render::module_releases_table_html(&module_views),
        )
            .into_response())
    } else if accepts_json(&headers) {
        let body: Vec<ReleaseRow> = releases.iter().map(ReleaseRow::from).collect();
        Ok(Json(body).into_response())
    } else {
        let module_views = releases_as_module_views(&releases);
        Ok((
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            render::module_releases_csv(&module_views),
        )
            .into_response())
    }
}

async fn create_release(
    State(state): State<ReleasesState>,
    _: HeaderMap,
    uri: Uri,
    Json(q): Json<CreateReleaseQuery>,
) -> Result<Response, StatusCode> {
    let to_status_error_code = |err| map_store_error(uri.to_string().as_str(), err);
    let (_, version) = parse_tag(&q.milestone)
        .ok_or_else(|| PersistenceError::NotFound())
        .map_err(to_status_error_code)?;
    let release = state
        .store
        .releases
        .get_release(&version)
        .await
        .map_err(|e| match e {
            PersistenceError::Sql(sqlx::Error::RowNotFound) => PersistenceError::NotFound(),
            other => other,
        })
        .map_err(to_status_error_code)?;
    if release.kind != ReleaseKind::Milestone || release.closed {
        return Err(to_status_error_code(PersistenceError::NotFound()));
    }

    let id = uuid7::uuid7().to_string();
    let job = Job::CreateRelease {
        id: id.clone(),
        milestone: q.milestone,
        candidate: q.candidate,
        description: q.description,
    };
    state
        .store
        .jobs
        .create_job(&job)
        .await
        .map_err(to_status_error_code)?;

    let dto = JobDto {
        id: id,
        status: JobStatusDto::Pending,
        error_code: None,
        error_detail: None,
    };
    Ok(Json(dto).into_response())
}

async fn get_job(
    State(state): State<ReleasesState>,
    _: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let job = state
        .store
        .jobs
        .get_job(&id)
        .await
        .map_err(|e| map_store_error("/sls/jobs/{id}", e))?;
    let status = match job.status {
        JobStatus::Pending => JobStatusDto::Pending,
        JobStatus::Running => JobStatusDto::Running,
        JobStatus::Ok => JobStatusDto::Ok,
        JobStatus::Failed => JobStatusDto::Failed,
    };
    let dto = JobDto {
        id: job.id,
        status: status,
        error_code: job.error_code,
        error_detail: job.error_detail,
    };
    Ok(Json(dto).into_response())
}

fn accepts_html(headers: &HeaderMap) -> bool {
    headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("html"))
        .unwrap_or(false)
}

fn accepts_json(headers: &HeaderMap) -> bool {
    headers
        .get(axum::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.contains("json"))
        .unwrap_or(false)
}

fn to_boolean(s: Option<&str>) -> bool {
    matches!(s, Some(v) if v.eq_ignore_ascii_case("true"))
}

fn releases_query_include(use_rc: bool, use_ms: bool) -> Include {
    Include {
        candidates: use_rc,
        milestones: use_ms,
    }
}

fn releases_as_module_views(releases: &[Release]) -> Vec<ModuleRelease> {
    releases
        .iter()
        .map(|r| ModuleRelease {
            version: r.version.clone(),
            url: r.url.clone(),
            date_time: r.date_time.clone(),
        })
        .collect()
}
