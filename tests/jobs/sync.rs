use std::sync::Arc;

use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::clients::github::ReleasesClient;
use sls_releases::domain::release::{ReleaseKind, Version};
use sls_releases::jobs::sync::sync_releases_once;
use sls_releases::persistence::{migrations, Include, ReleasesStore, SqliteReleasesStore};

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

async fn stub_milestones_page(server: &MockServer, page: i32, body: serde_json::Value, status: u16) {
    let template = ResponseTemplate::new(status)
        .insert_header("content-type", "application/json")
        .set_body_json(body);

    Mock::given(method("GET"))
        .and(path("/repos/crystalservice/SET10-Loyalty/milestones"))
        .and(query_param("state", "all"))
        .and(query_param("per_page", "100"))
        .and(query_param("page", page.to_string()))
        .respond_with(template)
        .mount(server)
        .await;
}

#[tokio::test]
async fn sync_once_mock_github_writes_sqlite() {
    let server = MockServer::start().await;
    stub_releases_page(
        &server,
        0,
        json!([
            {"tag_name":"a-v1.0.0","html_url":"https://example/a100","created_at":"2026-01-01T00:00:00Z"},
            {"tag_name":"b-v2.1.3","html_url":"https://example/b213","created_at":"2026-01-02T00:00:00Z"}
        ]),
        200,
    )
    .await;
    stub_releases_page(&server, 1, json!([]), 200).await;

    stub_milestones_page(
        &server,
        0,
        json!([
            {"title":"a-v1.2.0","html_url":"https://example/m/a120","created_at":"2026-02-01T00:00:00Z","state":"open"},
            {"title":"b-v9.9.9","html_url":"https://example/m/b999","created_at":"2026-02-02T00:00:00Z","state":"closed"}
        ]),
        200,
    )
    .await;
    stub_milestones_page(&server, 1, json!([]), 200).await;

    let mut known = std::collections::HashMap::new();
    known.insert("a".to_string(), "A".to_string());
    known.insert("b".to_string(), "B".to_string());
    let converter = Converter::new(known);

    let github: Arc<dyn ReleasesClient> = Arc::new(GitHubClient::new_with_base_url(
        "test-token".to_string(),
        server.uri(),
        "test-agent".to_string(),
    ));

    let sqlite = SqliteReleasesStore::in_memory().await.expect("in-memory sqlite");
    migrations::MIGRATOR
        .run(sqlite.pool())
        .await
        .expect("run migrations");
    let store: Arc<dyn ReleasesStore> = Arc::new(sqlite);

    sync_releases_once(&github, &converter, &store).await;

    let mut all = store
        .get_all_releases(&Include::all())
        .await
        .expect("read store");
    all.sort_by(|x, y| x.name.cmp(&y.name));

    assert_eq!(all.len(), 4);

    let a_prod = all.iter().find(|r| r.url == "https://example/a100").unwrap();
    assert_eq!(a_prod.name, "a");
    assert_eq!(a_prod.localized_name, "A");
    assert_eq!(a_prod.kind, ReleaseKind::Production);
    assert_eq!(
        a_prod.version,
        Version::Release {
            major: 1,
            minor: 0,
            patch: 0,
        }
    );
    assert!(!a_prod.date_time.is_empty());
    assert!(!a_prod.closed);

    let a_ms = all.iter().find(|r| r.url == "https://example/m/a120").unwrap();
    assert_eq!(a_ms.name, "a");
    assert_eq!(a_ms.localized_name, "A");
    assert_eq!(a_ms.kind, ReleaseKind::Milestone);
    assert_eq!(
        a_ms.version,
        Version::Release {
            major: 1,
            minor: 2,
            patch: 0,
        }
    );
    assert!(!a_ms.date_time.is_empty());
    assert!(!a_ms.closed);

    let b_prod = all.iter().find(|r| r.url == "https://example/b213").unwrap();
    assert_eq!(b_prod.name, "b");
    assert_eq!(b_prod.localized_name, "B");
    assert_eq!(b_prod.kind, ReleaseKind::Production);
    assert_eq!(
        b_prod.version,
        Version::Release {
            major: 2,
            minor: 1,
            patch: 3,
        }
    );
    assert!(!b_prod.date_time.is_empty());
    assert!(!b_prod.closed);

    let b_ms = all.iter().find(|r| r.url == "https://example/m/b999").unwrap();
    assert_eq!(b_ms.name, "b");
    assert_eq!(b_ms.localized_name, "B");
    assert_eq!(b_ms.kind, ReleaseKind::Milestone);
    assert_eq!(
        b_ms.version,
        Version::Release {
            major: 9,
            minor: 9,
            patch: 9,
        }
    );
    assert!(!b_ms.date_time.is_empty());
    assert!(b_ms.closed);
}
