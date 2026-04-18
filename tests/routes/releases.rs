use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use sls_releases::domain::release::Release;
use sls_releases::domain::release::ReleaseKind;
use sls_releases::domain::release::Version;
use sls_releases::persistence::{migrations, Include, PersistenceError, ReleasesStore, SqliteReleasesStore};
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;

use super::{body_string, csv_non_empty_line_count};

async fn releases_state_seeded(releases: Vec<Release>) -> ReleasesState {
    let store = SqliteReleasesStore::in_memory()
        .await
        .expect("in-memory sqlite");
    migrations::MIGRATOR
        .run(store.pool())
        .await
        .expect("run migrations");

    let mut seen: HashSet<&str> = HashSet::new();
    for rel in &releases {
        if seen.insert(rel.name.as_str()) {
            sqlx::query("INSERT OR REPLACE INTO modules (name, localized_name) VALUES (?, ?)")
                .bind(&rel.name)
                .bind(&rel.localized_name)
                .execute(store.pool())
                .await
                .expect("seed module for test");
        }
    }

    store
        .replace_all_releases(releases)
        .await
        .expect("seed store");

    ReleasesState {
        store: std::sync::Arc::new(store),
    }
}

fn r(name: &str, localized_name: &str, version: Version, url: &str) -> Release {
    let kind = match version {
        Version::Release { .. } => ReleaseKind::Production,
        Version::Candidate { .. } => ReleaseKind::Candidate,
    };
    Release {
        name: name.to_string(),
        localized_name: localized_name.to_string(),
        kind,
        version,
        url: url.to_string(),
        // These route tests don't validate date formatting; keep non-empty for realism.
        date_time: "2026-01-01T00:00:00Z".to_string(),
        closed: false,
    }
}

fn ms(name: &str, localized_name: &str, version: Version, url: &str) -> Release {
    Release {
        name: name.to_string(),
        localized_name: localized_name.to_string(),
        kind: ReleaseKind::Milestone,
        version,
        url: url.to_string(),
        date_time: "2026-01-01T00:00:00Z".to_string(),
        closed: false,
    }
}

struct AlwaysFailingStore;

impl ReleasesStore for AlwaysFailingStore {
    fn get_all_releases<'a>(
        &'a self,
        _include: &'a Include,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, PersistenceError>> + Send + 'a>> {
        Box::pin(async move { Err(PersistenceError::InvalidVersionKind("test".into())) })
    }

    fn get_releases_by_name<'a>(
        &'a self,
        _name: &'a str,
        _include: &'a Include,
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

#[tokio::test]
async fn releases_list_csv_default_accept_and_line_count() {
    let releases = vec![
        r(
            "a",
            "A",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/a100",
        ),
        r(
            "a",
            "A",
            Version::Candidate {
                major: 1,
                minor: 0,
                patch: 1,
                number: 9,
            },
            "https://example/a101rc9",
        ),
        r(
            "b",
            "B",
            Version::Release {
                major: 3,
                minor: 0,
                patch: 0,
            },
            "https://example/b300",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

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
    let releases = vec![
        r(
            "a",
            "A",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/a100",
        ),
        r(
            "a",
            "A",
            Version::Candidate {
                major: 1,
                minor: 0,
                patch: 1,
                number: 9,
            },
            "https://example/a101rc9",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

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
    let releases = vec![r(
        "a",
        "A",
        Version::Release {
            major: 1,
            minor: 0,
            patch: 0,
        },
        "https://example/a100",
    )];

    let app = routes::releases::router(releases_state_seeded(releases).await);

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
async fn releases_list_json_accept_header_renders_json() {
    let releases = vec![
        r(
            "a",
            "A",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/a100",
        ),
        r(
            "b",
            "B",
            Version::Release {
                major: 2,
                minor: 0,
                patch: 0,
            },
            "https://example/b200",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .header(axum::http::header::ACCEPT, "application/json")
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
        "application/json"
    );

    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "a");
    assert_eq!(arr[1]["name"], "b");
}

#[tokio::test]
async fn releases_list_accept_header_html_wins_over_json() {
    let releases = vec![r(
        "a",
        "A",
        Version::Release {
            major: 1,
            minor: 0,
            patch: 0,
        },
        "https://example/a100",
    )];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .header(axum::http::header::ACCEPT, "application/json, text/html")
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
    let releases = vec![
        r(
            "m",
            "M",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/m100",
        ),
        r(
            "m",
            "M",
            Version::Release {
                major: 2,
                minor: 0,
                patch: 0,
            },
            "https://example/m200",
        ),
        r(
            "other",
            "Other",
            Version::Release {
                major: 9,
                minor: 9,
                patch: 9,
            },
            "https://example/o999",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

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

#[tokio::test]
async fn releases_module_json_accept_header_renders_json_and_orders_versions_desc() {
    let releases = vec![
        r(
            "m",
            "M",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/m100",
        ),
        r(
            "m",
            "M",
            Version::Release {
                major: 2,
                minor: 0,
                patch: 0,
            },
            "https://example/m200",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases/m")
                .header(axum::http::header::ACCEPT, "application/json")
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
        "application/json"
    );

    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 2);

    // Ordering: version desc, so 2.0.0 should appear before 1.0.0.
    assert_eq!(arr[0]["version"]["Release"]["major"], 2);
    assert_eq!(arr[1]["version"]["Release"]["major"], 1);
}

#[tokio::test]
async fn releases_list_csv_default_excludes_milestones() {
    let releases = vec![
        ms(
            "edge",
            "Edge",
            Version::Release {
                major: 9,
                minor: 0,
                patch: 0,
            },
            "https://example/ms900",
        ),
        r(
            "stable",
            "Stable",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/s100",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(Request::builder().uri("/sls/releases").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert_eq!(csv_non_empty_line_count(&body), 1);
    assert!(body.contains("stable"));
    assert!(!body.contains("ms900"));
}

#[tokio::test]
async fn releases_list_csv_rc_true_includes_milestones() {
    let releases = vec![
        ms(
            "edge",
            "Edge",
            Version::Release {
                major: 9,
                minor: 0,
                patch: 0,
            },
            "https://example/ms900",
        ),
        r(
            "stable",
            "Stable",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/s100",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

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
    assert_eq!(csv_non_empty_line_count(&body), 2);
    assert!(body.contains("ms900"));
    assert!(body.contains("s100"));
}

#[tokio::test]
async fn releases_module_csv_default_excludes_milestones() {
    let releases = vec![
        r(
            "m",
            "M",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/m100",
        ),
        ms(
            "m",
            "M",
            Version::Release {
                major: 2,
                minor: 0,
                patch: 0,
            },
            "https://example/m200ms",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(Request::builder().uri("/sls/releases/m").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert_eq!(csv_non_empty_line_count(&body), 1);
    assert!(body.contains("m100"));
    assert!(!body.contains("m200ms"));
}

#[tokio::test]
async fn releases_module_csv_rc_true_includes_milestones_ordered_desc() {
    let releases = vec![
        r(
            "m",
            "M",
            Version::Release {
                major: 1,
                minor: 0,
                patch: 0,
            },
            "https://example/m100",
        ),
        ms(
            "m",
            "M",
            Version::Release {
                major: 2,
                minor: 0,
                patch: 0,
            },
            "https://example/m200ms",
        ),
    ];

    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases/m?rc=true")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert_eq!(csv_non_empty_line_count(&body), 2);
    let first_line = body.lines().find(|l| !l.is_empty()).unwrap();
    assert!(first_line.contains("2.0.0"));
    assert!(body.contains("m200ms"));
}

