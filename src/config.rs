use std::collections::HashMap;
use std::env;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub github_token: String,
    pub github_user_agent: String,
    pub sls_modules: HashMap<String, String>,
    pub sqlite_path: String,
    pub refresh_interval_secs: u64,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    server: RawServer,
    #[serde(default)]
    github: RawGithub,
    #[serde(default)]
    sls: RawSls,
    #[serde(default)]
    persistence: RawPersistence,
    #[serde(default)]
    refresh: RawRefresh,
}

#[derive(Debug, Default, Deserialize)]
struct RawServer {
    #[serde(default = "default_port")]
    port: u16,
}

#[derive(Debug, Default, Deserialize)]
struct RawGithub {
    #[serde(default)]
    token: String,
    #[serde(default)]
    user_agent: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawSls {
    #[serde(default)]
    modules: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawPersistence {
    #[serde(default)]
    sqlite_path: String,
}

#[derive(Debug, Default, Deserialize)]
struct RawRefresh {
    #[serde(default)]
    interval_secs: u64,
}

const fn default_port() -> u16 {
    8080
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to load config: {0}")]
    Load(#[from] config::ConfigError),
    #[error("github.token is empty (set it in config file or via GITHUB_TOKEN env var)")]
    MissingGithubToken,
    #[error("persistence.sqlite_path is empty (set path to SQLite database file)")]
    MissingSqlitePath,
    #[error("refresh.interval_secs must be greater than zero")]
    InvalidRefreshInterval,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    load_config_from_path(None)
}

pub fn load_config_from_path(path: Option<&Path>) -> Result<AppConfig, ConfigError> {
    // By default reads `sls.toml` (or other supported extensions) from current directory (repo root).
    let file = match path {
        Some(p) => config::File::from(p).required(true),
        None => config::File::with_name("sls").required(true),
    };

    let c = config::Config::builder().add_source(file).build()?;

    let raw: RawConfig = c.try_deserialize()?;

    // Highest priority: `GITHUB_TOKEN` env var.
    let github_token = env::var("GITHUB_TOKEN").unwrap_or(raw.github.token);
    if github_token.trim().is_empty() {
        return Err(ConfigError::MissingGithubToken);
    }

    let github_user_agent = if raw.github.user_agent.trim().is_empty() {
        format!("sls-releases/{}", env!("CARGO_PKG_VERSION"))
    } else {
        raw.github.user_agent
    };

    let sqlite_path = raw.persistence.sqlite_path.trim().to_string();
    if sqlite_path.is_empty() {
        return Err(ConfigError::MissingSqlitePath);
    }

    let refresh_interval_secs = raw.refresh.interval_secs;
    if refresh_interval_secs == 0 {
        return Err(ConfigError::InvalidRefreshInterval);
    }

    Ok(AppConfig {
        server_port: raw.server.port,
        github_token,
        github_user_agent,
        sls_modules: raw.sls.modules,
        sqlite_path,
        refresh_interval_secs,
    })
}
