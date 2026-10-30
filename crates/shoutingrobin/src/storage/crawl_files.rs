//! Each crawl's own database file, next to the catalog that lists the crawls.
//!
//! A crawl's pages, links, texts and the rest of what it found live in
//! `crawls/<id>.db`. Its size is the file's, and deleting it gives the space
//! back at once, where rows deleted from one shared file stayed allocated
//! until a vacuum rewrote it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr as _;
use std::sync::Arc;

use sqlx::ConnectOptions as _;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};

const CRAWL_MIGRATIONS: &[(&str, &str)] = &[(
    "0001_crawl",
    include_str!("../../migrations/crawl/0001_crawl.sql"),
)];

/// The tables a crawl file holds, in the order they are copied out of the
/// catalog when existing crawls move into files.
const CRAWL_TABLES: [&str; 14] = [
    "texts",
    "pages",
    "links",
    "images",
    "sitemap_urls",
    "a11y_violations",
    "structured_data",
    "performance",
    "ecommerce",
    "sd_issues",
    "page_layouts",
    "layout_keys",
    "layout_styles",
    "layout_templates",
];

/// One crawl's database.
#[derive(Clone, Debug)]
pub struct CrawlDb {
    pool: SqlitePool,
}

impl CrawlDb {
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Opens crawl files and keeps the pools open on them. Clones share them.
#[derive(Clone)]
pub struct CrawlStore {
    inner: Arc<StoreInner>,
}

struct StoreInner {
    dir: PathBuf,
    /// Held while a file opens, so two tasks opening the same crawl do not
    /// both create and migrate it.
    open: futures::lock::Mutex<HashMap<i64, CrawlDb>>,
    /// Owns the directory of a temporary store, removing it with the store.
    _temporary: Option<tempfile::TempDir>,
}

impl CrawlStore {
    pub fn new(dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(StoreInner {
                dir,
                open: futures::lock::Mutex::new(HashMap::new()),
                _temporary: None,
            }),
        }
    }

    /// A store in a directory of its own that is removed with the store.
    #[allow(dead_code)]
    pub(super) fn temporary() -> std::io::Result<Self> {
        let directory = tempfile::tempdir()?;
        Ok(Self {
            inner: Arc::new(StoreInner {
                dir: directory.path().to_path_buf(),
                open: futures::lock::Mutex::new(HashMap::new()),
                _temporary: Some(directory),
            }),
        })
    }

    pub fn path(&self, crawl_id: i64) -> PathBuf {
        self.inner.dir.join(format!("{crawl_id}.db"))
    }

    /// The crawl's database, created and brought up to date the first time.
    pub async fn open(&self, crawl_id: i64) -> Result<CrawlDb, sqlx::Error> {
        let mut open = self.inner.open.lock().await;
        if let Some(db) = open.get(&crawl_id) {
            return Ok(db.clone());
        }
        std::fs::create_dir_all(&self.inner.dir)?;
        let pool = connect(&self.path(crawl_id), SqliteJournalMode::Wal).await?;
        run_crawl_migrations(&pool).await?;
        let db = CrawlDb { pool };
        open.insert(crawl_id, db.clone());
        Ok(db)
    }

    /// Closes the crawl's database and deletes its file.
    pub async fn remove(&self, crawl_id: i64) -> Result<(), sqlx::Error> {
        let mut open = self.inner.open.lock().await;
        if let Some(db) = open.remove(&crawl_id) {
            db.pool.close().await;
        }
        remove_database_files(&self.path(crawl_id))?;
        Ok(())
    }

    /// The bytes the crawl's database takes on disk, its write-ahead log
    /// included. `None` when it has no file.
    pub fn size_on_disk(&self, crawl_id: i64) -> Option<u64> {
        let path = self.path(crawl_id);
        let database = std::fs::metadata(&path).ok()?.len();
        let log = std::fs::metadata(with_suffix(&path, "-wal"))
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        Some(database + log)
    }
}

/// A catalog and a crawl store in a temporary directory that is removed with
/// the store, for tests and tools that crawl without the app's data
/// directory. Files rather than memory: an in-memory SQLite database belongs
/// to the one connection that opened it, and a pool holds several. The app
/// itself never uses it, only the tests and the library's users.
#[allow(dead_code)]
pub async fn temporary_databases() -> Result<(SqlitePool, CrawlStore), sqlx::Error> {
    let store = CrawlStore::temporary()?;
    let catalog = connect_catalog(&store.inner.dir.join("catalog.db")).await?;
    super::run_migrations(&catalog, &store).await?;
    Ok((catalog, store))
}

/// Opens the catalog. Foreign keys are enforced, which the cascade from a
/// deleted crawl to its stored filter counts relies on.
pub async fn connect_catalog(path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .foreign_keys(true)
        .disable_statement_logging();
    SqlitePool::connect_with(options).await
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

fn remove_database_files(path: &Path) -> std::io::Result<()> {
    for file in [
        path.to_path_buf(),
        with_suffix(path, "-wal"),
        with_suffix(path, "-shm"),
    ] {
        match std::fs::remove_file(&file) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

/// Opens a crawl file with the settings the catalog uses: WAL so the grid can
/// read a crawl while the crawler writes it, and a busy timeout so a
/// contended write waits instead of failing.
async fn connect(path: &Path, journal: SqliteJournalMode) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))?
        .create_if_missing(true)
        .journal_mode(journal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(10))
        .disable_statement_logging();
    SqlitePool::connect_with(options).await
}

async fn run_crawl_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
    )
    .execute(pool)
    .await?;
    for (name, sql) in CRAWL_MIGRATIONS {
        let applied: Option<String> =
            sqlx::query_scalar("SELECT name FROM schema_migrations WHERE name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await?;
        if applied.is_some() {
            continue;
        }
        let mut transaction = pool.begin().await?;
        for statement in sql.split(";\n").map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(statement).execute(&mut *transaction).await?;
        }
        sqlx::query("INSERT INTO schema_migrations (name, applied_at) VALUES (?, ?)")
            .bind(name)
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
    }
    Ok(())
}

/// Copies every crawl still in the catalog's own tables into a file of its
/// own. A file is written under a temporary name and renamed once complete,
/// so a run cut short leaves either a whole file, which the next run keeps,
/// or a partial one, which it writes again.
pub(super) async fn move_crawls_into_files(
    catalog: &SqlitePool,
    store: &CrawlStore,
) -> Result<(), sqlx::Error> {
    let crawl_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM crawls ORDER BY id")
        .fetch_all(catalog)
        .await?;
    std::fs::create_dir_all(&store.inner.dir)?;
    for crawl_id in crawl_ids {
        let path = store.path(crawl_id);
        if path.exists() {
            continue;
        }
        let started = std::time::Instant::now();
        let partial = with_suffix(&path, ".partial");
        remove_database_files(&partial)?;
        // A rollback journal rather than WAL, so the finished copy is the
        // file alone and can be renamed without a log beside it.
        let pool = connect(&partial, SqliteJournalMode::Delete).await?;
        run_crawl_migrations(&pool).await?;
        pool.close().await;

        let mut conn = catalog.acquire().await?;
        sqlx::query("ATTACH DATABASE ? AS crawl")
            .bind(partial.display().to_string())
            .execute(&mut *conn)
            .await?;
        let copied = copy_crawl_tables(&mut conn, crawl_id).await;
        let detached = sqlx::query("DETACH DATABASE crawl")
            .execute(&mut *conn)
            .await;
        copied?;
        detached?;
        std::fs::rename(&partial, &path)?;
        tracing::info!(
            crawl_id,
            seconds = started.elapsed().as_secs_f64(),
            "moved a crawl into its own file"
        );
    }
    Ok(())
}

async fn copy_crawl_tables(
    conn: &mut sqlx::SqliteConnection,
    crawl_id: i64,
) -> Result<(), sqlx::Error> {
    let mut transaction = sqlx::Connection::begin(&mut *conn).await?;
    for table in CRAWL_TABLES {
        // The crawl file's own columns: the catalog's are the same names
        // with `crawl_id` added.
        let columns: Vec<String> =
            sqlx::query_scalar("SELECT name FROM pragma_table_info(?, 'crawl')")
                .bind(table)
                .fetch_all(&mut *transaction)
                .await?;
        let columns = columns.join(", ");
        sqlx::query(&format!(
            "INSERT INTO crawl.{table} ({columns}) SELECT {columns} FROM main.{table} WHERE crawl_id = ?"
        ))
        .bind(crawl_id)
        .execute(&mut *transaction)
        .await?;
    }
    // A finished crawl's counts are final, which One Inlink relies on.
    sqlx::query(
        "UPDATE crawl.crawl_state SET finished = \
         (SELECT finished_at IS NOT NULL FROM main.crawls WHERE id = ?)",
    )
    .bind(crawl_id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await
}
