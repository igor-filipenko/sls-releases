use std::sync::Arc;

use sqlx::{Pool, Sqlite, sqlite::{SqliteConnectOptions, SqlitePoolOptions}};

use crate::persistence::{PersistenceConnectionError, PersistenceError, Stores, sqlite::{jobs::SqliteJobsStore, releases::SqliteReleasesStore}};

mod releases;
mod jobs;

pub async fn connect(sqlite_path: &str) -> Result<Stores, PersistenceConnectionError> {
    let opts = SqliteConnectOptions::new()
    .filename(sqlite_path)
    .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect_with(opts)
    .await?;
    sqlx::query("PRAGMA journal_mode = WAL;")
    .execute(&pool)
    .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
    .execute(&pool)
    .await?;
    let migrator = sqlx::migrate!();
    migrator.run(&pool).await?;
    Ok(Stores {
        releases: Arc::new(SqliteReleasesStore::new(pool.clone())),
        jobs: Arc::new(SqliteJobsStore::new(pool.clone())),
    })
}

/// In-memory DB for tests (`sqlite::memory:`).
pub async fn in_memory() -> Result<Pool<Sqlite>, PersistenceError> {
    let pool = SqlitePoolOptions::new()
    .max_connections(1)
    .connect("sqlite::memory:")
    .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
    .execute(&pool)
    .await?;
    Ok(pool)
}

/// In-memory SQLite with migrations applied; returns stores and pool (for test seeding).
pub async fn in_memory_stores() -> Result<(Stores, Pool<Sqlite>), PersistenceConnectionError> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    let stores = Stores {
        releases: Arc::new(SqliteReleasesStore::new(pool.clone())),
        jobs: Arc::new(SqliteJobsStore::new(pool.clone())),
    };
    Ok((stores, pool))
}
