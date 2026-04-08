use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use http::header::{ACCEPT, AUTHORIZATION, CACHE_CONTROL};
use moka::future::Cache;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::clients::github::parse::parse_tag;
use crate::domain::release::Release;

#[derive(Debug, Clone)]
struct CachedPage {
    value: Arc<Vec<GitHubRelease>>,
    expires_at: Instant,
}

#[derive(Debug, Clone)]
pub struct GitHubClient {
    token: String,
    base_url: String,
    http: reqwest::Client,
    page_cache: Cache<i32, CachedPage>,
}

impl GitHubClient {
    pub fn new(token: String, user_agent: String) -> Self {
        Self::new_with_base_url(token, "https://api.github.com", user_agent)
    }

    pub fn new_with_base_url(token: String, base_url: impl Into<String>, user_agent: String) -> Self {
        let base_url = base_url.into();
        let base_url = base_url.trim_end_matches('/').to_string();
        Self {
            token,
            base_url,
            http: reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .expect("failed to build reqwest client"),
            page_cache: Cache::new(1024),
        }
    }

    pub async fn get_releases(&self, converter: &Converter) -> Result<Vec<Release>, GitHubError> {
        let mut page = 0;
        let mut out = Vec::new();
        loop {
            let items = self.get_page(page).await?;
            if items.is_empty() {
                break;
            }
            for ghr in items.iter() {
                if let Some(r) = converter.convert(ghr) {
                    out.push(r);
                }
            }
            page += 1;
        }
        Ok(out)
    }

    async fn get_page(&self, page: i32) -> Result<Arc<Vec<GitHubRelease>>, GitHubError> {
        let now = Instant::now();
        if let Some(cached) = self.page_cache.get(&page).await {
            if now < cached.expires_at {
                return Ok(cached.value);
            }
            self.page_cache.invalidate(&page).await;
        }

        let url = format!("{}/repos/crystalservice/SET10-Loyalty/releases?per_page=100&page={page}", self.base_url);
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/vnd.github+json".parse().unwrap());
        headers.insert(
            AUTHORIZATION,
            format!("Bearer {}", self.token).parse().unwrap(),
        );
        headers.insert("X-GitHub-Api-Version", "2022-11-28".parse().unwrap());

        let resp = self.http.get(url).headers(headers).send().await?;
        let status = resp.status();
        if status.as_u16() != 200 {
            let body = resp.text().await.unwrap_or_default();
            tracing::debug!("unexpected status from GitHub API: {}, body: {}", status.as_u16(), body);
            return Err(GitHubError::UnexpectedStatus(status.as_u16()));
        }

        let max_age = resp
            .headers()
            .get(CACHE_CONTROL)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_max_age_seconds)
            .unwrap_or(60);
        let expires_at = now + Duration::from_secs(max_age);

        let list: Vec<GitHubRelease> = resp.json().await?;
        tracing::debug!("got {:?} items, expires at {:?}", list.len(), expires_at);
        
        let value = Arc::new(list);
        self.page_cache
            .insert(
                page,
                CachedPage {
                    value: value.clone(),
                    expires_at,
                },
            )
            .await;
        Ok(value)
    }
}

fn parse_max_age_seconds(cache_control: &str) -> Option<u64> {
    // Mirror Kotlin: Regex("max-age=(\\d+)").find(it)
    static RE_STR: &str = r"max-age=(\d+)";
    let re = Regex::new(RE_STR).ok()?;
    let caps = re.captures(cache_control)?;
    caps.get(1)?.as_str().parse().ok()
}

#[derive(Debug, Clone)]
pub struct Converter {
    known_modules: HashMap<String, String>,
}

impl Converter {
    pub fn new(known_modules: HashMap<String, String>) -> Self {
        Self { known_modules }
    }

    pub fn convert(&self, ghr: &GitHubRelease) -> Option<Release> {
        let (module, version) = parse_tag(&ghr.tag_name)?;
        let localized_name = self.known_modules.get(&module)?.clone();

        Some(Release {
            name: module,
            localized_name,
            version,
            url: ghr.html_url.clone(),
            date_time: format_publish_time(&ghr.created_at),
        })
    }
}

fn format_publish_time(created_at: &str) -> String {
    // Kotlin formatter: "MMM d, yyyy 'at' h:mm a" with system zone + default locale.
    // Chrono doesn't handle OS locale in formatting; we output English month abbreviations.
    match chrono::DateTime::parse_from_rfc3339(created_at) {
        Ok(dt) => {
            let local: chrono::DateTime<chrono::Local> = chrono::DateTime::from(dt);
            local.format("%b %-d, %Y at %-I:%M %p").to_string()
        }
        Err(_) => created_at.to_string(),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRelease {
    #[serde(rename = "tag_name")]
    pub tag_name: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected github status: {0}")]
    UnexpectedStatus(u16),
}
