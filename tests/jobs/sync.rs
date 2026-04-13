use std::sync::Arc;

use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::clients::github::ReleasesClient;
use sls_releases::domain::release::Version;
use sls_releases::jobs::sync::sync_releases_once;
use sls_releases::persistence::{ReleasesStore, SqliteReleasesStore};

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
    let store: Arc<dyn ReleasesStore> = Arc::new(sqlite);

    sync_releases_once(&github, &converter, &store).await;

    let mut all = store.get_all_releases().await.expect("read store");
    all.sort_by(|x, y| x.name.cmp(&y.name));

    assert_eq!(all.len(), 2);

    assert_eq!(all[0].name, "a");
    assert_eq!(all[0].localized_name, "A");
    assert_eq!(
        all[0].version,
        Version::Release {
            major: 1,
            minor: 0,
            patch: 0,
        }
    );
    assert_eq!(all[0].url, "https://example/a100");
    assert!(!all[0].date_time.is_empty());

    assert_eq!(all[1].name, "b");
    assert_eq!(all[1].localized_name, "B");
    assert_eq!(
        all[1].version,
        Version::Release {
            major: 2,
            minor: 1,
            patch: 3,
        }
    );
    assert_eq!(all[1].url, "https://example/b213");
    assert!(!all[1].date_time.is_empty());
}
