use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{FixedOffset, TimeZone, Utc};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;
use sls_releases::routes::transactions::TransactionsState;

async fn body_string(resp: axum::response::Response) -> String {
    let bytes = resp
        .into_body()
        .collect()
        .await
        .expect("body collect")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf-8")
}

fn csv_non_empty_line_count(s: &str) -> usize {
    s.lines().filter(|l| !l.is_empty()).count()
}

fn releases_state_with_real_client(base_url: String) -> ReleasesState {
    let mut known = std::collections::HashMap::new();
    known.insert("a".to_string(), "A".to_string());
    known.insert("b".to_string(), "B".to_string());
    known.insert("m".to_string(), "M".to_string());

    ReleasesState {
        github: std::sync::Arc::new(GitHubClient::new_with_base_url(
            "test-token".to_string(),
            base_url,
        )),
        converter: std::sync::Arc::new(Converter::new(known)),
    }
}

async fn stub_releases_page(server: &MockServer, page: i32, body: serde_json::Value, status: u16) {
    let template = ResponseTemplate::new(status)
        .insert_header("content-type", "application/json")
        .insert_header("cache-control", "max-age=60")
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

    let app = routes::releases::router(releases_state_with_real_client(server.uri()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
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

    let app = routes::releases::router(releases_state_with_real_client(server.uri()));

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

    let app = routes::releases::router(releases_state_with_real_client(server.uri()));

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
async fn releases_list_github_error_maps_to_502() {
    let server = MockServer::start().await;
    // Non-200 on page=0 should map to BAD_GATEWAY in routes.
    let template = ResponseTemplate::new(500).set_body_string("boom");
    Mock::given(method("GET"))
        .and(path("/repos/crystalservice/SET10-Loyalty/releases"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", "0"))
        .respond_with(template)
        .mount(&server)
        .await;

    let app = routes::releases::router(releases_state_with_real_client(server.uri()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .body(Body::empty())
                .unwrap(),
        )
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

    let app = routes::releases::router(releases_state_with_real_client(server.uri()));

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases/m")
                .body(Body::empty())
                .unwrap(),
        )
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

#[tokio::test]
async fn transactions_route_valid_id_returns_exact_json() {
    fn encode_long(value: i64) -> String {
        const DIGITS: &[u8; 62] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let mut buf = [b'0'; 11];
        let mut v = value;
        if v < 0 {
            for i in (1..=10).rev() {
                let digit = (-(v % 62)) as usize;
                buf[i] = DIGITS[digit];
                v /= 62;
            }
            let first = (-(v - 31)) as usize;
            buf[0] = DIGITS[first];
        } else {
            for i in (1..=10).rev() {
                let digit = (v % 62) as usize;
                buf[i] = DIGITS[digit];
                v /= 62;
            }
            buf[0] = DIGITS[v as usize];
        }
        String::from_utf8(buf.to_vec()).unwrap()
    }

    let internal_id = 123456789i64;
    let seconds = 1710000000i64;
    let id = format!("{}{}", encode_long(internal_id), encode_long(seconds));

    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let app = routes::transactions::router(TransactionsState { zone_offset: offset });

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/transactions/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = body_string(resp).await;
    let dt = Utc.timestamp_opt(seconds, 0).unwrap();
    let created = dt
        .with_timezone(&FixedOffset::east_opt(3 * 3600).unwrap())
        .naive_local();

    let expected = format!(
        "{{\"id\":{internal_id},\"created\":\"{}\"}}",
        created.format("%Y-%m-%dT%H:%M:%S")
    );
    assert_eq!(body, expected);
}

#[tokio::test]
async fn transactions_route_invalid_id_returns_400_and_message() {
    let offset = FixedOffset::east_opt(3 * 3600).unwrap();
    let app = routes::transactions::router(TransactionsState { zone_offset: offset });

    let bad = "not-valid";
    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/transactions/{bad}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = body_string(resp).await;
    assert_eq!(body, format!("Invalid transaction ID: '{bad}'"));
}

