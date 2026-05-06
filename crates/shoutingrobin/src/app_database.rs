use gpui::{App, Global};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use sqlx::{ConnectOptions, Row};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone)]
pub struct AppDatabase {
    pool: SqlitePool,
}

pub struct UserSetting {
    pub theme: String,
}

impl Global for AppDatabase {}

impl AppDatabase {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_path = Self::app_db_path();

        let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
            .create_if_missing(true)
            .disable_statement_logging();

        let pool = SqlitePool::connect_with(options).await?;

        let mut db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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

        crate::storage::run_migrations(&self.pool).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn save_user_settings(&self, user_setting: &UserSetting) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO user_settings (id, theme, updated_at) VALUES (1, ?, ?)
                ON CONFLICT (id)
                DO UPDATE
                SET theme = excluded.theme,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&user_setting.theme)
        .bind(now)
        .execute(&self.pool)
        .await?;

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
