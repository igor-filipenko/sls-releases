pub mod client;
pub mod parse;

use async_trait::async_trait;

use crate::clients::github::client::{Converter, GitHubClient, GitHubError};
use crate::domain::release::Release;

#[async_trait]
pub trait ReleasesClient: Send + Sync {
    async fn get_releases(&self, converter: &Converter) -> Result<Vec<Release>, GitHubError>;
}

#[async_trait]
impl ReleasesClient for GitHubClient {
    async fn get_releases(&self, converter: &Converter) -> Result<Vec<Release>, GitHubError> {
        GitHubClient::get_releases(self, converter).await
    }
}
