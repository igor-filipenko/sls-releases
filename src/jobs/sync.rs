use std::sync::Arc;
use std::time::Duration;

use crate::clients::github::client::Converter;
use crate::clients::github::ReleasesClient;
use crate::persistence::ReleasesStore;

/// One GitHub fetch + SQLite replace (same behavior as the background refresh loop body).
pub async fn sync_releases_once(
    github: &Arc<dyn ReleasesClient>,
    converter: &Converter,
    store: &Arc<dyn ReleasesStore>,
) {
    match github.get_releases(converter).await {
        Ok(releases) => match store.replace_all_releases(releases).await {
            Ok(()) => tracing::debug!("releases snapshot updated"),
            Err(e) => tracing::warn!(error = %e, "releases refresh failed to persist"),
        },
        Err(e) => tracing::warn!(error = %e, "releases refresh failed to fetch from GitHub"),
    }
}

pub fn spawn_periodic_sync(
    github: Arc<dyn ReleasesClient>,
    converter: Arc<Converter>,
    store: Arc<dyn ReleasesStore>,
    interval_secs: u64,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            sync_releases_once(&github, &converter, &store).await;
        }
    });
}
