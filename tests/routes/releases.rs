use std::future::Future;
use std::pin::Pin;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::domain::release::Release;
use sls_releases::persistence::{PersistenceError, ReleasesStore, SqliteReleasesStore};
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;

use super::{body_string, csv_non_empty_line_count};

/// Fetch releases from a wiremock GitHub stub, then seed an in-memory SQLite store (same rows the old handler would have used).
async fn releases_state_seeded_from_github(base_url: String) -> ReleasesState {
    let mut known = std::collections::HashMap::new();
    known.insert("a".to_string(), "A".to_string());
    known.insert("b".to_string(), "B".to_string());
    known.insert("m".to_string(), "M".to_string());

    let client = GitHubClient::new_with_base_url(
        "test-token".to_string(),
        base_url,
        "test-agent".to_string(),
    );
    let converter = Converter::new(known);
    let releases = client
        .get_releases(&converter)
        .await
        .expect("stubbed GitHub should succeed");

    let store = SqliteReleasesStore::in_memory()
        .await
        .expect("in-memory sqlite");
    store
        .replace_all_releases(releases)
        .await
        .expect("seed store");

    ReleasesState {
        store: std::sync::Arc::new(store),
    }
}

struct AlwaysFailingStore;

impl ReleasesStore for AlwaysFailingStore {
    fn get_all_releases<'a>(
        &'a self,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>> {
        Box::pin(async move { Err(PersistenceError::InvalidVersionKind("test".into())) })
    }

    fn replace_all_releases<'a>(
        &'a self,
        _releases: Vec<Release>,
    ) -> Pin<Box<dyn Future<Output = Result<(), PersistenceError>> + Send + 'a>> {
        Box::pin(async move { Ok(()) })
    }
}

async fn stub_releases_page(server: &MockServer, page: i32, body: serde_json::Value, status: u16) {
    let template = ResponseTemplate::new(status)
        .insert_header("content-type", "application/json")
        .set_body_json(body);

    Mock::given(method("GET"))
        .and(path("/repos/crystalservice/SET10-Loyalty/releases"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", page.to_string()))
        .respond_with(template)
        .mount(server)
        .await;
}

#[tokio::test]
async fn releases_list_csv_default_accept_and_line_count() {
    let server = MockServer::start().await;

    // Page 0 includes: a release, a candidate (should be filtered out by default), b release.
    stub_releases_page(
        &server,
        0,
        json!([
          {"tag_name":"a-v1.0.0","html_url":"https://example/a100","created_at":"2026-01-01T00:00:00Z"},
          {"tag_name":"a-v1.0.1-RC9","html_url":"https://example/a101rc9","created_at":"2026-01-02T00:00:00Z"},
          {"tag_name":"b-v3.0.0","html_url":"https://example/b300","created_at":"2026-01-03T00:00:00Z"}
        ]),
        200,
    )
    .await;
    // Page 1 empty to terminate paging loop.
    stub_releases_page(&server, 1, json!([]), 200).await;

    let app = routes::releases::router(releases_state_seeded_from_github(server.uri()).await);

    let resp = app
        .oneshot(Request::builder().uri("/sls/releases").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(axum::http::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/plain; charset=utf-8"
    );

    let body = body_string(resp).await;
    assert!(!body.is_empty());
    assert_eq!(csv_non_empty_line_count(&body), 2);
}

#[tokio::test]
async fn releases_list_csv_rc_true_includes_candidates_and_picks_latest() {
    let server = MockServer::start().await;
    stub_releases_page(
        &server,
        0,
        json!([
          {"tag_name":"a-v1.0.0","html_url":"https://example/a100","created_at":"2026-01-01T00:00:00Z"},
          {"tag_name":"a-v1.0.1-RC9","html_url":"https://example/a101rc9","created_at":"2026-01-02T00:00:00Z"}
        ]),
        200,
    )
    .await;
    stub_releases_page(&server, 1, json!([]), 200).await;

    let app = routes::releases::router(releases_state_seeded_from_github(server.uri()).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases?rc=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(!body.is_empty());
    assert_eq!(csv_non_empty_line_count(&body), 1);
    assert!(body.contains("a, A, 1.0.1-RC9, https://example/a101rc9"));
}

#[tokio::test]
async fn releases_list_html_accept_header_renders_html() {
    let server = MockServer::start().await;
    stub_releases_page(
        &server,
        0,
        json!([
          {"tag_name":"a-v1.0.0","html_url":"https://example/a100","created_at":"2026-01-01T00:00:00Z"}
        ]),
        200,
    )
    .await;
    stub_releases_page(&server, 1, json!([]), 200).await;

    let app = routes::releases::router(releases_state_seeded_from_github(server.uri()).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .header(axum::http::header::ACCEPT, "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(axum::http::header::CONTENT_TYPE)
            .unwrap()
            .to_str()
            .unwrap(),
        "text/html; charset=utf-8"
    );
    let body = body_string(resp).await;
    assert!(!body.is_empty());
    assert!(body.contains("<table"));
}

#[tokio::test]
async fn releases_list_store_error_maps_to_502() {
    let app = routes::releases::router(ReleasesState {
        store: std::sync::Arc::new(AlwaysFailingStore),
    });

    let resp = app
        .oneshot(Request::builder().uri("/sls/releases").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn releases_module_csv_filters_and_orders_versions_desc() {
    let server = MockServer::start().await;
    stub_releases_page(
        &server,
        0,
        json!([
          {"tag_name":"m-v1.0.0","html_url":"https://example/m100","created_at":"2026-01-01T00:00:00Z"},
          {"tag_name":"m-v2.0.0","html_url":"https://example/m200","created_at":"2026-01-02T00:00:00Z"},
          {"tag_name":"other-v9.9.9","html_url":"https://example/o999","created_at":"2026-01-03T00:00:00Z"}
        ]),
        200,
    )
    .await;
    stub_releases_page(&server, 1, json!([]), 200).await;

    let app = routes::releases::router(releases_state_seeded_from_github(server.uri()).await);

    let resp = app
        .oneshot(Request::builder().uri("/sls/releases/m").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(!body.is_empty());
    assert_eq!(csv_non_empty_line_count(&body), 2);

    // Ordering: version desc, so 2.0.0 should appear before 1.0.0.
    let first_line = body.lines().find(|l| !l.is_empty()).unwrap();
    assert!(first_line.contains("2.0.0"));
}

