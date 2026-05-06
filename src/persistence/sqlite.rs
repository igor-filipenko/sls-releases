use std::collections::HashSet;

use crate::domain::module::Module;
use crate::domain::release::{Release, ReleaseKind};
use crate::persistence::{
    Include, PersistenceError, version_from_row, version_kind_db_str, version_parts,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Row, Sqlite};
use tracing::log;

#[derive(Clone)]
pub struct SqliteReleasesStore {
    pool: SqlitePool,
}

impl SqliteReleasesStore {
    pub async fn connect(sqlite_path: &str) -> Result<Self, PersistenceError> {
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
        Ok(Self { pool })
    }

    /// In-memory DB for tests (`sqlite::memory:`).
    pub async fn in_memory() -> Result<Self, PersistenceError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn list_modules(&self, name: Option<&str>) -> Result<Vec<Module>, PersistenceError> {
        get_modules(&self.pool, name).await
    }

    pub async fn get_all_releases(
        &self,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        let mut sql = String::from(
            r#"SELECT r.name, m.localized_name, r.url, r.date_time, r.version_kind, r.major, r.minor, r.patch, r.rc_number, r.closed
               FROM releases r
               INNER JOIN modules m ON m.name = r.name
               WHERE NOT r.closed
            "#,
        );
        let mut filters = Vec::new();
        if !include.candidates {
            filters.push("r.version_kind != 'candidate'");
        }
        if !include.milestones {
            filters.push("r.version_kind != 'milestone'");
        }
        if !filters.is_empty() {
            sql.push_str(" AND ");
            sql.push_str(&filters.join(" AND "));
        }
        sql.push_str(" ORDER BY r.name ASC");
        log::debug!("get_all_releases, sql: {}", sql);

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;
        let releases = rows_to_releases(&rows)?;
        Ok(releases)
    }

    pub async fn get_releases_by_name(
        &self,
        name: &str,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        let mut sql = String::from(
            r#"SELECT r.name, m.localized_name, r.url, r.date_time, r.version_kind, r.major, r.minor, r.patch, r.rc_number, r.closed
               FROM releases r
               INNER JOIN modules m ON m.name = r.name
               WHERE r.name = ?
                 AND NOT r.closed
            "#,
        );
        if !include.candidates {
            sql.push_str(" AND r.version_kind != 'candidate'");
        }
        if !include.milestones {
            sql.push_str(" AND r.version_kind != 'milestone'");
        }
        sql.push_str(" ORDER BY r.major ASC, r.minor ASC, r.patch ASC, r.date_time ASC");
        log::debug!("get_releases_by_name, sql: {}", sql);

        let rows = sqlx::query(&sql).bind(name).fetch_all(&self.pool).await?;

        let releases = rows_to_releases(&rows)?;
        Ok(releases)
    }

    pub async fn replace_all_releases(
        &self,
        releases: Vec<Release>,
    ) -> Result<(), PersistenceError> {
        let mut tx = self.pool.begin().await?;

        let modules = get_modules(&mut *tx, None).await?;
        let known: HashSet<String> = modules.into_iter().map(|m| m.name).collect();

        let sql = r#"INSERT INTO releases
                   (name, url, date_time, version_kind, major, minor, patch, rc_number, closed)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                   ON CONFLICT(name, version_kind, major, minor, patch, rc_number) DO UPDATE SET
                     url = excluded.url,
                     date_time = excluded.date_time,
                     closed = excluded.closed
                   WHERE releases.url != excluded.url
                      OR releases.date_time != excluded.date_time
                      OR releases.closed != excluded.closed"#;
        let mut processed = 0usize;
        let mut changed = 0usize;
        for r in releases {
            if !known.contains(&r.name) {
                log::trace!("skipping unknown module: {}", r.name);
                continue;
            }

            let (kind, major, minor, patch, rc) = version_parts(&r);
            let kind = version_kind_db_str(kind);
            let closed = matches!(r.kind, ReleaseKind::Milestone) && r.closed;
            let result = sqlx::query(sql)
                .bind(&r.name)
                .bind(&r.url)
                .bind(&r.date_time)
                .bind(kind)
                .bind(major)
                .bind(minor)
                .bind(patch)
                .bind(rc.unwrap_or(-1))
                .bind(closed)
                .execute(&mut *tx)
                .await?;
            processed += 1;
            if result.rows_affected() > 0 {
                changed += 1;
            }
        }

        tx.commit().await?;
        log::info!(
            "Processed {} releases, changed {} releases",
            processed,
            changed
        );
        Ok(())
    }
}

fn rows_to_releases(rows: &Vec<SqliteRow>) -> Result<Vec<Release>, PersistenceError> {
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let rel = row_to_release(row)?;
        out.push(rel);
    }
    Ok(out)
}

async fn get_modules<'e, E>(
    executor: E,
    name: Option<&str>,
) -> Result<Vec<Module>, PersistenceError>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    let rows = match name {
        Some(n) => {
            let sql = "SELECT name, localized_name FROM modules WHERE name = ? ORDER BY name ASC";
            log::debug!("get_modules, sql: {}", sql);
            sqlx::query(sql).bind(n).fetch_all(executor).await?
        }
        None => {
            let sql = "SELECT name, localized_name FROM modules ORDER BY name ASC";
            log::debug!("get_modules, sql: {}", sql);
            sqlx::query(sql).fetch_all(executor).await?
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(Module {
            name: row.try_get("name")?,
            localized_name: row.try_get("localized_name")?,
        });
    }
    log::debug!("get_modules, out: {:?}", out.len());
    Ok(out)
}

fn row_to_release(row: &SqliteRow) -> Result<Release, PersistenceError> {
    let kind: String = row.try_get("version_kind")?;
    let kind_enum = match kind.as_str() {
        "milestone" => ReleaseKind::Milestone,
        "production" | "release" => ReleaseKind::Production,
        "candidate" => ReleaseKind::Candidate,
        other => return Err(PersistenceError::InvalidVersionKind(other.to_string())),
    };
    let major: i32 = row.try_get("major")?;
    let minor: i32 = row.try_get("minor")?;
    let patch: i32 = row.try_get("patch")?;
    let rc_raw: i32 = row.try_get("rc_number")?;
    let rc = (rc_raw >= 0).then_some(rc_raw);
    let version = version_from_row(&kind, major, minor, patch, rc)?;
    let mut closed: bool = row.try_get("closed")?;
    if kind_enum != ReleaseKind::Milestone {
        closed = false;
    }

    Ok(Release {
        name: row.try_get("name")?,
        localized_name: row.try_get("localized_name")?,
        kind: kind_enum,
        version,
        url: row.try_get("url")?,
        date_time: row.try_get("date_time")?,
        closed,
    })
}
