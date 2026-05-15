use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use axum::Router;

#[cfg(not(feature = "embedded-web"))]
use axum::http::Uri;

use chrono::Offset;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use sls_releases::clients::github::ReleasesClient;
use sls_releases::clients::github::client::GitHubClient;
use sls_releases::config::{CliConfig, load_config};
use sls_releases::jobs::sync::spawn_periodic_sync;
use sls_releases::persistence::Stores;
use sls_releases::persistence::sqlite;
use sls_releases::routes;
use sls_releases::routes::modules::ModulesState;
use sls_releases::routes::releases::ReleasesState;
use sls_releases::routes::transactions::TransactionsState;

#[derive(Debug, Parser)]
#[command(
    name = "sls-releases",
    version,
    about = "HTTP service for Set Loyalty releases control",
    long_about = "Serves release and transaction endpoints backed by SQLite and GitHub.\n\
\n\
Without --config, the process reads GITHUB_TOKEN from the environment and uses simple defaults \
(see --port and --database). With --config, settings load from a TOML file; GITHUB_TOKEN still \
overrides github.token when set."
)]
struct Cli {
    /// TOML file with [server], [github], [persistence], and [refresh] sections.
    #[arg(short = 'c', long = "config")]
    config: Option<std::path::PathBuf>,
    /// TCP port to listen on when `--config` is not used [default: 8080].
    #[arg(short = 'p', long = "port")]
    port: Option<u16>,
    /// SQLite database file path when `--config` is not used [default: releases.db].
    #[arg(short = 'd', long = "database")]
    database: Option<String>,
    /// Path to web application
    #[arg(short = 'w', long = "web")]
    web: Option<std::path::PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let base_cfg = CliConfig {
        config_path: cli.config,
        server_port: cli.port,
        database: cli.database,
        web: cli.web,
    };
    let cfg = load_config(&base_cfg).context("failed to load config")?;

    let github: Arc<dyn ReleasesClient> = Arc::new(GitHubClient::new(
        cfg.github_token.clone(),
        cfg.github_user_agent.clone(),
    ));

    let stores: Stores = sqlite::connect(&cfg.sqlite_path)
        .await
        .context("failed to open SQLite database")?;

    spawn_periodic_sync(
        github.clone(),
        stores.releases.clone(),
        cfg.refresh_interval_secs,
    );

    let app = Router::new()
        .merge(routes::releases::router(ReleasesState {
            store: stores.clone(),
        }))
        .merge(routes::modules::router(ModulesState {
            store: stores.clone(),
        }))
        .merge(routes::transactions::router(TransactionsState {
            zone_offset: chrono::Local::now().offset().fix(),
        }));

    let web_root = cfg.web_path.map(PathBuf::from);

    #[cfg(feature = "embedded-web")]
    if let Some(_) = web_root {
        Err(anyhow::anyhow!(
            "web path is not supported when embedded-web feature is enabled"
        ))?;
    }

    #[cfg(feature = "embedded-web")]
    let app = app.fallback(axum::routing::get(routes::web::fallback));

    #[cfg(not(feature = "embedded-web"))]
    let app = app.fallback(axum::routing::get(move |uri: Uri| {
        let web_root = web_root.clone();
        async move { routes::web::fallback(uri, web_root).await }
    }));

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], cfg.server_port));
    tracing::info!(%addr, "listening");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind failed ({addr})"))?;
    axum::serve(listener, app).await.context("server failed")?;
    Ok(())
}
