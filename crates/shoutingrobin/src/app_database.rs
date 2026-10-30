use gpui_kit::{App, Global};
use sqlx::Row;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppDatabase {
    /// The catalog: the list of crawls, site settings and app settings.
    pool: SqlitePool,
    /// Each crawl's own database.
    crawls: crate::storage::CrawlStore,
}

/// Turns off SQLite's allocation statistics for the whole process.
///
/// With them on, every allocation SQLite makes takes one lock shared by all
/// connections, so the reads that open a crawl run one at a time however many
/// connections they use: on a 400k row crawl that lock alone cost 7 seconds.
/// Nothing here reads the statistics. SQLite only accepts the setting before
/// it initialises, which the first connection does, so this runs before any
/// pool opens.
pub fn disable_sqlite_memory_statistics() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // SAFETY: `sqlite3_config` is only unsafe because it must not race with
        // SQLite starting up or with another call to it. `Once` rules out the
        // second, and every caller runs before the first connection opens.
        let code =
            unsafe { libsqlite3_sys::sqlite3_config(libsqlite3_sys::SQLITE_CONFIG_MEMSTATUS, 0) };
        if code != libsqlite3_sys::SQLITE_OK {
            tracing::warn!(code, "SQLite kept its memory statistics on");
        }
    });
}

pub struct UserSetting {
    pub theme: String,
}

impl Global for AppDatabase {}

impl AppDatabase {
    pub async fn new() -> Result<Self, sqlx::Error> {
        disable_sqlite_memory_statistics();
        let db_path = Self::app_db_path();
        let crawls = crate::storage::CrawlStore::new(db_path.with_file_name("crawls"));

        let pool = crate::storage::connect_catalog(&db_path).await?;

        let mut db = Self { pool, crawls };
        db.init_schema().await?;
        // Nothing crawls yet, so a crawl still open was cut off when the app
        // was killed or quit, and is never going to finish.
        match crate::storage::abort_unfinished_crawls(&db.pool, &db.crawls).await {
            Ok(0) => {}
            Ok(count) => tracing::info!(count, "marked unfinished crawls as aborted"),
            Err(error) => tracing::warn!(%error, "failed to mark unfinished crawls as aborted"),
        }
        Ok(db)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn crawls(&self) -> &crate::storage::CrawlStore {
        &self.crawls
    }

    fn app_db_path() -> PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("shoutingrobin");
        std::fs::create_dir_all(&path).ok();
        path.push("shoutingrobin.db");
        path
    }

    async fn init_schema(&mut self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_settings (
                id INTEGER PRIMARY KEY,
                theme TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        crate::storage::run_migrations(&self.pool, &self.crawls).await?;

        Ok(())
    }

    pub async fn get_user_settings(&self) -> Result<Option<UserSetting>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT ut.theme
            FROM user_settings ut
            WHERE ut.id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(UserSetting { theme: row.get(0) }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO settings (key, value, created_at, updated_at)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_all_settings(&self) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT key, value FROM settings
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.get(0), r.get(1))).collect())
    }
}
