use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use sls_releases::persistence::sqlite;
use sls_releases::routes;
use sls_releases::routes::modules::ModulesState;

use super::{body_string, stores_with_releases};

async fn modules_state_with_migrations() -> ModulesState {
    let (store, _) = sqlite::in_memory_stores()
        .await
        .expect("in-memory sqlite");

    ModulesState { store }
}

#[tokio::test]
async fn modules_list_json_includes_seeded_row_and_orders_by_name() {
    let state = modules_state_with_migrations().await;
    let app = routes::modules::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules")
                .header(axum::http::header::ACCEPT, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    let arr = v.as_array().expect("array");
    assert!(!arr.is_empty());

    let bonuses = arr
        .iter()
        .find(|row| row["name"] == "bonuses")
        .expect("seeded module bonuses");
    assert_eq!(bonuses["localized_name"], "Бонусы");

    let names: Vec<&str> = arr
        .iter()
        .map(|row| row["name"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted);
}

#[tokio::test]
async fn modules_list_json_filter_by_name_returns_single_match() {
    let state = modules_state_with_migrations().await;
    let app = routes::modules::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules?name=bonuses")
                .header(axum::http::header::ACCEPT, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    let arr = v.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], "bonuses");
    assert_eq!(arr[0]["localized_name"], "Бонусы");
}

#[tokio::test]
async fn modules_list_json_unknown_name_returns_empty_array() {
    let state = modules_state_with_migrations().await;
    let app = routes::modules::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules?name=no-such-module-xyz")
                .header(axum::http::header::ACCEPT, "application/json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    let v: serde_json::Value = serde_json::from_str(&body).expect("valid JSON");
    let arr = v.as_array().expect("array");
    assert!(arr.is_empty());
}

#[tokio::test]
async fn modules_list_csv_uses_comma_space_and_filter() {
    let state = modules_state_with_migrations().await;
    let app = routes::modules::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules?name=coupons")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert_eq!(body.lines().filter(|l| !l.is_empty()).count(), 1);
    assert!(body.starts_with("coupons, "));
}

#[tokio::test]
async fn modules_list_html_renders_table() {
    let state = modules_state_with_migrations().await;
    let app = routes::modules::router(state);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules?name=gateway")
                .header(axum::http::header::ACCEPT, "text/html")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_string(resp).await;
    assert!(body.contains("<table rules=\"all\">"));
    assert!(body.contains("gateway"));
    assert!(body.contains("Внешний API"));
}

#[tokio::test]
async fn modules_list_persistence_invalid_version_returns_500() {
    use async_trait::async_trait;
    use sls_releases::domain::release::Release;
    use sls_releases::persistence::{Include, PersistenceError, ReleasesStore};

    struct AlwaysFailingStore;

    #[async_trait]
    impl ReleasesStore for AlwaysFailingStore {
        async fn get_all_releases(
            &self,
            _include: &Include,
        ) -> Result<Vec<Release>, PersistenceError> {
            Err(PersistenceError::InvalidVersionKind("test".into()))
        }

        async fn get_releases_by_name(
            &self,
            _name: &str,
            _include: &Include,
        ) -> Result<Vec<Release>, PersistenceError> {
            Err(PersistenceError::InvalidVersionKind("test".into()))
        }

        async fn replace_all_releases(
            &self,
            _releases: Vec<Release>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn list_modules(
            &self,
            _name: Option<&str>,
        ) -> Result<Vec<sls_releases::domain::module::Module>, PersistenceError> {
            Err(PersistenceError::InvalidVersionKind("test".into()))
        }

        async fn get_release(
            &self,
            _version: &sls_releases::domain::release::Version,
        ) -> Result<Release, PersistenceError> {
            Err(PersistenceError::InvalidVersionKind("test".into()))
        }
    }

    let app = routes::modules::router(ModulesState {
        store: stores_with_releases(std::sync::Arc::new(AlwaysFailingStore)),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn modules_list_persistence_sql_error_returns_502() {
    use async_trait::async_trait;
    use sls_releases::domain::release::Release;
    use sls_releases::persistence::{Include, PersistenceError, ReleasesStore};

    struct SqlRowNotFoundStore;

    #[async_trait]
    impl ReleasesStore for SqlRowNotFoundStore {
        async fn get_all_releases(
            &self,
            _include: &Include,
        ) -> Result<Vec<Release>, PersistenceError> {
            Ok(vec![])
        }

        async fn get_releases_by_name(
            &self,
            _name: &str,
            _include: &Include,
        ) -> Result<Vec<Release>, PersistenceError> {
            Ok(vec![])
        }

        async fn replace_all_releases(
            &self,
            _releases: Vec<Release>,
        ) -> Result<(), PersistenceError> {
            Ok(())
        }

        async fn list_modules(
            &self,
            _name: Option<&str>,
        ) -> Result<Vec<sls_releases::domain::module::Module>, PersistenceError> {
            Err(PersistenceError::Sql(sqlx::Error::RowNotFound))
        }

        async fn get_release(
            &self,
            _version: &sls_releases::domain::release::Version,
        ) -> Result<Release, PersistenceError> {
            Err(PersistenceError::Sql(sqlx::Error::RowNotFound))
        }
    }

    let app = routes::modules::router(ModulesState {
        store: stores_with_releases(std::sync::Arc::new(SqlRowNotFoundStore)),
    });

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/sls/modules")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);
}
