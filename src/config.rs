use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub github_token: String,
    pub sls_modules: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    server: RawServer,
    #[serde(default)]
    github: RawGithub,
    #[serde(default)]
    sls: RawSls,
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
}

#[derive(Debug, Default, Deserialize)]
struct RawSls {
    #[serde(default)]
    modules: HashMap<String, String>,
}

const fn default_port() -> u16 {
    8080
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to load config: {0}")]
    Load(#[from] config::ConfigError),
    #[error("github.token is empty (fill it in the config file before running)")]
    MissingGithubToken,
}

pub fn load_config() -> Result<AppConfig, ConfigError> {
    load_config_from_path(None)
}

pub fn load_config_from_path(path: Option<&Path>) -> Result<AppConfig, ConfigError> {
    // By default reads `sls.toml` (or other supported extensions) from current directory (repo root).
    // Token is intentionally NOT sourced from environment variables.
    let file = match path {
        Some(p) => config::File::from(p).required(true),
        None => config::File::with_name("sls").required(true),
    };

    let c = config::Config::builder().add_source(file).build()?;

    let raw: RawConfig = c.try_deserialize()?;
    if raw.github.token.trim().is_empty() {
        return Err(ConfigError::MissingGithubToken);
    }

    Ok(AppConfig {
        server_port: raw.server.port,
        github_token: raw.github.token,
        sls_modules: raw.sls.modules,
    })
}
