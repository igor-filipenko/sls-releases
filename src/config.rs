use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::string::ToString;
use serde::Deserialize;

///
/// Environment variable name for GitHub token.
/// 
const ENV_GITHUB_TOKEN: &str = "GITHUB_TOKEN";

///
/// Default user agent for GitHub API requests.
/// 
const DEFAULT_USER_AGENT_TEMPLATE: &str = "sls-releases/{version}";

///
/// Default server port.
/// 
const DEFAULT_PORT: u16 = 8080;


#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server_port: u16,
    pub github_token: String,
    pub github_user_agent: String,
    pub sls_modules: HashMap<String, String>,
    pub sqlite_path: String,
    pub refresh_interval_secs: u64,
}

pub struct CliConfig {
    pub config_path: Option<std::path::PathBuf>,
    pub server_port: Option<u16>,
    pub database: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FileConfig {
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

#[derive(Debug, Deserialize)]
struct RawServer {
    #[serde(default = "default_port")]
    port: u16,
}

impl Default for RawServer {
    fn default() -> Self {
        Self {
            port: default_port(),
        }
    }
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

pub fn load_config(cli: &CliConfig) -> Result<AppConfig, ConfigError> {
    if let Some(config_path) = &cli.config_path {
        load_config_from_path(config_path.as_path())
    } else {
        Ok(AppConfig {
            server_port: cli.server_port.unwrap_or(8080),
            github_token: env::var(ENV_GITHUB_TOKEN)
                .map_err(|_| ConfigError::MissingGithubToken)?,
            github_user_agent: default_user_agent(),
            sls_modules: HashMap::new(),
            sqlite_path: get_database_path(&cli.database)?,
            refresh_interval_secs: 300,
        })
    }
}

fn load_config_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    let file = config::File::from(path).required(true);
    let c = config::Config::builder().add_source(file).build()?;

    let raw: FileConfig = c.try_deserialize()?;

    // Highest priority: `GITHUB_TOKEN` env var.
    let github_token = env::var(ENV_GITHUB_TOKEN).unwrap_or(raw.github.token);
    if github_token.trim().is_empty() {
        return Err(ConfigError::MissingGithubToken);
    }

    let github_user_agent = if raw.github.user_agent.trim().is_empty() {
        default_user_agent()
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

fn get_database_path(path: &Option<String>) -> Result<String, ConfigError> {
    match path {
        None => Ok("releases.db".to_string()),
        Some(s) => validate_sqlite_path(s),
    }
}

fn validate_sqlite_path(path: &str) -> Result<String, ConfigError> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::MissingSqlitePath);
    }
    Ok(trimmed.to_string())
}

fn default_user_agent() -> String {
    format!("{}", DEFAULT_USER_AGENT_TEMPLATE.replace("{}", env!("CARGO_PKG_VERSION")))
}

const fn default_port() -> u16 {
    DEFAULT_PORT
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_github_token(token: &str, f: impl FnOnce()) {
        let _lock = ENV_LOCK.lock().expect("env test lock poisoned");
        unsafe {
            std::env::set_var(ENV_GITHUB_TOKEN, token);
        }
        struct ClearToken;
        impl Drop for ClearToken {
            fn drop(&mut self) {
                unsafe {
                    std::env::remove_var(ENV_GITHUB_TOKEN);
                }
            }
        }
        let _clear = ClearToken;
        f();
    }

    fn without_github_token(f: impl FnOnce()) {
        let _lock = ENV_LOCK.lock().expect("env test lock poisoned");
        unsafe {
            std::env::remove_var(ENV_GITHUB_TOKEN);
        }
        f();
    }

    #[test]
    fn cli_only_uses_defaults_when_no_config_file() {
        with_github_token("cli-test-token", || {
            let cli = CliConfig {
                config_path: None,
                server_port: None,
                database: None,
            };
            let cfg = load_config(&cli).expect("load_config");
            assert_eq!(cfg.server_port, DEFAULT_PORT);
            assert_eq!(cfg.github_token, "cli-test-token");
            assert_eq!(cfg.github_user_agent, default_user_agent());
            assert!(cfg.sls_modules.is_empty());
            assert_eq!(cfg.sqlite_path, "releases.db");
            assert_eq!(cfg.refresh_interval_secs, 300);
        });
    }

    #[test]
    fn cli_only_respects_port_flag() {
        with_github_token("t", || {
            let cli = CliConfig {
                config_path: None,
                server_port: Some(9000),
                database: None,
            };
            let cfg = load_config(&cli).expect("load_config");
            assert_eq!(cfg.server_port, 9000);
        });
    }

    #[test]
    fn cli_only_errors_when_github_token_missing() {
        without_github_token(|| {
            let cli = CliConfig {
                config_path: None,
                server_port: None,
                database: None,
            };
            let err = load_config(&cli).expect_err("expected missing token");
            assert!(matches!(err, ConfigError::MissingGithubToken));
        });
    }

    #[test]
    fn config_from_file_applies_section_defaults() {
        // Clear `GITHUB_TOKEN` so file token is used (also serializes with other env tests).
        without_github_token(|| {
            let db = NamedTempFile::with_suffix(".db").expect("temp db");
            let db_path = db.path().to_str().expect("utf-8 temp db path");

            let mut file = NamedTempFile::with_suffix(".toml").expect("temp config");
            write!(
                file,
                r#"
[github]
token = "from-file"

[persistence]
sqlite_path = "{db_path}"

[refresh]
interval_secs = 60
"#,
                db_path = db_path,
            )
            .expect("write config");
            file.flush().expect("flush");

            let cli = CliConfig {
                config_path: Some(file.path().to_path_buf()),
                server_port: None,
                database: None,
            };
            let cfg = load_config(&cli).expect("load_config");
            assert_eq!(cfg.server_port, DEFAULT_PORT, "server.port defaults");
            assert_eq!(cfg.github_token, "from-file");
            assert_eq!(cfg.github_user_agent, default_user_agent(), "empty user_agent defaults");
            assert!(cfg.sls_modules.is_empty());
            assert_eq!(cfg.sqlite_path, db_path);
            assert_eq!(cfg.refresh_interval_secs, 60);
        });
    }

}
