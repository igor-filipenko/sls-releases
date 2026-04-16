use std::collections::BTreeMap;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::get;

use crate::domain::release::{ModuleRelease, Release, Version};
use crate::persistence::ReleasesStore;
use crate::routes::render;

#[derive(Clone)]
pub struct ReleasesState {
    pub store: Arc<dyn ReleasesStore>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ReleasesQuery {
    pub rc: Option<String>,
}

pub fn router(state: ReleasesState) -> Router {
    Router::new()
        .route("/sls/releases", get(list_latest))
        .route("/sls/releases/{module}", get(list_module))
        .with_state(state)
}

async fn list_latest(
    State(state): State<ReleasesState>,
    headers: HeaderMap,
    uri: Uri,
    Query(q): Query<ReleasesQuery>,
) -> Result<Response, StatusCode> {
    let use_rc = to_boolean(q.rc.as_deref());

    let all = state
        .store
        .get_all_releases()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let filtered = all.into_iter().filter(|r| {
        if use_rc {
            true
        } else {
            matches!(r.version, Version::Release { .. })
        }
    });

    let mut by_name: BTreeMap<String, Release> = BTreeMap::new();
    for r in filtered {
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
            render::releases_table_html(&base_url, use_rc, &latest),
        )
            .into_response())
    } else if accepts_json(&headers) {
        Ok(Json(latest).into_response())
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

    let all = state
        .store
        .get_releases_by_name(&module)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut list: Vec<ModuleRelease> = all
        .into_iter()
        .filter(|r| {
            if !use_rc && !matches!(r.version, Version::Release { .. }) {
                return false;
            }
            true
        })
        .map(|r| ModuleRelease {
            version: r.version,
            url: r.url,
            date_time: r.date_time,
        })
        .collect();

    list.sort_by(|a, b| b.version.cmp(&a.version));

    if accepts_html(&headers) {
        Ok((
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            render::module_releases_table_html(&list),
        )
            .into_response())
    } else if accepts_json(&headers) {
        Ok(Json(list).into_response())
    } else {
        Ok((
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            render::module_releases_csv(&list),
        )
            .into_response())
    }
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
