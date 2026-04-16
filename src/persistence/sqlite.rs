use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use tracing::log;
use crate::domain::release::{Release, ReleaseKind};
use crate::persistence::{Include, PersistenceError, version_from_row, version_kind_db_str, version_parts};

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
        Ok(Self { pool })
    }

    /// In-memory DB for tests (`sqlite::memory:`).
    pub async fn in_memory() -> Result<Self, PersistenceError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn get_all_releases(
        &self,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        let mut sql = String::from(
            r#"SELECT name, localized_name, url, date_time, version_kind, major, minor, patch, rc_number, closed
               FROM releases"#,
        );
        let mut filters = Vec::new();
        if !include.candidates {
            filters.push("version_kind != 'candidate'");
        }
        if !include.milestones {
            filters.push("version_kind != 'milestone'");
        }
        if !filters.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&filters.join(" AND "));
        }
        sql.push_str(" ORDER BY name ASC");

        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
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
            out.push(Release {
                name: row.try_get("name")?,
                localized_name: row.try_get("localized_name")?,
                kind: kind_enum,
                version,
                url: row.try_get("url")?,
                date_time: row.try_get("date_time")?,
                closed,
            });
        }
        Ok(out)
    }

    pub async fn get_releases_by_name(
        &self,
        name: &str,
        include: &Include,
    ) -> Result<Vec<Release>, PersistenceError> {
        let mut sql = String::from(
            r#"SELECT name, localized_name, url, date_time, version_kind, major, minor, patch, rc_number, closed
               FROM releases
               WHERE name = ?"#,
        );
        if !include.candidates {
            sql.push_str(" AND version_kind != 'candidate'");
        }
        if !include.milestones {
            sql.push_str(" AND version_kind != 'milestone'");
        }
        sql.push_str(" ORDER BY major ASC, minor ASC, patch ASC, date_time ASC");

        let rows = sqlx::query(&sql).bind(name).fetch_all(&self.pool).await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
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
            out.push(Release {
                name: row.try_get("name")?,
                localized_name: row.try_get("localized_name")?,
                kind: kind_enum,
                version,
                url: row.try_get("url")?,
                date_time: row.try_get("date_time")?,
                closed,
            });
        }
        Ok(out)
    }

    pub async fn replace_all_releases(&self, releases: Vec<Release>) -> Result<(), PersistenceError> {
        let count = releases.len();
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM releases")
            .execute(&mut *tx)
            .await?;

        for r in releases {
            let (kind, major, minor, patch, rc) = version_parts(&r);
            let kind = version_kind_db_str(kind);
            let closed = matches!(r.kind, ReleaseKind::Milestone) && r.closed;
            sqlx::query(
                r#"INSERT INTO releases
                   (name, localized_name, url, date_time, version_kind, major, minor, patch, rc_number, closed)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                   ON CONFLICT DO UPDATE SET
                     localized_name = excluded.localized_name,
                     url = excluded.url,
                     date_time = excluded.date_time,
                     closed = excluded.closed"#,
            )
            .bind(&r.name)
            .bind(&r.localized_name)
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
        }

        tx.commit().await?;
        log::info!("Updated {} releases", count);
        Ok(())
    }
}
