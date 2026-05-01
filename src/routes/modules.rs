use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Json;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

use crate::persistence::ReleasesStore;
use crate::routes::dto::modules::{Module as ModuleDto, ModulesQuery};
use crate::routes::render;

#[derive(Clone)]
pub struct ModulesState {
    pub store: Arc<dyn ReleasesStore>,
}

pub fn router(state: ModulesState) -> Router {
    Router::new()
        .route("/sls/modules", get(list_modules))
        .with_state(state)
}

async fn list_modules(
    State(state): State<ModulesState>,
    headers: HeaderMap,
    Query(q): Query<ModulesQuery>,
) -> Result<Response, StatusCode> {
    let name_filter = q.name.as_deref().filter(|s| !s.is_empty());

    let modules = state
        .store
        .list_modules(name_filter)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    if accepts_html(&headers) {
        Ok((
            StatusCode::OK,
            [("content-type", "text/html; charset=utf-8")],
            render::modules_table_html(&modules),
        )
            .into_response())
    } else if accepts_json(&headers) {
        let body: Vec<ModuleDto> = modules.into_iter().map(ModuleDto::from).collect();
        Ok(Json(body).into_response())
    } else {
        Ok((
            StatusCode::OK,
            [("content-type", "text/plain; charset=utf-8")],
            render::modules_csv(&modules),
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
