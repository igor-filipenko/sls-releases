use std::sync::Arc;

use axum::Router;
use chrono::Offset;
use tracing_subscriber::EnvFilter;

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::config::load_config;
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;
use sls_releases::routes::transactions::TransactionsState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = load_config().expect("failed to load config");

    let github = Arc::new(GitHubClient::new(cfg.github_token));
    let converter = Arc::new(Converter::new(cfg.sls_modules));

    let app = Router::new()
        .merge(routes::releases::router(ReleasesState {
            github,
            converter,
        }))
        .merge(routes::transactions::router(TransactionsState {
            zone_offset: chrono::Local::now().offset().fix(),
        }));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.server_port));
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app).await.expect("server failed");
}
