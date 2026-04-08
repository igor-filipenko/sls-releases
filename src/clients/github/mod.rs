pub mod client;
pub mod parse;

use std::future::Future;
use std::pin::Pin;

use crate::clients::github::client::{Converter, GitHubClient, GitHubError};
use crate::domain::release::Release;

pub trait ReleasesClient: Send + Sync {
    fn get_releases<'a>(
        &'a self,
        converter: &'a Converter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, GitHubError>> + Send + 'a>>;
}

impl ReleasesClient for GitHubClient {
    fn get_releases<'a>(
        &'a self,
        converter: &'a Converter,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Release>, GitHubError>> + Send + 'a>> {
        Box::pin(async move { GitHubClient::get_releases(self, converter).await })
    }
}
