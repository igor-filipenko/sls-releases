use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use axum::Router;
use chrono::Offset;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use sls_releases::clients::github::client::{Converter, GitHubClient};
use sls_releases::clients::github::ReleasesClient;
use sls_releases::config::load_config_from_path;
use sls_releases::persistence::{ReleasesStore, SqliteReleasesStore};
use sls_releases::routes;
use sls_releases::routes::releases::ReleasesState;
use sls_releases::routes::transactions::TransactionsState;

#[derive(Debug, Parser)]
#[command(name = "sls-releases")]
struct Cli {
    #[arg(short = 'c', long = "config")]
    config: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let cfg = load_config_from_path(cli.config.as_deref()).context("failed to load config")?;

    let github: Arc<dyn ReleasesClient> =
        Arc::new(GitHubClient::new(cfg.github_token.clone(), cfg.github_user_agent.clone()));
    let converter = Arc::new(Converter::new(cfg.sls_modules));

    let sqlite = SqliteReleasesStore::connect(&cfg.sqlite_path)
        .await
        .context("failed to open SQLite database")?;
    let store: Arc<dyn ReleasesStore> = Arc::new(sqlite);

    let github_job = github.clone();
    let converter_job = converter.clone();
    let store_job = store.clone();
    let interval_secs = cfg.refresh_interval_secs;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            match github_job.get_releases(&converter_job).await {
                Ok(releases) => match store_job.replace_all_releases(releases).await {
                    Ok(()) => tracing::debug!("releases snapshot updated"),
                    Err(e) => tracing::warn!(error = %e, "releases refresh failed to persist"),
                },
                Err(e) => tracing::warn!(error = %e, "releases refresh failed to fetch from GitHub"),
            }
        }
    });

    let app = Router::new()
        .merge(routes::releases::router(ReleasesState {
            store: store.clone(),
        }))
        .merge(routes::transactions::router(TransactionsState {
            zone_offset: chrono::Local::now().offset().fix(),
        }));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.server_port));
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind failed ({addr})"))?;
    axum::serve(listener, app)
        .await
        .context("server failed")?;
    Ok(())
}
