use std::collections::HashMap;

use http::header::{ACCEPT, AUTHORIZATION};
use reqwest::header::HeaderMap;
use serde::Deserialize;

use crate::clients::github::parse::parse_tag;
use crate::domain::release::{Release, ReleaseKind, Version};

#[derive(Debug, Clone)]
pub struct GitHubClient {
    token: String,
    base_url: String,
    http: reqwest::Client,
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
        }
    }

    pub async fn get_releases(&self, converter: &Converter) -> Result<Vec<Release>, GitHubError> {
        let mut out = Vec::new();

        // GitHub releases.
        let mut page = 0;
        loop {
            let items = self.get_releases_page(page).await?;
            if items.is_empty() {
                break;
            }
            for ghr in items.iter() {
                if let Some(r) = converter.convert_release(ghr) {
                    out.push(r);
                }
            }
            page += 1;
        }

        // GitHub milestones (all states).
        let mut page = 0;
        loop {
            let items = self.get_milestones_page(page).await?;
            if items.is_empty() {
                break;
            }
            for m in items.iter() {
                if let Some(r) = converter.convert_milestone(m) {
                    out.push(r);
                }
            }
            page += 1;
        }

        Ok(out)
    }

    async fn get_releases_page(&self, page: i32) -> Result<Vec<GitHubRelease>, GitHubError> {
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

        let list: Vec<GitHubRelease> = resp.json().await?;
        tracing::debug!("got {} items from GitHub page {}", list.len(), page);
        Ok(list)
    }

    async fn get_milestones_page(&self, page: i32) -> Result<Vec<GitHubMilestone>, GitHubError> {
        let url = format!(
            "{}/repos/crystalservice/SET10-Loyalty/milestones?state=all&per_page=100&page={page}",
            self.base_url
        );
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
            tracing::debug!(
                "unexpected status from GitHub API: {}, body: {}",
                status.as_u16(),
                body
            );
            return Err(GitHubError::UnexpectedStatus(status.as_u16()));
        }

        let list: Vec<GitHubMilestone> = resp.json().await?;
        tracing::debug!("got {} milestone items from GitHub page {}", list.len(), page);
        Ok(list)
    }
}

#[derive(Debug, Clone)]
pub struct Converter {
    known_modules: HashMap<String, String>,
}

impl Converter {
    pub fn new(known_modules: HashMap<String, String>) -> Self {
        Self { known_modules }
    }

    pub fn convert_release(&self, ghr: &GitHubRelease) -> Option<Release> {
        let (module, version) = parse_tag(&ghr.tag_name)?;
        let localized_name = self.known_modules.get(&module)?.clone();
        let kind = match version {
            Version::Release { .. } => ReleaseKind::Production,
            Version::Candidate { .. } => ReleaseKind::Candidate,
        };

        Some(Release {
            name: module,
            localized_name,
            kind,
            version,
            url: ghr.html_url.clone(),
            date_time: format_publish_time(&ghr.created_at),
            closed: false,
        })
    }

    pub fn convert_milestone(&self, m: &GitHubMilestone) -> Option<Release> {
        let (module, version) = parse_tag(&m.title)?;
        let localized_name = self.known_modules.get(&module)?.clone();
        Some(Release {
            name: module,
            localized_name,
            kind: ReleaseKind::Milestone,
            version,
            url: m.html_url.clone(),
            date_time: format_publish_time(&m.created_at),
            closed: m.state.eq_ignore_ascii_case("closed"),
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

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubMilestone {
    pub title: String,
    #[serde(rename = "html_url")]
    pub html_url: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    /// GitHub returns "open" / "closed".
    pub state: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unexpected github status: {0}")]
    UnexpectedStatus(u16),
}
