use std::collections::HashSet;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use tower::ServiceExt;

use sls_releases::domain::job::Job;
use sls_releases::domain::release::Release;
use sls_releases::domain::release::ReleaseKind;
use sls_releases::domain::release::Version;
use sls_releases::persistence::{Include, PersistenceError, ReleasesStore, sqlite};
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;

use super::{body_string, csv_non_empty_line_count, stores_with_releases};

async fn releases_state_seeded(releases: Vec<Release>) -> ReleasesState {
    let (stores, pool) = sqlite::in_memory_stores().await.expect("in-memory sqlite");

    let mut seen: HashSet<&str> = HashSet::new();
    for rel in &releases {
        if seen.insert(rel.name.as_str()) {
            sqlx::query("INSERT OR REPLACE INTO modules (name, localized_name) VALUES (?, ?)")
                .bind(&rel.name)
                .bind(&rel.localized_name)
                .execute(&pool)
                .await
                .expect("seed module for test");
        }
    }

    stores
        .releases
        .replace_all_releases(releases)
        .await
        .expect("seed store");

    ReleasesState { store: stores }
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
    ms_with_closed(name, localized_name, version, url, false)
}

fn ms_closed(name: &str, localized_name: &str, version: Version, url: &str) -> Release {
    ms_with_closed(name, localized_name, version, url, true)
}

fn ms_with_closed(
    name: &str,
    localized_name: &str,
    version: Version,
    url: &str,
    closed: bool,
) -> Release {
    Release {
        name: name.to_string(),
        localized_name: localized_name.to_string(),
        kind: ReleaseKind::Milestone,
        version,
        url: url.to_string(),
        date_time: "2026-01-01T00:00:00Z".to_string(),
        closed,
    }
}

fn milestone_tag(module: &str, version: &Version) -> String {
    format!("{module}-v{version}")
}

fn candidate_tag(module: &str, version: &Version) -> String {
    format!("{module}-v{version}")
}

async fn delete_release(app: &axum::Router, tag: &str) -> axum::response::Response {
    let body = serde_json::json!({ "tag": tag });
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/sls/releases")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn post_create_release(
    app: &axum::Router,
    milestone: &str,
    candidate: bool,
    description: Option<&str>,
) -> axum::response::Response {
    let body = serde_json::json!({
        "milestone": milestone,
        "candidate": candidate,
        "description": description,
    });
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/sls/releases")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap()
}

struct AlwaysFailingStore;

#[async_trait]
impl ReleasesStore for AlwaysFailingStore {
    async fn get_all_releases(&self, _include: &Include) -> Result<Vec<Release>, PersistenceError> {
        Err(PersistenceError::InvalidVersionKind("test".into()))
    }

    async fn get_releases_by_name(
        &self,
        _name: &str,
        _include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        Err(PersistenceError::InvalidVersionKind("test".into()))
    }

    async fn replace_all_releases(&self, _releases: Vec<Release>) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn list_modules(
        &self,
        _name: Option<&str>,
    ) -> Result<Vec<sls_releases::domain::module::Module>, PersistenceError> {
        Err(PersistenceError::InvalidVersionKind("test".into()))
    }

    async fn get_release(&self, _version: &Version) -> Result<Release, PersistenceError> {
        Err(PersistenceError::InvalidVersionKind("test".into()))
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
async fn releases_list_persistence_invalid_version_returns_500() {
    let app = routes::releases::router(ReleasesState {
        store: stores_with_releases(std::sync::Arc::new(AlwaysFailingStore)),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

struct SqlRowNotFoundStore;

#[async_trait]
impl ReleasesStore for SqlRowNotFoundStore {
    async fn get_all_releases(&self, _include: &Include) -> Result<Vec<Release>, PersistenceError> {
        Err(PersistenceError::Sql(sqlx::Error::RowNotFound))
    }

    async fn get_releases_by_name(
        &self,
        _name: &str,
        _include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        Err(PersistenceError::Sql(sqlx::Error::RowNotFound))
    }

    async fn replace_all_releases(&self, _releases: Vec<Release>) -> Result<(), PersistenceError> {
        Ok(())
    }

    async fn list_modules(
        &self,
        _name: Option<&str>,
    ) -> Result<Vec<sls_releases::domain::module::Module>, PersistenceError> {
        Ok(vec![])
    }

    async fn get_release(&self, _version: &Version) -> Result<Release, PersistenceError> {
        Err(PersistenceError::Sql(sqlx::Error::RowNotFound))
    }
}

#[tokio::test]
async fn releases_list_persistence_sql_error_returns_502() {
    let app = routes::releases::router(ReleasesState {
        store: stores_with_releases(std::sync::Arc::new(SqlRowNotFoundStore)),
    });

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
async fn releases_module_persistence_sql_error_returns_502() {
    let app = routes::releases::router(ReleasesState {
        store: stores_with_releases(std::sync::Arc::new(SqlRowNotFoundStore)),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/releases/foo")
                .body(Body::empty())
                .unwrap(),
        )
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
    assert_eq!(arr[0]["name"], "m");
    assert_eq!(arr[0]["localized_name"], "M");
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
        .oneshot(
            Request::builder()
                .uri("/sls/releases")
                .body(Body::empty())
                .unwrap(),
        )
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
    assert_eq!(csv_non_empty_line_count(&body), 1);
    assert!(!body.contains("ms900"));
    assert!(body.contains("s100"));
}

#[tokio::test]
async fn releases_list_csv_ms_true_includes_milestones() {
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
                .uri("/sls/releases?ms=true")
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
    assert_eq!(csv_non_empty_line_count(&body), 1);
    assert!(body.contains("m100"));
    assert!(!body.contains("m200ms"));
}

#[tokio::test]
async fn releases_module_csv_ms_true_includes_milestones_ordered_desc() {
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
                .uri("/sls/releases/m?ms=true")
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

#[tokio::test]
async fn create_release_not_exists_milestone_returns_400() {
    let app = routes::releases::router(releases_state_seeded(vec![]).await);

    let resp = post_create_release(
        &app,
        &milestone_tag(
            "m",
            &Version::Release {
                major: 9,
                minor: 0,
                patch: 0,
            },
        ),
        false,
        None,
    )
    .await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_release_version_not_milestone_returns_400() {
    let version = Version::Release {
        major: 1,
        minor: 0,
        patch: 0,
    };
    let releases = vec![r("m", "M", version.clone(), "https://example/m100")];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = post_create_release(&app, &milestone_tag("m", &version), false, None).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_release_closed_milestone_returns_400() {
    let version = Version::Release {
        major: 2,
        minor: 0,
        patch: 0,
    };
    let releases = vec![ms_closed(
        "m",
        "M",
        version.clone(),
        "https://example/m200ms",
    )];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = post_create_release(&app, &milestone_tag("m", &version), false, None).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_job_returns_pending_status() {
    let version = Version::Release {
        major: 1,
        minor: 0,
        patch: 0,
    };
    let releases = vec![ms("m", "M", version.clone(), "https://example/m100ms")];
    let state = releases_state_seeded(releases).await;
    let job_id = "job-test-001".to_string();
    state
        .store
        .jobs
        .create_job(&Job::CreateRelease {
            id: job_id.clone(),
            milestone: milestone_tag("m", &version),
            candidate: false,
            description: Some("release notes".into()),
        })
        .await
        .expect("seed job");

    let app = routes::releases::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["id"], job_id);
    assert_eq!(v["status"], "Pending");
    assert!(v["errorCode"].is_null());
    assert!(v["errorDetail"].is_null());
}

#[tokio::test]
async fn delete_release_invalid_tag_returns_400() {
    let app = routes::releases::router(releases_state_seeded(vec![]).await);

    let resp = delete_release(&app, "not-a-valid-tag").await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_release_not_found_returns_400() {
    let version = Version::Candidate {
        major: 1,
        minor: 0,
        patch: 0,
        number: 1,
    };
    let app = routes::releases::router(releases_state_seeded(vec![]).await);

    let resp = delete_release(&app, &candidate_tag("m", &version)).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_release_production_returns_400() {
    let version = Version::Release {
        major: 1,
        minor: 0,
        patch: 0,
    };
    let releases = vec![r("m", "M", version.clone(), "https://example/m100")];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = delete_release(&app, &milestone_tag("m", &version)).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_release_milestone_returns_400() {
    let version = Version::Release {
        major: 2,
        minor: 0,
        patch: 0,
    };
    let releases = vec![ms("m", "M", version.clone(), "https://example/m200ms")];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = delete_release(&app, &milestone_tag("m", &version)).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_release_module_mismatch_returns_400() {
    let version = Version::Candidate {
        major: 1,
        minor: 0,
        patch: 0,
        number: 1,
    };
    let releases = vec![r("m", "M", version.clone(), "https://example/m-rc1")];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = delete_release(&app, &candidate_tag("other", &version)).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_release_candidate_returns_pending_job() {
    let version = Version::Candidate {
        major: 1,
        minor: 0,
        patch: 0,
        number: 2,
    };
    let tag = candidate_tag("m", &version);
    let releases = vec![r("m", "M", version.clone(), "https://example/m-rc2")];
    let app = routes::releases::router(releases_state_seeded(releases).await);

    let resp = delete_release(&app, &tag).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    assert_eq!(v["status"], "Pending");
    assert!(v["errorCode"].is_null());
    assert!(v["errorDetail"].is_null());
    let job_id = v["id"].as_str().expect("job id");

    let job_resp = app
        .oneshot(
            Request::builder()
                .uri(format!("/sls/jobs/{job_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(job_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn get_job_not_exists_returns_400() {
    let app = routes::releases::router(releases_state_seeded(vec![]).await);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/jobs/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
