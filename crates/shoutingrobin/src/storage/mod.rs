use sqlx::{Row, SqlitePool};

mod crawl_files;

#[allow(unused_imports)]
pub use crawl_files::temporary_databases;
pub use crawl_files::{CrawlDb, CrawlStore, connect_catalog};

use crate::crawl::event::{
    A11yIssue, ImageRef, Outlink, PageRecord, SdFormat, SdIssue, SdItem, SdSeverity,
};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../../migrations/0001_initial.sql"),
    ),
    (
        "0002_persistence_fix",
        include_str!("../../migrations/0002_persistence_fix.sql"),
    ),
    (
        "0003_images",
        include_str!("../../migrations/0003_images.sql"),
    ),
    (
        "0004_near_duplicates",
        include_str!("../../migrations/0004_near_duplicates.sql"),
    ),
    (
        "0005_sitemaps",
        include_str!("../../migrations/0005_sitemaps.sql"),
    ),
    (
        "0006_ecommerce_audit",
        include_str!("../../migrations/0006_ecommerce_audit.sql"),
    ),
    (
        "0007_a11y_counts",
        include_str!("../../migrations/0007_a11y_counts.sql"),
    ),
    (
        "0008_headers",
        include_str!("../../migrations/0008_headers.sql"),
    ),
    (
        "0009_redirect_url",
        include_str!("../../migrations/0009_redirect_url.sql"),
    ),
    (
        "0010_secondary_elements",
        include_str!("../../migrations/0010_secondary_elements.sql"),
    ),
    (
        "0011_redirect_status",
        include_str!("../../migrations/0011_redirect_status.sql"),
    ),
    (
        "0012_sd_issues",
        include_str!("../../migrations/0012_sd_issues.sql"),
    ),
    (
        "0013_link_score",
        include_str!("../../migrations/0013_link_score.sql"),
    ),
    (
        "0014_hreflang_issues",
        include_str!("../../migrations/0014_hreflang_issues.sql"),
    ),
    (
        "0015_ssr_content",
        include_str!("../../migrations/0015_ssr_content.sql"),
    ),
    (
        "0016_render_mode_and_near_dup_urls",
        include_str!("../../migrations/0016_render_mode_and_near_dup_urls.sql"),
    ),
    (
        "0017_readability",
        include_str!("../../migrations/0017_readability.sql"),
    ),
    (
        "0018_blocked_by_robots",
        include_str!("../../migrations/0018_blocked_by_robots.sql"),
    ),
    (
        "0019_links_csr_only",
        include_str!("../../migrations/0019_links_csr_only.sql"),
    ),
    ("0020_fcp", include_str!("../../migrations/0020_fcp.sql")),
    (
        "0021_is_resource",
        include_str!("../../migrations/0021_is_resource.sql"),
    ),
    (
        "0022_resource_initiator",
        include_str!("../../migrations/0022_resource_initiator.sql"),
    ),
    (
        "0023_is_page",
        include_str!("../../migrations/0023_is_page.sql"),
    ),
    (
        "0024_drop_readability",
        include_str!("../../migrations/0024_drop_readability.sql"),
    ),
    (
        "0025_mixed_content",
        include_str!("../../migrations/0025_mixed_content.sql"),
    ),
    (
        "0026_sitemap_lastmod",
        include_str!("../../migrations/0026_sitemap_lastmod.sql"),
    ),
    (
        "0027_hreflang_sources",
        include_str!("../../migrations/0027_hreflang_sources.sql"),
    ),
    (
        "0028_body_tag",
        include_str!("../../migrations/0028_body_tag.sql"),
    ),
    (
        "0029_h2_non_sequential",
        include_str!("../../migrations/0029_h2_non_sequential.sql"),
    ),
    (
        "0030_unique_page_url",
        include_str!("../../migrations/0030_unique_page_url.sql"),
    ),
    (
        "0031_drop_unused_tables",
        include_str!("../../migrations/0031_drop_unused_tables.sql"),
    ),
    (
        "0032_site_icons",
        include_str!("../../migrations/0032_site_icons.sql"),
    ),
    (
        "0033_page_layouts",
        include_str!("../../migrations/0033_page_layouts.sql"),
    ),
    (
        "0034_crawl_counts",
        include_str!("../../migrations/0034_crawl_counts.sql"),
    ),
    (
        "0035_crawl_filter_counts",
        include_str!("../../migrations/0035_crawl_filter_counts.sql"),
    ),
    (
        TEXTS_MIGRATION,
        include_str!("../../migrations/0036_texts.sql"),
    ),
    (
        "0037_site_settings",
        include_str!("../../migrations/0037_site_settings.sql"),
    ),
    (
        "0038_crawl_aborted",
        include_str!("../../migrations/0038_crawl_aborted.sql"),
    ),
    (
        "0039_asset_fetches",
        include_str!("../../migrations/0039_asset_fetches.sql"),
    ),
    (
        TEXT_HASH_MIGRATION,
        include_str!("../../migrations/0040_text_hashes.sql"),
    ),
    // Rewrites rows in Rust, see `compress_json_columns`.
    (COMPRESSED_JSON_MIGRATION, ""),
    (
        "0042_unused_indexes",
        include_str!("../../migrations/0042_unused_indexes.sql"),
    ),
    (
        "0043_transfer_bytes",
        include_str!("../../migrations/0043_transfer_bytes.sql"),
    ),
    (
        CRAWL_FILES_MIGRATION,
        include_str!("../../migrations/0044_crawl_files.sql"),
    ),
];

/// Stores every URL and repeated text once per crawl, in `texts`, and points
/// to it by id. It rewrites most of the database, which only gets smaller on
/// disk once it is vacuumed.
const TEXTS_MIGRATION: &str = "0036_texts";

/// Indexes `texts` by a hash of each text rather than the text itself, which
/// the index stored a second copy of. See `hash_texts`.
const TEXT_HASH_MIGRATION: &str = "0040_text_hashes";

/// Compresses the JSON columns already stored. See `compress_json_columns`.
const COMPRESSED_JSON_MIGRATION: &str = "0041_compressed_json";

/// Moves each crawl's rows out of the catalog into a file of its own, see
/// `crawl_files::move_crawls_into_files`, then drops the catalog's tables.
const CRAWL_FILES_MIGRATION: &str = "0044_crawl_files";

/// The migrations that rewrite enough of the database to vacuum after.
const REWRITING_MIGRATIONS: [&str; 5] = [
    TEXTS_MIGRATION,
    TEXT_HASH_MIGRATION,
    COMPRESSED_JSON_MIGRATION,
    "0042_unused_indexes",
    CRAWL_FILES_MIGRATION,
];

/// Brings the catalog up to date, moving crawls into files of their own in
/// `crawls` on the way.
pub async fn run_migrations(pool: &SqlitePool, crawls: &CrawlStore) -> Result<(), sqlx::Error> {
    migrate(pool, crawls, None).await
}

/// Applies the catalog's migrations in order, stopping before `stop_before`
/// when it is given.
async fn migrate(
    pool: &SqlitePool,
    crawls: &CrawlStore,
    stop_before: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    let mut rewritten = false;
    for (name, sql) in MIGRATIONS {
        if stop_before == Some(*name) {
            break;
        }
        let already: Option<(String,)> =
            sqlx::query_as("SELECT name FROM schema_migrations WHERE name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await?;
        if already.is_some() {
            continue;
        }
        // Before the migration's transaction: the copy attaches each crawl's
        // file, which SQLite does not allow inside one.
        if *name == CRAWL_FILES_MIGRATION {
            crawl_files::move_crawls_into_files(pool, crawls).await?;
        }
        let mut tx = pool.begin().await?;
        for stmt in sql.split(";\n").map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt).execute(&mut *tx).await?;
        }
        match *name {
            TEXT_HASH_MIGRATION => hash_texts(&mut tx).await?,
            COMPRESSED_JSON_MIGRATION => compress_json_columns(&mut tx).await?,
            _ => {}
        }
        sqlx::query("INSERT INTO schema_migrations (name, applied_at) VALUES (?, ?)")
            .bind(name)
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        tracing::info!("Applied migration {}", name);
        rewritten |= REWRITING_MIGRATIONS.contains(name);
    }

    if rewritten {
        // The migrations' results are kept whether or not this works: the
        // space only stays unused until a later vacuum.
        let started = std::time::Instant::now();
        match sqlx::query("VACUUM").execute(pool).await {
            Ok(_) => tracing::info!(
                seconds = started.elapsed().as_secs_f64(),
                "vacuumed the database"
            ),
            Err(e) => tracing::warn!(error=%e, "failed to vacuum the database"),
        }
        // In WAL mode the vacuum writes the whole database to the log, and
        // the log file keeps that size until it is truncated.
        if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(pool)
            .await
        {
            tracing::warn!(error=%e, "failed to truncate the write-ahead log");
        }
    }

    Ok(())
}

/// Rows a migration reads at a time, so a large crawl is never in memory at
/// once.
const MIGRATION_BATCH: i64 = 5_000;

/// Copies `texts` into `texts_hashed` with each text's hash, then puts it in
/// the place of `texts`, indexed by hash. Ids are kept, so every table that
/// points at a text still does.
async fn hash_texts(conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
    let crawl_ids: Vec<i64> = sqlx::query_scalar("SELECT DISTINCT crawl_id FROM texts")
        .fetch_all(&mut *conn)
        .await?;
    for crawl_id in crawl_ids {
        let mut hashes: std::collections::HashSet<i64> = std::collections::HashSet::new();
        let mut after = 0_i64;
        loop {
            let rows: Vec<(i64, String)> = sqlx::query_as(
                "SELECT id, text FROM texts WHERE crawl_id = ? AND id > ? ORDER BY id LIMIT ?",
            )
            .bind(crawl_id)
            .bind(after)
            .bind(MIGRATION_BATCH)
            .fetch_all(&mut *conn)
            .await?;
            let Some(&(last, _)) = rows.last() else {
                break;
            };
            after = last;
            let rows: Vec<(i64, Option<i64>, String)> = rows
                .into_iter()
                .map(|(id, text)| {
                    let hash = text_hash(&text);
                    // The texts are already distinct, so a hash met twice is
                    // two texts that share it: the second goes unhashed.
                    (id, hashes.insert(hash).then_some(hash), text)
                })
                .collect();
            for chunk in rows.chunks(TEXT_ROWS_PER_INSERT) {
                let mut insert: sqlx::QueryBuilder<sqlx::Sqlite> =
                    sqlx::QueryBuilder::new("INSERT INTO texts_hashed (id, crawl_id, hash, text) ");
                insert.push_values(chunk, |mut row, (id, hash, text)| {
                    row.push_bind(*id)
                        .push_bind(crawl_id)
                        .push_bind(*hash)
                        .push_bind(text.as_str());
                });
                insert.build().execute(&mut *conn).await?;
            }
        }
    }
    for statement in [
        "DROP TABLE texts",
        "ALTER TABLE texts_hashed RENAME TO texts",
        "CREATE UNIQUE INDEX idx_texts_crawl_hash ON texts(crawl_id, hash)",
    ] {
        sqlx::query(statement).execute(&mut *conn).await?;
    }
    Ok(())
}

/// Rewrites the JSON columns already stored with `pack_json`.
async fn compress_json_columns(conn: &mut sqlx::SqliteConnection) -> Result<(), sqlx::Error> {
    fn repack(value: Option<Vec<u8>>) -> Result<Option<Vec<u8>>, sqlx::Error> {
        value
            .map(|bytes| unpack_json(&bytes).map(|json| pack_json(&json)))
            .transpose()
    }

    let mut after = 0_i64;
    loop {
        type PageJson = (i64, Option<Vec<u8>>, Option<Vec<u8>>, Option<Vec<u8>>);
        let rows: Vec<PageJson> = sqlx::query_as(
            "SELECT id, CAST(headers_json AS BLOB), CAST(hreflang_tags_json AS BLOB),
                    CAST(near_duplicate_urls_json AS BLOB)
             FROM pages
             WHERE id > ? AND (headers_json IS NOT NULL OR hreflang_tags_json IS NOT NULL
                               OR near_duplicate_urls_json IS NOT NULL)
             ORDER BY id LIMIT ?",
        )
        .bind(after)
        .bind(MIGRATION_BATCH)
        .fetch_all(&mut *conn)
        .await?;
        let Some(&(last, ..)) = rows.last() else {
            break;
        };
        after = last;
        for (id, headers, hreflang, near_duplicates) in rows {
            sqlx::query(
                "UPDATE pages SET headers_json = ?, hreflang_tags_json = ?,
                                  near_duplicate_urls_json = ?
                 WHERE id = ?",
            )
            .bind(repack(headers)?)
            .bind(repack(hreflang)?)
            .bind(repack(near_duplicates)?)
            .bind(id)
            .execute(&mut *conn)
            .await?;
        }
    }

    let mut after = 0_i64;
    loop {
        let rows: Vec<(i64, Vec<u8>)> = sqlx::query_as(
            "SELECT id, CAST(json AS BLOB) FROM structured_data WHERE id > ? ORDER BY id LIMIT ?",
        )
        .bind(after)
        .bind(MIGRATION_BATCH)
        .fetch_all(&mut *conn)
        .await?;
        let Some(&(last, _)) = rows.last() else {
            break;
        };
        after = last;
        for (id, json) in rows {
            sqlx::query("UPDATE structured_data SET json = ? WHERE id = ?")
                .bind(pack_json(&unpack_json(&json)?))
                .bind(id)
                .execute(&mut *conn)
                .await?;
        }
    }
    Ok(())
}

/// The hash `texts` is indexed by. It is stored, so it has to give the same
/// value in every build: xxh3 is specified, unlike std's hasher.
fn text_hash(text: &str) -> i64 {
    i64::from_ne_bytes(xxhash_rust::xxh3::xxh3_64(text.as_bytes()).to_ne_bytes())
}

/// Rows per multi-row `INSERT` into `texts`, at four parameters a row.
const TEXT_ROWS_PER_INSERT: usize = 500;

/// The id of `text` in the crawl, stored first if the crawl has not met it.
/// The insert comes first so the write holds SQLite's write lock before it
/// reads, which keeps a concurrent write from slipping in between.
async fn text_id(conn: &mut sqlx::SqliteConnection, text: &str) -> Result<i64, sqlx::Error> {
    let hash = text_hash(text);
    sqlx::query("INSERT OR IGNORE INTO texts (hash, text) VALUES (?, ?)")
        .bind(hash)
        .bind(text)
        .execute(&mut *conn)
        .await?;
    // A text is found by its hash, or among the few stored without one
    // because another text holds their hash. Every lookup of a text by its
    // value has this shape.
    let found: Option<i64> =
        sqlx::query_scalar("SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?")
            .bind(hash)
            .bind(text)
            .fetch_optional(&mut *conn)
            .await?;
    if let Some(id) = found {
        return Ok(id);
    }
    // Another text of the crawl holds this hash, so the insert above kept
    // that one. This one is stored without a hash, where lookups also look.
    let inserted = sqlx::query("INSERT INTO texts (hash, text) VALUES (NULL, ?)")
        .bind(text)
        .execute(&mut *conn)
        .await?;
    Ok(inserted.last_insert_rowid())
}

/// A zstd frame's first four bytes. JSON text never starts with them.
const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];

/// JSON shorter than this is stored as it is: a frame's own overhead takes
/// most of what compressing it would save.
const COMPRESS_FROM_BYTES: usize = 128;

const COMPRESSION_LEVEL: i32 = 9;

/// A JSON column's stored form: zstd compressed when that is smaller.
/// `unpack_json` reads either form, and the plain text of older rows.
fn pack_json(json: &str) -> Vec<u8> {
    if json.len() >= COMPRESS_FROM_BYTES {
        match zstd::bulk::compress(json.as_bytes(), COMPRESSION_LEVEL) {
            Ok(compressed) if compressed.len() < json.len() => return compressed,
            Ok(_) => {}
            Err(error) => tracing::warn!(%error, "failed to compress a JSON column"),
        }
    }
    json.as_bytes().to_vec()
}

pub(crate) fn unpack_json(bytes: &[u8]) -> Result<String, sqlx::Error> {
    let decoded = if bytes.starts_with(&ZSTD_MAGIC) {
        zstd::stream::decode_all(bytes).map_err(|error| sqlx::Error::Decode(Box::new(error)))?
    } else {
        bytes.to_vec()
    };
    String::from_utf8(decoded).map_err(|error| sqlx::Error::Decode(Box::new(error)))
}

/// A JSON column read as `CAST(column AS BLOB)`.
fn json_column(row: &sqlx::sqlite::SqliteRow, column: &str) -> Result<Option<String>, sqlx::Error> {
    row.try_get::<Option<Vec<u8>>, _>(column)?
        .map(|bytes| unpack_json(&bytes))
        .transpose()
}

/// The ids of the texts one write stores. A page repeats its own URL on every
/// side table row and often the same anchor or alt text, so each is looked up
/// once per write.
struct TextIds<'a> {
    known: std::collections::HashMap<&'a str, i64>,
}

impl<'a> TextIds<'a> {
    fn new() -> Self {
        Self {
            known: std::collections::HashMap::new(),
        }
    }

    /// The text's id, stored first if the crawl has not met it yet.
    async fn id(
        &mut self,
        conn: &mut sqlx::SqliteConnection,
        text: &'a str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(&id) = self.known.get(text) {
            return Ok(id);
        }
        let id = text_id(conn, text).await?;
        self.known.insert(text, id);
        Ok(id)
    }

    async fn optional_id(
        &mut self,
        conn: &mut sqlx::SqliteConnection,
        text: Option<&'a str>,
    ) -> Result<Option<i64>, sqlx::Error> {
        match text {
            Some(text) => Ok(Some(self.id(conn, text).await?)),
            None => Ok(None),
        }
    }
}

/// Every text of one crawl by id, read once when the crawl loads so the large
/// tables are read as integers. One `Arc` per text: on a 400k row crawl, 1.5M
/// links pointed at 51k URLs.
struct CrawlTexts(std::collections::HashMap<i64, std::sync::Arc<str>>);

impl CrawlTexts {
    async fn load(db: &CrawlDb) -> Result<Self, sqlx::Error> {
        use futures::TryStreamExt as _;
        let mut rows =
            sqlx::query_as::<_, (i64, String)>("SELECT id, text FROM texts").fetch(db.pool());
        let mut texts = std::collections::HashMap::new();
        while let Some((id, text)) = rows.try_next().await? {
            texts.insert(id, text.into());
        }
        Ok(Self(texts))
    }

    fn get(&self, id: i64) -> Result<std::sync::Arc<str>, sqlx::Error> {
        self.0
            .get(&id)
            .cloned()
            .ok_or_else(|| sqlx::Error::Decode(format!("text {id} is not stored").into()))
    }

    fn optional(&self, id: Option<i64>) -> Result<Option<std::sync::Arc<str>>, sqlx::Error> {
        id.map(|id| self.get(id)).transpose()
    }
}

pub async fn create_crawl(
    pool: &SqlitePool,
    root_url: &str,
    render_mode: &str,
    config: &crate::crawl::CrawlConfig,
) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    // Stored so a crawl can be replayed later with the settings it actually ran
    // with, rather than whatever the settings happen to be at replay time.
    let config_json = match serde_json::to_string(config) {
        Ok(json) => Some(json),
        Err(e) => {
            tracing::warn!(error=%e, "failed to serialize crawl config, recrawl will fall back to current settings");
            None
        }
    };
    let result = sqlx::query(
        "INSERT INTO crawls (root_url, started_at, render_mode, config_json) VALUES (?, ?, ?, ?)",
    )
    .bind(root_url)
    .bind(now)
    .bind(render_mode)
    .bind(config_json)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

/// The config a crawl ran with, if it was recorded. Crawls created before
/// `config_json` was written back have `None` here.
pub async fn load_crawl_config(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Option<crate::crawl::CrawlConfig>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Option<String>,)>("SELECT config_json FROM crawls WHERE id = ?")
        .bind(crawl_id)
        .fetch_optional(pool)
        .await?;

    let Some((Some(json),)) = row else {
        return Ok(None);
    };

    match serde_json::from_str(&json) {
        Ok(config) => Ok(Some(config)),
        Err(e) => {
            tracing::warn!(error=%e, crawl_id, "stored crawl config could not be parsed");
            Ok(None)
        }
    }
}

/// A host's stored icon. `icon_png` is `None` when the last fetch found none.
#[derive(Clone, Debug)]
pub struct StoredSiteIcon {
    pub host: String,
    pub icon_png: Option<Vec<u8>>,
    pub fetched_at: i64,
}

/// Records the outcome of an icon fetch for `host`. A fetch that found nothing
/// keeps an icon stored earlier, since a site that is down for a moment has
/// not lost its icon.
pub async fn save_site_icon(
    pool: &SqlitePool,
    host: &str,
    icon_png: Option<&[u8]>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO site_icons (host, icon_png, fetched_at) VALUES (?, ?, ?) \
         ON CONFLICT(host) DO UPDATE SET \
         icon_png = COALESCE(excluded.icon_png, site_icons.icon_png), \
         fetched_at = excluded.fetched_at",
    )
    .bind(host)
    .bind(icon_png)
    .bind(chrono::Utc::now().timestamp())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn load_site_icons(pool: &SqlitePool) -> Result<Vec<StoredSiteIcon>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, Option<Vec<u8>>, i64)>(
        "SELECT host, icon_png, fetched_at FROM site_icons",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(host, icon_png, fetched_at)| StoredSiteIcon {
            host,
            icon_png,
            fetched_at,
        })
        .collect())
}

/// The crawl settings remembered for a site, with when they were saved.
pub async fn load_site_settings(
    pool: &SqlitePool,
    host: &str,
) -> Result<Option<(crate::crawl::site_settings::SiteSettings, i64)>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, i64)>(
        "SELECT settings_json, saved_at FROM site_settings WHERE host = ?",
    )
    .bind(host)
    .fetch_optional(pool)
    .await?;
    let Some((json, saved_at)) = row else {
        return Ok(None);
    };
    match serde_json::from_str(&json) {
        Ok(settings) => Ok(Some((settings, saved_at))),
        Err(error) => {
            tracing::warn!(%error, host, "remembered site settings could not be parsed");
            Ok(None)
        }
    }
}

pub async fn save_site_settings(
    pool: &SqlitePool,
    host: &str,
    settings: &crate::crawl::site_settings::SiteSettings,
    saved_at: i64,
) -> Result<(), sqlx::Error> {
    let json =
        serde_json::to_string(settings).map_err(|error| sqlx::Error::Encode(Box::new(error)))?;
    sqlx::query(
        "INSERT INTO site_settings (host, settings_json, saved_at) VALUES (?, ?, ?) \
         ON CONFLICT(host) DO UPDATE SET \
         settings_json = excluded.settings_json, saved_at = excluded.saved_at",
    )
    .bind(host)
    .bind(json)
    .bind(saved_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn forget_site_settings(pool: &SqlitePool, host: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM site_settings WHERE host = ?")
        .bind(host)
        .execute(pool)
        .await?;
    Ok(())
}

/// Stores one page's layout with the key and style table entries it added,
/// in one transaction so a page never refers to entries that were not saved.
pub async fn save_page_layout(
    db: &CrawlDb,
    url: &str,
    encoded_layout: &[u8],
    added: &crate::crawl::layout::NewTableEntries,
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    for (id, key) in &added.keys {
        sqlx::query("INSERT OR REPLACE INTO layout_keys (id, key) VALUES (?, ?)")
            .bind(i64::from(*id))
            .bind(key)
            .execute(&mut *tx)
            .await?;
    }
    for (id, style) in &added.styles {
        let style_json = serde_json::to_string(style).unwrap_or_else(|_| "[]".to_string());
        sqlx::query("INSERT OR REPLACE INTO layout_styles (id, style_json) VALUES (?, ?)")
            .bind(i64::from(*id))
            .bind(style_json)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("INSERT OR REPLACE INTO page_layouts (url, layout) VALUES (?, ?)")
        .bind(url)
        .bind(encoded_layout)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn load_layout_tables(
    db: &CrawlDb,
) -> Result<crate::crawl::layout::LayoutTables, sqlx::Error> {
    let keys = sqlx::query_as::<_, (i64, String)>("SELECT id, key FROM layout_keys")
        .fetch_all(db.pool())
        .await?;
    let styles = sqlx::query_as::<_, (i64, String)>("SELECT id, style_json FROM layout_styles")
        .fetch_all(db.pool())
        .await?;
    let keys = keys
        .into_iter()
        .filter_map(|(id, key)| u32::try_from(id).ok().map(|id| (id, key)))
        .collect();
    let styles = styles
        .into_iter()
        .filter_map(|(id, json)| {
            let id = u32::try_from(id).ok()?;
            match serde_json::from_str::<Vec<String>>(&json) {
                Ok(style) => Some((id, style)),
                Err(e) => {
                    tracing::warn!(error=%e, id, "stored layout style could not be parsed");
                    None
                }
            }
        })
        .collect();
    Ok(crate::crawl::layout::LayoutTables::from_rows(keys, styles))
}

/// One page's stored layout, still encoded.
pub async fn load_page_layout(db: &CrawlDb, url: &str) -> Result<Option<Vec<u8>>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Vec<u8>,)>("SELECT layout FROM page_layouts WHERE url = ?")
        .bind(url)
        .fetch_optional(db.pool())
        .await?;
    Ok(row.map(|(layout,)| layout))
}

/// A page of stored layouts in URL order, starting after `after_url`. Read
/// in pages so a pass over a large crawl never holds every layout at once.
pub async fn load_page_layouts_after(
    db: &CrawlDb,
    after_url: &str,
    limit: i64,
) -> Result<Vec<(String, Vec<u8>)>, sqlx::Error> {
    sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT url, layout FROM page_layouts WHERE url > ? ORDER BY url LIMIT ?",
    )
    .bind(after_url)
    .bind(limit)
    .fetch_all(db.pool())
    .await
}

/// Replaces the crawl's templates and every page's place in them.
pub async fn save_layout_grouping(
    db: &CrawlDb,
    grouping: &crate::crawl::layout_groups::Grouping,
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    sqlx::query("DELETE FROM layout_templates")
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE page_layouts SET template_id = NULL, assigned_by = NULL")
        .execute(&mut *tx)
        .await?;
    for template in &grouping.templates {
        sqlx::query(
            "INSERT INTO layout_templates (id, name, type_key, page_count) VALUES (?, ?, ?, ?)",
        )
        .bind(template.id)
        .bind(&template.name)
        .bind(&template.type_key)
        .bind(template.page_count as i64)
        .execute(&mut *tx)
        .await?;
    }
    for (url, (template_id, assignment)) in &grouping.pages {
        sqlx::query("UPDATE page_layouts SET template_id = ?, assigned_by = ? WHERE url = ?")
            .bind(template_id)
            .bind(assignment.as_str())
            .bind(url)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn count_page_layouts(db: &CrawlDb) -> Result<i64, sqlx::Error> {
    let (count,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM page_layouts")
        .fetch_one(db.pool())
        .await?;
    Ok(count)
}

/// A crawl's row counts as the history lists them: every row, the
/// documents, and the URLs found but never requested.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrawlCounts {
    pub pages: i64,
    pub documents: i64,
    pub unfetched: i64,
}

async fn count_crawl_rows(db: &CrawlDb) -> Result<CrawlCounts, sqlx::Error> {
    let (pages, documents, unfetched): (i64, i64, i64) = sqlx::query_as(
        r#"
        SELECT COUNT(*),
               COALESCE(SUM(is_page = 1), 0),
               COALESCE(SUM(status IS NULL AND redirect_url IS NULL), 0)
        FROM pages
        "#,
    )
    .fetch_one(db.pool())
    .await?;
    Ok(CrawlCounts {
        pages,
        documents,
        unfetched,
    })
}

/// Marks a crawl finished and stores its row counts, which the crawl history
/// lists. Counting them per listing read every crawl's pages, and a finished
/// crawl's rows no longer change.
pub async fn finish_crawl(
    catalog: &SqlitePool,
    db: &CrawlDb,
    crawl_id: i64,
) -> Result<(), sqlx::Error> {
    close_crawl(catalog, db, crawl_id, chrono::Utc::now().timestamp(), false).await
}

/// Records a crawl as over, in its own file and in the catalog's list.
async fn close_crawl(
    catalog: &SqlitePool,
    db: &CrawlDb,
    crawl_id: i64,
    finished_at: i64,
    aborted: bool,
) -> Result<(), sqlx::Error> {
    let counts = count_crawl_rows(db).await?;
    sqlx::query("UPDATE crawl_state SET finished = 1")
        .execute(db.pool())
        .await?;
    sqlx::query(
        r#"
        UPDATE crawls SET
            finished_at = ?, aborted = ?, page_count = ?, document_count = ?, unfetched_count = ?
        WHERE id = ?
        "#,
    )
    .bind(finished_at)
    .bind(aborted)
    .bind(counts.pages)
    .bind(counts.documents)
    .bind(counts.unfetched)
    .bind(crawl_id)
    .execute(catalog)
    .await?;
    Ok(())
}

/// Marks every crawl without a `finished_at` as aborted. Run at startup,
/// before any crawl can start: a crawl's process was killed, crashed or quit
/// mid crawl, so it will never finish. It is closed like a finished one, with
/// its stored counts, so the history stops calling it running. It finished
/// when its last page was recorded, or when it started if none was.
pub async fn abort_unfinished_crawls(
    catalog: &SqlitePool,
    crawls: &CrawlStore,
) -> Result<u64, sqlx::Error> {
    let unfinished: Vec<(i64, i64)> =
        sqlx::query_as("SELECT id, started_at FROM crawls WHERE finished_at IS NULL")
            .fetch_all(catalog)
            .await?;
    for &(crawl_id, started_at) in &unfinished {
        let db = crawls.open(crawl_id).await?;
        let last_page: Option<i64> = sqlx::query_scalar("SELECT MAX(crawled_at) FROM pages")
            .fetch_one(db.pool())
            .await?;
        close_crawl(
            catalog,
            &db,
            crawl_id,
            last_page.unwrap_or(started_at),
            true,
        )
        .await?;
    }
    Ok(unfinished.len() as u64)
}

/// The filter counts stored for a finished crawl under `filter_version`, as the
/// JSON the grid wrote. `None` for a crawl that has not finished, has none
/// stored, or has them under another version.
pub async fn load_filter_counts(
    pool: &SqlitePool,
    crawl_id: i64,
    filter_version: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT f.counts_json FROM crawl_filter_counts f
        JOIN crawls c ON c.id = f.crawl_id
        WHERE f.crawl_id = ? AND f.filter_version = ? AND c.finished_at IS NOT NULL
        "#,
    )
    .bind(crawl_id)
    .bind(filter_version)
    .fetch_optional(pool)
    .await
}

/// Stores a crawl's filter counts, replacing any under another version. Only a
/// finished crawl's are kept: until then its rows can still change. Returns
/// whether they were stored.
pub async fn store_filter_counts(
    pool: &SqlitePool,
    crawl_id: i64,
    filter_version: i64,
    counts_json: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO crawl_filter_counts (crawl_id, filter_version, counts_json)
        SELECT id, ?, ? FROM crawls WHERE id = ? AND finished_at IS NOT NULL
        ON CONFLICT(crawl_id) DO UPDATE SET
            filter_version = excluded.filter_version,
            counts_json = excluded.counts_json
        "#,
    )
    .bind(filter_version)
    .bind(counts_json)
    .bind(crawl_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Removes a crawl from the list and deletes its file.
pub async fn delete_crawl(
    catalog: &SqlitePool,
    crawls: &CrawlStore,
    crawl_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM crawls WHERE id = ?")
        .bind(crawl_id)
        .execute(catalog)
        .await?;
    crawls.remove(crawl_id).await
}

pub async fn insert_page(db: &CrawlDb, record: &PageRecord) -> Result<(), sqlx::Error> {
    // One transaction per page: a page with a hundred links and forty images
    // is otherwise two hundred autocommits, and a crash mid-page leaves a row
    // with half its links.
    let mut transaction = db.pool().begin().await?;
    if write_page(&mut transaction, record).await? {
        transaction.commit().await
    } else {
        transaction.rollback().await
    }
}

/// Records many pages in one transaction. Each page is its own savepoint, so
/// a URL already recorded leaves nothing behind, as with [`insert_page`]. The
/// sitemap orphans of a large site are hundreds of thousands of rows, and a
/// transaction each kept the crawl from finishing for minutes.
pub async fn insert_pages(db: &CrawlDb, records: &[PageRecord]) -> Result<(), sqlx::Error> {
    let mut transaction = db.pool().begin().await?;
    for record in records {
        let mut savepoint = sqlx::Connection::begin(&mut *transaction).await?;
        if write_page(&mut savepoint, record).await? {
            savepoint.commit().await?;
        } else {
            savepoint.rollback().await?;
        }
    }
    transaction.commit().await
}

/// Writes a page and its side tables. `false` when the URL was already
/// recorded, in which case the caller rolls back the text it stored.
async fn write_page(
    conn: &mut sqlx::SqliteConnection,
    record: &PageRecord,
) -> Result<bool, sqlx::Error> {
    let now = chrono::Utc::now().timestamp();

    let hreflang_json = if record.hreflang_tags.is_empty() {
        None
    } else {
        serde_json::to_string(record.hreflang_tags.as_slice()).ok()
    };
    let hreflang_sources_json = if record.hreflang_sources.is_empty() {
        None
    } else {
        serde_json::to_string(record.hreflang_sources.as_slice()).ok()
    };
    let sd_types_json = if record.sd_types.is_empty() {
        None
    } else {
        serde_json::to_string(record.sd_types.as_slice()).ok()
    };
    let headers_json = if record.headers.is_empty() {
        None
    } else {
        serde_json::to_string(record.headers.as_slice()).ok()
    };

    let mut texts = TextIds::new();
    let url_id = texts.id(conn, &record.url).await?;
    let inserted = sqlx::query(
        r#"
        INSERT INTO pages (
            url_id, status, content_type, size_bytes, response_time_ms,
            depth, title, meta_description, h1, h2, canonical, robots,
            indexability, word_count, hash, crawled_at,
            sd_errors, sd_warnings, hreflang_tags_json, sd_types_json,
            simhash, og_type, a11y_errors, a11y_warnings, headers_json,
            redirect_url, redirect_status,
            title_2, meta_description_2, h1_2, h2_2,
            title_pixel_width, meta_description_pixel_width,
            ssr_word_count, ssr_h1, ssr_content_missing,
            blocked_by_robots, is_resource, resource_initiator, is_page,
            has_mixed_content, hreflang_sources_json, has_body_tag,
            h2_non_sequential, network_fetches, revalidated_fetches, transfer_bytes
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(url_id) DO NOTHING
        "#,
    )
    .bind(url_id)
    .bind(record.status.map(|s| s as i64))
    .bind(record.content_type.as_deref())
    .bind(record.size_bytes as i64)
    .bind(record.response_time.as_millis() as i64)
    .bind(record.depth.map(|d| d as i64))
    .bind(record.title.as_deref())
    .bind(record.meta_description.as_deref())
    .bind(record.h1.as_deref())
    .bind(record.h2.as_deref())
    .bind(record.canonical.as_deref())
    .bind(record.robots.as_deref())
    .bind(record.indexability.as_deref().unwrap_or("N/A"))
    .bind(record.word_count.map(|w| w as i64))
    .bind(&record.content_hash)
    .bind(now)
    .bind(record.sd_errors as i64)
    .bind(record.sd_warnings as i64)
    .bind(hreflang_json.as_deref().map(pack_json))
    .bind(sd_types_json)
    .bind(record.simhash.map(|h| h as i64))
    .bind(&record.og_type)
    .bind(record.a11y_errors as i64)
    .bind(record.a11y_warnings as i64)
    .bind(headers_json.as_deref().map(pack_json))
    .bind(record.redirect_url.as_deref())
    .bind(record.redirect_status.map(|s| s as i64))
    .bind(record.title_2.as_deref())
    .bind(record.meta_description_2.as_deref())
    .bind(record.h1_2.as_deref())
    .bind(record.h2_2.as_deref())
    .bind(record.title_pixel_width.map(|w| w as i64))
    .bind(record.meta_description_pixel_width.map(|w| w as i64))
    .bind(record.ssr_word_count.map(|w| w as i64))
    .bind(record.ssr_h1.as_deref())
    .bind(record.ssr_content_missing.map(|b| b as i64))
    .bind(record.blocked_by_robots.map(|b| b as i64))
    .bind(record.is_resource as i64)
    .bind(record.resource_initiator.as_deref())
    .bind(record.is_page as i64)
    .bind(record.has_mixed_content as i64)
    .bind(hreflang_sources_json)
    .bind(record.has_body_tag.map(|has| has as i64))
    .bind(record.h2_non_sequential.map(|out_of_order| out_of_order as i64))
    .bind(record.network_fetches.map(i64::from))
    .bind(record.revalidated_fetches.map(i64::from))
    .bind(record.transfer_bytes.and_then(|bytes| i64::try_from(bytes).ok()))
    .execute(&mut *conn)
    .await?;
    // The first record of a URL wins: a page that Chrome later also reports
    // as a subresource of another page is already fully described.
    if inserted.rows_affected() == 0 {
        return Ok(false);
    }

    insert_structured_data(conn, url_id, record).await?;
    insert_performance(conn, url_id, record).await?;
    insert_images(conn, url_id, record, &mut texts).await?;
    insert_ecommerce(conn, url_id, record).await?;
    insert_links(conn, url_id, record, &mut texts).await?;
    insert_a11y_violations(conn, url_id, record, &mut texts).await?;
    insert_sd_issues(conn, url_id, record).await?;

    Ok(true)
}

async fn insert_structured_data(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    use crate::crawl::event::SdFormat;
    for item in &record.sd_items {
        let format_str = match item.format {
            SdFormat::JsonLd => "json-ld",
            SdFormat::Microdata => "microdata",
        };
        sqlx::query(
            "INSERT INTO structured_data (page_url_id, format, type_name, json, errors) VALUES (?, ?, ?, ?, NULL)",
        )
        .bind(url_id)
        .bind(format_str)
        .bind(&item.type_name)
        .bind(pack_json(&item.raw_json))
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_performance(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    if record.ttfb_ms.is_none()
        && record.lcp_ms.is_none()
        && record.cls.is_none()
        && record.fcp_ms.is_none()
    {
        return Ok(());
    }
    sqlx::query(
        "INSERT INTO performance (page_url_id, lcp_ms, cls, fcp_ms, ttfb_ms, transfer_kb) VALUES (?, ?, ?, ?, ?, NULL)",
    )
    .bind(url_id)
    .bind(record.lcp_ms.map(|ms| ms as i64))
    .bind(record.cls)
    .bind(record.fcp_ms.map(|ms| ms as i64))
    .bind(record.ttfb_ms.map(|ms| ms as i64))
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn insert_images<'a>(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &'a PageRecord,
    texts: &mut TextIds<'a>,
) -> Result<(), sqlx::Error> {
    for img in &record.images {
        let src_id = texts.id(conn, &img.src).await?;
        let alt_id = texts.optional_id(conn, img.alt.as_deref()).await?;
        sqlx::query(
            "INSERT INTO images (page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(url_id)
        .bind(src_id)
        .bind(alt_id)
        .bind(img.width.map(|w| w as i64))
        .bind(img.height.map(|h| h as i64))
        .bind(img.has_alt_attr as i64)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_ecommerce(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    let Some(ref audit) = record.ecommerce else {
        return Ok(());
    };
    sqlx::query(
        r#"
        INSERT INTO ecommerce (
            page_url_id, page_type, price, currency, availability,
            sku, gtin, brand, has_image, has_description, has_review_or_rating
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(url_id)
    .bind(record.og_type.as_deref())
    .bind(audit.price.as_deref().and_then(|p| p.parse::<f64>().ok()))
    .bind(audit.currency.as_deref())
    .bind(audit.availability.as_deref())
    .bind(audit.sku.as_deref())
    .bind(audit.gtin.as_deref())
    .bind(audit.brand.as_deref())
    .bind(audit.has_image as i64)
    .bind(audit.has_description as i64)
    .bind(audit.has_review_or_rating as i64)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

async fn insert_links<'a>(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &'a PageRecord,
    texts: &mut TextIds<'a>,
) -> Result<(), sqlx::Error> {
    for link in &record.outlinks {
        let internal = crate::crawl::engine::is_same_domain(&record.url, &link.dst_url);
        let dst_id = texts.id(conn, &link.dst_url).await?;
        let anchor_id = texts.optional_id(conn, link.anchor.as_deref()).await?;
        let rel_id = texts.optional_id(conn, link.rel.as_deref()).await?;
        sqlx::query(
            "INSERT INTO links (src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(url_id)
        .bind(dst_id)
        .bind(anchor_id)
        .bind(rel_id)
        .bind(internal as i64)
        .bind(link.csr_only as i64)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Every URL already recorded as a row of this crawl. The resource pass uses
/// it to skip URLs the crawler already reached or Chrome already reported.
pub async fn load_page_urls(
    db: &CrawlDb,
) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id")
            .fetch_all(db.pool())
            .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// The URLs of a crawl that need no further request: every page, and every
/// resource whose response the crawl actually saw. A resource Chrome reported
/// through the Resource Timing API with no size and no headers is a
/// cross-origin asset the browser was not allowed to describe, and is worth
/// a HEAD of its own.
pub async fn load_described_page_urls(
    db: &CrawlDb,
) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE NOT (p.is_resource = 1 AND p.size_bytes = 0 AND p.headers_json IS NULL)
        "#,
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// The resources Chrome reported without describing: rows from the Resource
/// Timing API with no size and no headers, as (url, initiator).
pub async fn load_undescribed_resources(
    db: &CrawlDb,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT t.text, p.resource_initiator FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE p.is_resource = 1 AND p.size_bytes = 0
          AND p.headers_json IS NULL
        ORDER BY p.id
        "#,
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows
        .into_iter()
        .map(|(url, initiator)| (url, initiator.unwrap_or_default()))
        .collect())
}

/// Records the outcome of a resource check. Updates the row a Resource Timing
/// entry already created for the URL, otherwise inserts it.
pub async fn record_resource_check(db: &CrawlDb, record: &PageRecord) -> Result<(), sqlx::Error> {
    let headers_json = if record.headers.is_empty() {
        None
    } else {
        serde_json::to_string(record.headers.as_slice()).ok()
    };
    let updated = sqlx::query(
        r#"
        UPDATE pages
        SET status = COALESCE(?, status),
            size_bytes = CASE WHEN ? > 0 THEN ? ELSE size_bytes END,
            content_type = COALESCE(?, content_type),
            headers_json = COALESCE(?, headers_json),
            redirect_url = COALESCE(?, redirect_url),
            redirect_status = COALESCE(?, redirect_status),
            indexability = ?
        WHERE is_resource = 1
          AND url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)
        "#,
    )
    .bind(record.status.map(|s| s as i64))
    .bind(record.size_bytes as i64)
    .bind(record.size_bytes as i64)
    .bind(record.content_type.as_deref())
    .bind(headers_json.as_deref().map(pack_json))
    .bind(record.redirect_url.as_deref())
    .bind(record.redirect_status.map(|s| s as i64))
    .bind(record.indexability.as_deref().unwrap_or("N/A"))
    .bind(text_hash(&record.url))
    .bind(&record.url)
    .execute(db.pool())
    .await?;
    if updated.rows_affected() == 0 {
        insert_page(db, record).await?;
    }
    Ok(())
}

/// Every document of a crawl that declares a canonical, as (url, canonical).
/// The canonical is returned exactly as the page wrote it, so the caller
/// resolves it against the page's own URL before doing anything with it.
pub async fn load_declared_canonicals(db: &CrawlDb) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT t.text, p.canonical FROM pages p JOIN texts t ON t.id = p.url_id
         WHERE p.is_page = 1 AND p.canonical IS NOT NULL
           AND trim(p.canonical) != ''",
    )
    .fetch_all(db.pool())
    .await
}

/// Every URL of a crawl that answered with a redirect.
///
/// A 3xx status is only half of them. spider follows a chain and reports the
/// *final* response, so the row for a redirecting URL can carry status 200 and
/// a `redirect_url` two hops away — which is the shape that hid the middle hop
/// of `/` -> `/sv/` -> `/sv`. The definition here matches
/// `PageRecord::is_redirect`, so the pass sees every row the app calls a
/// redirect.
pub async fn load_redirects(db: &CrawlDb) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id
         WHERE (p.redirect_url IS NOT NULL OR (p.status >= 300 AND p.status < 400))
         ORDER BY t.text",
    )
    .fetch_all(db.pool())
    .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// Records where a redirect went, once we have resolved it ourselves.
pub async fn set_redirect_target(
    db: &CrawlDb,
    url: &str,
    target: &str,
    status: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pages SET redirect_url = ?, redirect_status = ?
         WHERE url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
    )
    .bind(target)
    .bind(status as i64)
    .bind(text_hash(url))
    .bind(url)
    .execute(db.pool())
    .await?;
    Ok(())
}

/// How often Chrome fetched each asset over the network during a crawl, and
/// how many of those fetches were 304 revalidations, by asset URL.
pub type AssetFetches = std::collections::HashMap<String, (u32, u32)>;

/// Writes each asset's final network fetch counts onto its row. A row is
/// written when the asset is first seen, before later pages fetch it again,
/// so the counts are only complete once the crawl's pages are done.
pub async fn set_asset_fetches(db: &CrawlDb, fetches: &AssetFetches) -> Result<(), sqlx::Error> {
    let mut transaction = db.pool().begin().await?;
    for (url, (network, revalidated)) in fetches {
        sqlx::query(
            "UPDATE pages SET network_fetches = ?, revalidated_fetches = ?
             WHERE is_resource = 1
               AND url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
        )
        .bind(i64::from(*network))
        .bind(i64::from(*revalidated))
        .bind(text_hash(url))
        .bind(url)
        .execute(&mut *transaction)
        .await?;
    }
    transaction.commit().await?;
    Ok(())
}

async fn insert_a11y_violations<'a>(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &'a PageRecord,
    texts: &mut TextIds<'a>,
) -> Result<(), sqlx::Error> {
    for issue in &record.a11y_issues {
        let rule_id = texts.id(conn, &issue.rule).await?;
        let impact_id = texts.id(conn, &issue.impact).await?;
        let target_id = texts.optional_id(conn, issue.target.as_deref()).await?;
        let html_id = texts.optional_id(conn, issue.html.as_deref()).await?;
        sqlx::query(
            "INSERT INTO a11y_violations (page_url_id, rule_id, impact_id, target_id, html_id) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(url_id)
        .bind(rule_id)
        .bind(impact_id)
        .bind(target_id)
        .bind(html_id)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_sd_issues(
    conn: &mut sqlx::SqliteConnection,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for issue in &record.sd_issues {
        let severity_str = match issue.severity {
            SdSeverity::Error => "error",
            SdSeverity::Warning => "warning",
        };
        sqlx::query(
            "INSERT INTO sd_issues (page_url_id, severity, type_name, code, message) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(url_id)
        .bind(severity_str)
        .bind(&issue.type_name)
        .bind(&issue.code)
        .bind(&issue.message)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

pub async fn update_link_scores(db: &CrawlDb, scores: &[(String, f32)]) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    for (url, score) in scores {
        sqlx::query(
            "UPDATE pages SET link_score = ?
             WHERE url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
        )
        .bind(*score)
        .bind(text_hash(url))
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn update_crawl_depths(
    db: &CrawlDb,
    depths: &[(String, u32)],
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    for (url, depth) in depths {
        sqlx::query(
            "UPDATE pages SET depth = ?
             WHERE url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
        )
        .bind(*depth as i64)
        .bind(text_hash(url))
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn update_hreflang_issues(
    db: &CrawlDb,
    issues: &[(String, Vec<crate::crawl::event::HreflangIssue>)],
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    for (url, page_issues) in issues {
        let json = if page_issues.is_empty() {
            None
        } else {
            serde_json::to_string(page_issues).ok()
        };
        sqlx::query(
            "UPDATE pages SET hreflang_issues_json = ?
             WHERE url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
        )
        .bind(json)
        .bind(text_hash(url))
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// The links pointing at one URL in a crawl, in the order they were found.
/// Read when a page is selected rather than with the crawl: every page's list
/// together runs to hundreds of thousands of rows, and only the selected
/// page's is ever shown. External targets are included, since the page that
/// holds an external link is the only way to find where a broken one has to
/// be fixed.
/// What a page's details show that is not loaded with the crawl: the pages
/// linking to it, and its structured data with the JSON. Read per page on
/// selection, since across a large crawl they run to hundreds of MB.
pub struct PageOnDemand {
    pub backlinks: Vec<crate::crawl::event::Backlink>,
    pub sd_items: Vec<SdItem>,
}

pub async fn load_page_on_demand(db: &CrawlDb, url: &str) -> Result<PageOnDemand, sqlx::Error> {
    let backlinks = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        r#"
        SELECT s.text, a.text, r.text FROM links l
        JOIN texts s ON s.id = l.src_url_id
        LEFT JOIN texts a ON a.id = l.anchor_id
        LEFT JOIN texts r ON r.id = l.rel_id
        WHERE l.dst_url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)
        ORDER BY l.id
        "#,
    )
    .bind(text_hash(url))
    .bind(url)
    .fetch_all(db.pool())
    .await?
    .into_iter()
    .map(|(source_url, anchor, rel)| crate::crawl::event::Backlink {
        source_url,
        anchor,
        rel,
    })
    .collect();
    let sd_items = sqlx::query_as::<_, (String, String, Vec<u8>)>(
        "SELECT format, type_name, CAST(json AS BLOB) FROM structured_data \
         WHERE page_url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?) \
         ORDER BY id",
    )
    .bind(text_hash(url))
    .bind(url)
    .fetch_all(db.pool())
    .await?
    .into_iter()
    .map(|(format, type_name, raw_json)| {
        Ok(SdItem {
            format: sd_format(&format),
            type_name,
            raw_json: unpack_json(&raw_json)?,
        })
    })
    .collect::<Result<_, sqlx::Error>>()?;
    Ok(PageOnDemand {
        backlinks,
        sd_items,
    })
}

fn sd_format(stored: &str) -> SdFormat {
    match stored {
        "microdata" => SdFormat::Microdata,
        _ => SdFormat::JsonLd,
    }
}

#[derive(Debug, Clone)]
pub struct CrawlRow {
    pub id: i64,
    pub root_url: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    /// The app went away mid crawl, so the crawl never finished. Its
    /// `finished_at` was set by [`abort_unfinished_crawls`] at the next start.
    pub aborted: bool,
    /// Every URL the crawl recorded, documents and resources alike. This is the
    /// number on the sidebar badge, and it is far larger than the page count a
    /// tab shows, so [`CrawlRow::breakdown`] splits it for the badge's tooltip.
    pub page_count: i64,
    /// Of `page_count`, the rows that are a navigated HTML document.
    pub document_count: i64,
    /// Of `page_count`, the rows discovered but never requested: a resource
    /// past the check cap, or a URL only a sitemap mentioned.
    pub unfetched_count: i64,
    pub render_mode: String,
    /// How the crawl recorded page layout, from its stored config. Crawls
    /// recorded before the setting existed read as off.
    pub layout_capture: crate::crawl::LayoutCapture,
    /// The bytes the crawl's file takes on disk. `None` when it has none.
    pub size_bytes: Option<u64>,
}

impl CrawlRow {
    /// The badge's total split into the three groups it is made of, as counts
    /// that add back up to it.
    pub fn breakdown(&self) -> (i64, i64, i64) {
        let others = (self.page_count - self.document_count - self.unfetched_count).max(0);
        (self.document_count, others, self.unfetched_count)
    }
}

/// The `SELECT` list every crawl-row query shares, so the sidebar and the
/// baseline lookup describe a crawl the same way. A crawl's counts are stored
/// when it ends, so a running crawl has none here and [`crawl_row`] counts its
/// file.
const CRAWL_ROW_COLUMNS: &str = r#"
    c.id, c.root_url, c.started_at, c.finished_at, c.aborted,
    c.page_count, c.document_count, c.unfetched_count, c.render_mode,
    json_extract(c.config_json, '$.layout_capture') as layout_capture
"#;

type CrawlRowColumns = (
    i64,
    String,
    i64,
    Option<i64>,
    bool,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    String,
    Option<String>,
);

async fn crawl_row(crawls: &CrawlStore, columns: CrawlRowColumns) -> Result<CrawlRow, sqlx::Error> {
    let (
        id,
        root_url,
        started_at,
        finished_at,
        aborted,
        page_count,
        document_count,
        unfetched_count,
        render_mode,
        layout_capture,
    ) = columns;
    let layout_capture = match layout_capture {
        Some(value) => serde_json::from_value(serde_json::Value::String(value.clone()))
            .unwrap_or_else(|error| {
                tracing::warn!(%error, crawl_id = id, value, "unknown stored layout capture");
                crate::crawl::LayoutCapture::Off
            }),
        None => crate::crawl::LayoutCapture::Off,
    };
    let counts = match (page_count, document_count, unfetched_count) {
        (Some(pages), Some(documents), Some(unfetched)) => CrawlCounts {
            pages,
            documents,
            unfetched,
        },
        _ => count_crawl_rows(&crawls.open(id).await?).await?,
    };
    Ok(CrawlRow {
        id,
        root_url,
        started_at,
        finished_at,
        aborted,
        page_count: counts.pages,
        document_count: counts.documents,
        unfetched_count: counts.unfetched,
        render_mode,
        layout_capture,
        size_bytes: crawls.size_on_disk(id),
    })
}

pub async fn list_crawls(
    catalog: &SqlitePool,
    crawls: &CrawlStore,
) -> Result<Vec<CrawlRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CrawlRowColumns>(&format!(
        r#"
        SELECT {CRAWL_ROW_COLUMNS}
        FROM crawls c
        ORDER BY c.started_at DESC
        LIMIT 100
        "#
    ))
    .fetch_all(catalog)
    .await?;

    let mut listed = Vec::with_capacity(rows.len());
    for row in rows {
        listed.push(crawl_row(crawls, row).await?);
    }
    Ok(listed)
}

/// Finds the crawl to use as a comparison baseline for a given root URL.
///
/// When `current_crawl_id` is `Some`, returns the most recent crawl of the same
/// `root_url` that started strictly before the given crawl (the sidebar flow).
/// When `None`, the current crawl is assumed to be the most recent row for the
/// URL (a just-finished live crawl), so we skip it with `OFFSET 1`.
pub async fn find_previous_crawl(
    catalog: &SqlitePool,
    crawls: &CrawlStore,
    root_url: &str,
    current_crawl_id: Option<i64>,
) -> Result<Option<CrawlRow>, sqlx::Error> {
    let row = match current_crawl_id {
        Some(id) => {
            sqlx::query_as::<_, CrawlRowColumns>(&format!(
                r#"
                SELECT {CRAWL_ROW_COLUMNS}
                FROM crawls c
                WHERE c.root_url = ?
                  AND c.started_at < (SELECT started_at FROM crawls WHERE id = ?)
                ORDER BY c.started_at DESC
                LIMIT 1
                "#
            ))
            .bind(root_url)
            .bind(id)
            .fetch_optional(catalog)
            .await?
        }
        None => {
            sqlx::query_as::<_, CrawlRowColumns>(&format!(
                r#"
                SELECT {CRAWL_ROW_COLUMNS}
                FROM crawls c
                WHERE c.root_url = ?
                ORDER BY c.started_at DESC
                LIMIT 1 OFFSET 1
                "#
            ))
            .bind(root_url)
            .fetch_optional(catalog)
            .await?
        }
    };

    match row {
        Some(row) => Ok(Some(crawl_row(crawls, row).await?)),
        None => Ok(None),
    }
}

/// The parts of reading a crawl back. After the crawl's texts, they run at the
/// same time and finish in any order, so a loading screen can count them off
/// as they arrive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoadStep {
    Texts,
    Links,
    PageDetails,
    Inlinks,
    Pages,
}

impl LoadStep {
    pub const COUNT: usize = 5;
}

/// The app shows progress while it reads, so only the library's users and the
/// tests read a crawl without it.
#[allow(dead_code)]
pub async fn load_pages_for_crawl(
    db: &CrawlDb,
    root_url: &str,
) -> Result<Vec<PageRecord>, sqlx::Error> {
    load_pages_for_crawl_reporting(db, root_url, |_| {}).await
}

/// [`load_pages_for_crawl`], calling `report` as each step finishes.
///
/// The crawl's texts are read first, so the large tables after them are read
/// as integers and every URL and repeated text is one shared copy. Those reads
/// are independent, and on a crawl with a million links each takes seconds,
/// so they run at once on separate pool connections. Each is its own task
/// because turning SQLite rows into Rust values costs as much as reading them,
/// and in one task that work would still run one read at a time.
pub async fn load_pages_for_crawl_reporting(
    db: &CrawlDb,
    root_url: &str,
    report: impl Fn(LoadStep),
) -> Result<Vec<PageRecord>, sqlx::Error> {
    let report = &report;
    let crawl_finished: bool = sqlx::query_scalar("SELECT finished != 0 FROM crawl_state")
        .fetch_optional(db.pool())
        .await?
        .unwrap_or(false);
    let texts = std::sync::Arc::new(CrawlTexts::load(db).await?);
    report(LoadStep::Texts);
    let links = async_std::task::spawn({
        let db = db.clone();
        let texts = texts.clone();
        async move {
            use futures::TryStreamExt as _;
            let mut rows = sqlx::query_as::<_, (i64, i64, Option<i64>, Option<i64>, i64)>(
                r#"
                SELECT src_url_id, dst_url_id, anchor_id, rel_id, csr_only
                FROM links
                ORDER BY id
                "#,
            )
            .fetch(db.pool());
            let mut outlinks_by_url: std::collections::HashMap<i64, Vec<Outlink>> =
                std::collections::HashMap::new();
            while let Some((src_id, dst_id, anchor_id, rel_id, csr_only)) = rows.try_next().await? {
                outlinks_by_url.entry(src_id).or_default().push(Outlink {
                    dst_url: texts.get(dst_id)?,
                    anchor: texts.optional(anchor_id)?,
                    rel: texts.optional(rel_id)?,
                    csr_only: csr_only != 0,
                });
            }
            Ok::<_, sqlx::Error>(outlinks_by_url)
        }
    });
    let page_details = async_std::task::spawn({
        let db = db.clone();
        let texts = texts.clone();
        async move {
            let (
                perf_rows,
                sd_type_rows,
                images_by_url,
                a11y_by_url,
                sd_issue_rows,
                layout_rows,
                sitemap_rows,
                ecom_rows,
            ) = futures::try_join!(
                load_performance_rows(&db),
                load_structured_data_rows(&db),
                load_images_by_url(&db, &texts),
                load_a11y_by_url(&db, &texts),
                load_sd_issue_rows(&db),
                load_layout_rows(&db),
                load_sitemap_rows(&db),
                load_ecommerce_rows(&db),
            )?;
            let mut layout_by_url: std::collections::HashMap<
                String,
                crate::crawl::event::PageLayoutInfo,
            > = std::collections::HashMap::with_capacity(layout_rows.len());
            for (url, template, template_type, assigned_by) in layout_rows {
                layout_by_url.insert(
                    url,
                    crate::crawl::event::PageLayoutInfo {
                        template,
                        template_type,
                        assignment: assigned_by
                            .as_deref()
                            .and_then(crate::crawl::layout_groups::Assignment::from_stored),
                    },
                );
            }

            #[allow(clippy::type_complexity)]
            let mut perf_by_url: std::collections::HashMap<
                i64,
                (Option<i64>, Option<i64>, Option<f64>, Option<i64>),
            > = std::collections::HashMap::new();
            for (url_id, ttfb, lcp, cls, fcp) in &perf_rows {
                perf_by_url.insert(*url_id, (*ttfb, *lcp, *cls, *fcp));
            }

            let mut sd_items_by_url: std::collections::HashMap<i64, Vec<SdItem>> =
                std::collections::HashMap::new();
            for (url_id, format, type_name) in sd_type_rows {
                sd_items_by_url.entry(url_id).or_default().push(SdItem {
                    format: sd_format(&format),
                    type_name,
                    raw_json: String::new(),
                });
            }

            let mut sd_issues_by_url: std::collections::HashMap<i64, Vec<SdIssue>> =
                std::collections::HashMap::new();
            for (url_id, severity, type_name, code, message) in sd_issue_rows {
                let severity = match severity.as_str() {
                    "error" => SdSeverity::Error,
                    _ => SdSeverity::Warning,
                };
                sd_issues_by_url.entry(url_id).or_default().push(SdIssue {
                    severity,
                    type_name,
                    code,
                    message,
                });
            }

            let mut sitemap_by_url: std::collections::HashMap<i64, SitemapEntry> =
                std::collections::HashMap::new();
            for (url_id, sitemap_id, lastmod_id) in sitemap_rows {
                if let std::collections::hash_map::Entry::Vacant(entry) =
                    sitemap_by_url.entry(url_id)
                {
                    entry.insert((texts.get(sitemap_id)?, texts.optional(lastmod_id)?));
                }
            }

            let mut ecom_by_url: std::collections::HashMap<
                i64,
                crate::crawl::event::EcommerceAudit,
            > = std::collections::HashMap::new();
            for (
                url_id,
                _page_type,
                price,
                currency,
                availability,
                sku,
                gtin,
                brand,
                has_image,
                has_description,
                has_review_or_rating,
            ) in ecom_rows
            {
                ecom_by_url.insert(
                    url_id,
                    crate::crawl::event::EcommerceAudit {
                        price: price.map(|p| p.to_string()),
                        currency,
                        availability,
                        sku,
                        gtin,
                        brand,
                        has_image: has_image != 0,
                        has_description: has_description != 0,
                        has_review_or_rating: has_review_or_rating != 0,
                    },
                );
            }
            Ok::<_, sqlx::Error>(PageDetailsByUrl {
                layout: layout_by_url,
                performance: perf_by_url,
                sd_items: sd_items_by_url,
                images: images_by_url,
                a11y: a11y_by_url,
                sd_issues: sd_issues_by_url,
                sitemap: sitemap_by_url,
                ecommerce: ecom_by_url,
            })
        }
    });
    let inlinks = async_std::task::spawn({
        let db = db.clone();
        async move {
            let rows = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
                r#"
                SELECT dst_url_id,
                       COUNT(*),
                       COUNT(DISTINCT src_url_id),
                       COALESCE(SUM(csr_only), 0),
                       COUNT(DISTINCT CASE WHEN csr_only = 1 THEN src_url_id END)
                FROM links GROUP BY dst_url_id
                "#,
            )
            .fetch_all(db.pool())
            .await?;
            // (total, unique sources, rendered-only links, unique rendered-only
            // sources). The unique rendered-only figure is the one that says how
            // much of a page's discoverability depends on a JavaScript nav running.
            let counts: std::collections::HashMap<i64, (u32, u32, u32, u32)> = rows
                .into_iter()
                .map(|(url_id, total, unique, csr, unique_csr)| {
                    (
                        url_id,
                        (total as u32, unique as u32, csr as u32, unique_csr as u32),
                    )
                })
                .collect();
            Ok::<_, sqlx::Error>(counts)
        }
    });
    // One pass over `pages`, keyed by URL: several aligned scans joined by row
    // position put another page's title on a URL the moment one of them grew a
    // WHERE clause.
    let pages = async_std::task::spawn({
        let db = db.clone();
        let texts = texts.clone();
        let root_url = root_url.to_string();
        async move {
            use futures::TryStreamExt as _;
            let mut rows = sqlx::query(
                r#"
                SELECT url_id, status, title, size_bytes, content_type, meta_description, h1, h2,
                       canonical, word_count, depth, indexability, response_time_ms, og_type,
                       a11y_errors, a11y_warnings, CAST(headers_json AS BLOB) AS headers_json,
                       robots, is_resource,
                       resource_initiator, is_page, has_mixed_content, has_body_tag, redirect_url,
                       redirect_status, title_2, meta_description_2, h1_2, h2_2, title_pixel_width,
                       meta_description_pixel_width, ssr_word_count, ssr_h1, ssr_content_missing,
                       blocked_by_robots, h2_non_sequential, network_fetches, revalidated_fetches,
                       transfer_bytes,
                       hash, simhash, closest_similarity,
                       near_duplicate_count,
                       CAST(near_duplicate_urls_json AS BLOB) AS near_duplicate_urls_json,
                       sd_errors, sd_warnings,
                       CAST(hreflang_tags_json AS BLOB) AS hreflang_tags_json, sd_types_json, hreflang_issues_json, hreflang_sources_json,
                       link_score
                FROM pages
                ORDER BY id
                "#,
            )
            .fetch(db.pool());
            let mut records = Vec::new();
            let mut shared = SharedTexts::default();
            while let Some(row) = rows.try_next().await? {
                let url_id: i64 = row.try_get("url_id")?;
                let url = texts.get(url_id)?.to_string();
                records.push((
                    url_id,
                    page_record_from_row(&row, url, &root_url, &mut shared)?,
                ));
            }
            Ok::<_, sqlx::Error>(records)
        }
    });
    let (mut outlinks_by_url, details, inlink_counts, page_rows) = futures::try_join!(
        reported(LoadStep::Links, links, report),
        reported(LoadStep::PageDetails, page_details, report),
        reported(LoadStep::Inlinks, inlinks, report),
        reported(LoadStep::Pages, pages, report),
    )?;
    let PageDetailsByUrl {
        layout: mut layout_by_url,
        performance: perf_by_url,
        sd_items: mut sd_items_by_url,
        images: mut images_by_url,
        a11y: mut a11y_by_url,
        sd_issues: mut sd_issues_by_url,
        sitemap: mut sitemap_by_url,
        ecommerce: mut ecom_by_url,
    } = details;

    let sitemap_known = !sitemap_by_url.is_empty();
    let mut records = Vec::with_capacity(page_rows.len());
    for (url_id, mut record) in page_rows {
        (record.ttfb_ms, record.lcp_ms, record.cls, record.fcp_ms) = perf_by_url
            .get(&url_id)
            .map(|(ttfb, lcp, cls, fcp)| {
                (
                    ttfb.map(|ms| ms as u64),
                    lcp.map(|ms| ms as u64),
                    *cls,
                    fcp.map(|ms| ms as u64),
                )
            })
            .unwrap_or_default();
        let page_sitemap = sitemap_by_url.remove(&url_id);
        record.in_sitemap = if page_sitemap.is_some() {
            Some(true)
        } else if sitemap_known {
            Some(false)
        } else {
            None
        };
        (record.sitemap_url, record.sitemap_lastmod) = page_sitemap
            .map(|(sitemap_url, lastmod)| (Some(sitemap_url), lastmod))
            .unwrap_or_default();
        (
            record.inlinks_count,
            record.unique_inlinks_count,
            record.csr_inlinks_count,
            record.unique_csr_inlinks_count,
        ) = inlink_counts.get(&url_id).copied().unwrap_or_default();
        record.inlinks_complete = crawl_finished;
        record.layout = layout_by_url.remove(&record.url).map(Box::new);
        record.outlinks = outlinks_by_url.remove(&url_id).unwrap_or_default().into();
        record.images = images_by_url.remove(&url_id).unwrap_or_default().into();
        record.ecommerce = ecom_by_url.remove(&url_id).map(Box::new);
        record.a11y_issues = a11y_by_url.remove(&url_id).unwrap_or_default().into();
        record.sd_items = sd_items_by_url.remove(&url_id).unwrap_or_default().into();
        record.sd_jsonld_count = record
            .sd_items
            .iter()
            .filter(|item| item.format == SdFormat::JsonLd)
            .count() as u32;
        record.sd_microdata_count = record
            .sd_items
            .iter()
            .filter(|item| item.format == SdFormat::Microdata)
            .count() as u32;
        record.sd_issues = sd_issues_by_url.remove(&url_id).unwrap_or_default().into();
        // Backlinks and structured data JSON are read per page on selection,
        // see `load_page_on_demand`.
        records.push(record);
    }

    Ok(records)
}

/// The columns of one `pages` row, before the per-page side tables are joined
/// on in [`load_pages_for_crawl_reporting`].
fn page_record_from_row(
    row: &sqlx::sqlite::SqliteRow,
    url: String,
    root_url: &str,
    texts: &mut SharedTexts,
) -> Result<PageRecord, sqlx::Error> {
    let mut shared = |column: &str| -> Result<Option<std::sync::Arc<str>>, sqlx::Error> {
        Ok(row
            .try_get::<Option<String>, _>(column)?
            .map(|text| texts.share(text)))
    };
    fn json_list<T: serde::de::DeserializeOwned>(value: Option<&str>) -> thin_vec::ThinVec<T> {
        value
            .and_then(|json| serde_json::from_str::<Vec<T>>(json).ok())
            .map(thin_vec::ThinVec::from)
            .unwrap_or_default()
    }

    let is_internal = crate::crawl::engine::is_same_domain(root_url, &url);
    let status: Option<i64> = row.try_get("status")?;
    let size_bytes: i64 = row.try_get("size_bytes")?;
    let word_count: Option<i64> = row.try_get("word_count")?;
    let depth: Option<i64> = row.try_get("depth")?;
    let response_time_ms: Option<i64> = row.try_get("response_time_ms")?;
    let a11y_errors: i64 = row.try_get("a11y_errors")?;
    let a11y_warnings: i64 = row.try_get("a11y_warnings")?;
    let is_resource: i64 = row.try_get("is_resource")?;
    let is_page: i64 = row.try_get("is_page")?;
    let has_mixed_content: i64 = row.try_get("has_mixed_content")?;
    let has_body_tag: Option<i64> = row.try_get("has_body_tag")?;
    let redirect_status: Option<i64> = row.try_get("redirect_status")?;
    let title_pixel_width: Option<i64> = row.try_get("title_pixel_width")?;
    let meta_description_pixel_width: Option<i64> = row.try_get("meta_description_pixel_width")?;
    let ssr_word_count: Option<i64> = row.try_get("ssr_word_count")?;
    let ssr_content_missing: Option<i64> = row.try_get("ssr_content_missing")?;
    let blocked_by_robots: Option<i64> = row.try_get("blocked_by_robots")?;
    let h2_non_sequential: Option<i64> = row.try_get("h2_non_sequential")?;
    let network_fetches: Option<i64> = row.try_get("network_fetches")?;
    let revalidated_fetches: Option<i64> = row.try_get("revalidated_fetches")?;
    let transfer_bytes: Option<i64> = row.try_get("transfer_bytes")?;
    let simhash: Option<i64> = row.try_get("simhash")?;
    let closest_similarity: Option<i64> = row.try_get("closest_similarity")?;
    let near_duplicate_count: Option<i64> = row.try_get("near_duplicate_count")?;
    let sd_errors: i64 = row.try_get("sd_errors")?;
    let sd_warnings: i64 = row.try_get("sd_warnings")?;
    Ok(PageRecord {
        status: status.map(|s| s as u16),
        title: row.try_get("title")?,
        size_bytes: size_bytes as u64,
        content_type: shared("content_type")?,
        meta_description: row.try_get("meta_description")?,
        h1: row.try_get("h1")?,
        h2: row.try_get("h2")?,
        canonical: row.try_get("canonical")?,
        robots: row.try_get("robots")?,
        word_count: word_count.map(|w| w as u32),
        depth: depth.map(|d| d as u32),
        is_internal,
        is_page: is_page != 0,
        is_resource: is_resource != 0,
        resource_initiator: shared("resource_initiator")?,
        has_mixed_content: has_mixed_content != 0,
        has_body_tag: has_body_tag.map(|has| has != 0),
        indexability: shared("indexability")?,
        response_time: std::time::Duration::from_millis(response_time_ms.unwrap_or(0) as u64),
        sd_errors: sd_errors as u32,
        sd_warnings: sd_warnings as u32,
        sd_types: json_list(row.try_get::<Option<&str>, _>("sd_types_json")?),
        hreflang_tags: json_list(json_column(row, "hreflang_tags_json")?.as_deref()),
        hreflang_issues: json_list(row.try_get::<Option<&str>, _>("hreflang_issues_json")?),
        hreflang_sources: json_list(row.try_get::<Option<&str>, _>("hreflang_sources_json")?),
        content_hash: row.try_get("hash")?,
        simhash: simhash.map(|h| h as u64),
        closest_similarity: closest_similarity.map(|s| s as u8),
        near_duplicate_count: near_duplicate_count.map(|c| c as u32),
        near_duplicate_urls: json_list(json_column(row, "near_duplicate_urls_json")?.as_deref()),
        og_type: row.try_get("og_type")?,
        a11y_errors: a11y_errors as u32,
        a11y_warnings: a11y_warnings as u32,
        headers: json_list(json_column(row, "headers_json")?.as_deref()),
        redirect_url: row.try_get("redirect_url")?,
        redirect_status: redirect_status.map(|s| s as u16),
        title_2: row.try_get("title_2")?,
        meta_description_2: row.try_get("meta_description_2")?,
        h1_2: row.try_get("h1_2")?,
        h2_2: row.try_get("h2_2")?,
        title_pixel_width: title_pixel_width.map(|w| w as u32),
        meta_description_pixel_width: meta_description_pixel_width.map(|w| w as u32),
        ssr_word_count: ssr_word_count.map(|w| w as u32),
        ssr_h1: row.try_get("ssr_h1")?,
        ssr_content_missing: ssr_content_missing.map(|b| b != 0),
        blocked_by_robots: blocked_by_robots.map(|b| b != 0),
        h2_non_sequential: h2_non_sequential.map(|b| b != 0),
        network_fetches: network_fetches.and_then(|count| u32::try_from(count).ok()),
        revalidated_fetches: revalidated_fetches.and_then(|count| u32::try_from(count).ok()),
        transfer_bytes: transfer_bytes.and_then(|bytes| u64::try_from(bytes).ok()),
        link_score: row.try_get("link_score")?,
        url,
        ..Default::default()
    })
}

/// The sitemap a URL was listed in, and the `<lastmod>` it gave.
type SitemapEntry = (std::sync::Arc<str>, Option<std::sync::Arc<str>>);

/// A crawl's per-page side tables, each grouped by the id of the page's URL,
/// except layouts, which their own tables key by the URL itself.
struct PageDetailsByUrl {
    layout: std::collections::HashMap<String, crate::crawl::event::PageLayoutInfo>,
    performance:
        std::collections::HashMap<i64, (Option<i64>, Option<i64>, Option<f64>, Option<i64>)>,
    sd_items: std::collections::HashMap<i64, Vec<SdItem>>,
    images: std::collections::HashMap<i64, Vec<ImageRef>>,
    a11y: std::collections::HashMap<i64, Vec<A11yIssue>>,
    sd_issues: std::collections::HashMap<i64, Vec<SdIssue>>,
    sitemap: std::collections::HashMap<i64, SitemapEntry>,
    ecommerce: std::collections::HashMap<i64, crate::crawl::event::EcommerceAudit>,
}

/// Waits for one of a crawl load's reads and reports its step once it is in.
async fn reported<T>(
    step: LoadStep,
    read: async_std::task::JoinHandle<Result<T, sqlx::Error>>,
    report: &impl Fn(LoadStep),
) -> Result<T, sqlx::Error> {
    let rows = read.await?;
    report(step);
    Ok(rows)
}

type PerformanceRow = (i64, Option<i64>, Option<i64>, Option<f64>, Option<i64>);
type ImageRow = (i64, i64, Option<i64>, Option<i64>, Option<i64>, i64);
type EcommerceRow = (
    i64,
    Option<String>,
    Option<f64>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
    i64,
    i64,
);

async fn load_performance_rows(db: &CrawlDb) -> Result<Vec<PerformanceRow>, sqlx::Error> {
    sqlx::query_as("SELECT page_url_id, ttfb_ms, lcp_ms, cls, fcp_ms FROM performance")
        .fetch_all(db.pool())
        .await
}

/// Every structured data item of a crawl, without its JSON: only the details
/// panel shows that, and it reads it per page, see [`load_page_on_demand`].
async fn load_structured_data_rows(
    db: &CrawlDb,
) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
    sqlx::query_as("SELECT page_url_id, format, type_name FROM structured_data ORDER BY id")
        .fetch_all(db.pool())
        .await
}

/// One copy of each distinct text the `pages` read meets in the columns that
/// repeat across rows, such as a MIME type.
#[derive(Default)]
struct SharedTexts(std::collections::HashSet<std::sync::Arc<str>>);

impl SharedTexts {
    fn share(&mut self, text: String) -> std::sync::Arc<str> {
        if let Some(shared) = self.0.get(text.as_str()) {
            return shared.clone();
        }
        let shared: std::sync::Arc<str> = text.into();
        self.0.insert(shared.clone());
        shared
    }
}

/// A crawl's images, grouped by the page that shows them. Streamed, as are the
/// other large reads: collecting a million rows before grouping them held both
/// copies at once, and the allocator kept that peak long after.
async fn load_images_by_url(
    db: &CrawlDb,
    texts: &CrawlTexts,
) -> Result<std::collections::HashMap<i64, Vec<ImageRef>>, sqlx::Error> {
    use futures::TryStreamExt as _;
    let mut rows = sqlx::query_as::<_, ImageRow>(
        "SELECT page_url_id, src_id, alt_id, width, height, has_alt_attr FROM images \
 ORDER BY id",
    )
    .fetch(db.pool());
    let mut images_by_url: std::collections::HashMap<i64, Vec<ImageRef>> =
        std::collections::HashMap::new();
    while let Some((url_id, src_id, alt_id, width, height, has_alt_attr)) = rows.try_next().await? {
        images_by_url.entry(url_id).or_default().push(ImageRef {
            src: texts.get(src_id)?,
            alt: texts.optional(alt_id)?,
            width: width.map(|w| w as u32),
            height: height.map(|h| h as u32),
            has_alt_attr: has_alt_attr != 0,
        });
    }
    Ok(images_by_url)
}

async fn load_a11y_by_url(
    db: &CrawlDb,
    texts: &CrawlTexts,
) -> Result<std::collections::HashMap<i64, Vec<A11yIssue>>, sqlx::Error> {
    use futures::TryStreamExt as _;
    let mut rows = sqlx::query_as::<_, (i64, i64, i64, Option<i64>, Option<i64>)>(
        "SELECT page_url_id, rule_id, impact_id, target_id, html_id FROM a11y_violations \
 ORDER BY id",
    )
    .fetch(db.pool());
    let mut a11y_by_url: std::collections::HashMap<i64, Vec<A11yIssue>> =
        std::collections::HashMap::new();
    while let Some((url_id, rule_id, impact_id, target_id, html_id)) = rows.try_next().await? {
        a11y_by_url.entry(url_id).or_default().push(A11yIssue {
            rule: texts.get(rule_id)?,
            impact: texts.get(impact_id)?,
            target: texts.optional(target_id)?,
            html: texts.optional(html_id)?,
        });
    }
    Ok(a11y_by_url)
}

async fn load_sd_issue_rows(
    db: &CrawlDb,
) -> Result<Vec<(i64, String, String, String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, severity, type_name, code, message FROM sd_issues \
 ORDER BY id",
    )
    .fetch_all(db.pool())
    .await
}

async fn load_layout_rows(
    db: &CrawlDb,
) -> Result<Vec<(String, Option<String>, Option<String>, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT pl.url, t.name, t.type_key, pl.assigned_by
        FROM page_layouts pl
        LEFT JOIN layout_templates t ON t.id = pl.template_id

        "#,
    )
    .fetch_all(db.pool())
    .await
}

async fn load_sitemap_rows(db: &CrawlDb) -> Result<Vec<(i64, i64, Option<i64>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, sitemap_url_id, lastmod_id FROM sitemap_urls \
 ORDER BY id",
    )
    .fetch_all(db.pool())
    .await
}

async fn load_ecommerce_rows(db: &CrawlDb) -> Result<Vec<EcommerceRow>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT page_url_id, page_type, price, currency, availability,
               sku, gtin, brand, has_image, has_description, has_review_or_rating
        FROM ecommerce
        "#,
    )
    .fetch_all(db.pool())
    .await
}

pub async fn load_simhashes_for_crawl(
    db: &CrawlDb,
) -> Result<Vec<(String, u64, Option<String>)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, Option<i64>, Option<String>)>(
        r#"
        SELECT t.text, p.simhash, p.hash
        FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE p.simhash IS NOT NULL
        ORDER BY p.id
        "#,
    )
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(url, simhash, content_hash)| simhash.map(|sh| (url, sh as u64, content_hash)))
        .collect())
}

pub async fn update_near_duplicates(
    db: &CrawlDb,
    results: &[(String, u8, u32, Vec<String>)],
) -> Result<(), sqlx::Error> {
    let mut tx = db.pool().begin().await?;
    for (url, similarity, count, dup_urls) in results {
        let dup_urls_json = if dup_urls.is_empty() {
            None
        } else {
            serde_json::to_string(dup_urls).ok()
        };
        sqlx::query(
            "UPDATE pages SET closest_similarity = ?, near_duplicate_count = ?, near_duplicate_urls_json = ?
             WHERE url_id = (SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?)",
        )
        .bind(*similarity as i64)
        .bind(*count as i64)
        .bind(dup_urls_json.as_deref().map(pack_json))
        .bind(text_hash(url))
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Rows per multi-row `INSERT`, kept well under SQLite's bound parameter
/// limit at four parameters a row.
const SITEMAP_ROWS_PER_INSERT: usize = 500;

/// Stores a crawl's sitemap entries before the crawl starts. A large shop
/// lists hundreds of thousands of URLs, and storing them one statement at a
/// time held the crawl back for over half a minute, so the texts and the rows
/// go in as multi-row inserts and the ids are read back in one query.
pub async fn insert_sitemap_urls(
    db: &CrawlDb,
    sitemap_urls: &[crate::crawl::sitemap::SitemapUrl],
) -> Result<(), sqlx::Error> {
    use futures::TryStreamExt as _;

    let mut distinct: Vec<&str> = Vec::new();
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for entry in sitemap_urls {
        for text in [
            Some(entry.sitemap_url.as_str()),
            Some(entry.page_url.as_str()),
            entry.lastmod.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            if seen.insert(text) {
                distinct.push(text);
            }
        }
    }

    let mut transaction = db.pool().begin().await?;
    for chunk in distinct.chunks(SITEMAP_ROWS_PER_INSERT) {
        let mut insert: sqlx::QueryBuilder<sqlx::Sqlite> =
            sqlx::QueryBuilder::new("INSERT OR IGNORE INTO texts (hash, text) ");
        insert.push_values(chunk, |mut row, text| {
            row.push_bind(text_hash(text)).push_bind(*text);
        });
        insert.build().execute(&mut *transaction).await?;
    }

    let mut ids: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    {
        let mut rows = sqlx::query_as::<_, (i64, String)>("SELECT id, text FROM texts")
            .fetch(&mut *transaction);
        while let Some((id, text)) = rows.try_next().await? {
            if seen.contains(text.as_str()) {
                ids.insert(text, id);
            }
        }
    }
    // The batch insert skipped any text whose hash another text holds.
    for text in &distinct {
        if !ids.contains_key(*text) {
            let id = text_id(&mut transaction, text).await?;
            ids.insert(text.to_string(), id);
        }
    }
    let id_of = |text: &str| {
        ids.get(text)
            .copied()
            .ok_or_else(|| sqlx::Error::Protocol(format!("no text id stored for {text}")))
    };

    for chunk in sitemap_urls.chunks(SITEMAP_ROWS_PER_INSERT) {
        let rows = chunk
            .iter()
            .map(|entry| {
                Ok((
                    id_of(&entry.sitemap_url)?,
                    id_of(&entry.page_url)?,
                    entry.lastmod.as_deref().map(id_of).transpose()?,
                ))
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        let mut insert: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
            "INSERT INTO sitemap_urls (sitemap_url_id, page_url_id, lastmod_id) ",
        );
        insert.push_values(rows, |mut row, (sitemap_id, page_id, lastmod_id)| {
            row.push_bind(sitemap_id)
                .push_bind(page_id)
                .push_bind(lastmod_id);
        });
        insert.build().execute(&mut *transaction).await?;
    }
    transaction.commit().await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SitemapStatus {
    pub page_url: String,
    pub sitemap_url: String,
    pub lastmod: Option<String>,
}

pub async fn load_sitemap_orphans(db: &CrawlDb) -> Result<Vec<SitemapStatus>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        r#"
        SELECT page.text, sitemap.text, lastmod.text
        FROM sitemap_urls su
        JOIN texts page ON page.id = su.page_url_id
        JOIN texts sitemap ON sitemap.id = su.sitemap_url_id
        LEFT JOIN texts lastmod ON lastmod.id = su.lastmod_id
        LEFT JOIN pages p ON p.url_id = su.page_url_id
        WHERE p.id IS NULL
        ORDER BY su.id
        "#,
    )
    .fetch_all(db.pool())
    .await?;

    Ok(rows
        .into_iter()
        .map(|(page_url, sitemap_url, lastmod)| SitemapStatus {
            page_url,
            sitemap_url,
            lastmod,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> crate::crawl::CrawlConfig {
        crate::crawl::CrawlConfig {
            max_pages: 0,
            max_concurrent: 1,
            delay_ms: 0,
            timeout_seconds: 10,
            respect_robots_txt: false,
            follow_sitemaps: false,
            block_images: false,
            near_duplicate_threshold: 90,
            content_selector: String::new(),
            user_agent: None,
            extra_headers: Vec::new(),
            include_patterns: Vec::new(),
            exclude_patterns: Vec::new(),
            crawl_subdomains: false,
            list_mode: false,
            seed_urls: Vec::new(),
            check_resources: false,
            layout_capture: crate::crawl::LayoutCapture::Off,
        }
    }

    async fn test_databases() -> (SqlitePool, CrawlStore) {
        temporary_databases().await.expect("databases")
    }

    fn link(dst_url: &str, anchor: &str) -> crate::crawl::event::Outlink {
        crate::crawl::event::Outlink {
            dst_url: dst_url.into(),
            anchor: Some(anchor.into()),
            rel: None,
            csr_only: false,
        }
    }

    /// A site's crawl settings come back as they were saved, a later save
    /// replaces them, and forgetting them leaves nothing to restore.
    #[test]
    fn site_settings_are_remembered_per_host_until_forgotten() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let (pool, _crawls) = test_databases().await;
            let settings = crate::crawl::site_settings::SiteSettings {
                exclude_patterns: vec!["/tag/".to_string()],
                max_pages: 500,
                header_names: vec!["Authorization".to_string()],
                ..Default::default()
            };
            save_site_settings(&pool, "books.toscrape.com", &settings, 10)
                .await
                .expect("save");
            assert_eq!(
                load_site_settings(&pool, "books.toscrape.com")
                    .await
                    .expect("load"),
                Some((settings.clone(), 10))
            );
            assert_eq!(
                load_site_settings(&pool, "shop.books.toscrape.com")
                    .await
                    .expect("load"),
                None
            );

            let changed = crate::crawl::site_settings::SiteSettings {
                max_pages: 0,
                ..settings
            };
            save_site_settings(&pool, "books.toscrape.com", &changed, 20)
                .await
                .expect("save again");
            assert_eq!(
                load_site_settings(&pool, "books.toscrape.com")
                    .await
                    .expect("load"),
                Some((changed, 20))
            );

            forget_site_settings(&pool, "books.toscrape.com")
                .await
                .expect("forget");
            assert_eq!(
                load_site_settings(&pool, "books.toscrape.com")
                    .await
                    .expect("load"),
                None
            );
        });
    }

    /// The details panel reads one page's backlinks when it is selected, so
    /// they have to be that page's and in the order the links were found.
    #[test]
    fn a_page_reads_only_its_own_backlinks() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let backlinks = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let crawl_id = create_crawl(&pool, "https://a.test/", "http", &config())
                .await
                .expect("crawl");
            for (url, outlinks) in [
                (
                    "https://a.test/one",
                    vec![
                        link("https://a.test/target", "First"),
                        link("https://a.test/other", "Other"),
                    ],
                ),
                (
                    "https://a.test/two",
                    vec![link("https://a.test/target", "Second")],
                ),
            ] {
                insert_page(
                    &crawls.open(crawl_id).await.expect("crawl file"),
                    &PageRecord {
                        url: url.into(),
                        status: Some(200),
                        is_page: true,
                        outlinks: outlinks.into(),
                        ..Default::default()
                    },
                )
                .await
                .expect("page");
            }
            load_page_on_demand(
                &crawls.open(crawl_id).await.expect("crawl file"),
                "https://a.test/target",
            )
            .await
            .expect("backlinks")
            .backlinks
        });
        let found: Vec<(&str, Option<&str>)> = backlinks
            .iter()
            .map(|backlink| (backlink.source_url.as_str(), backlink.anchor.as_deref()))
            .collect();
        assert_eq!(
            found,
            vec![
                ("https://a.test/one", Some("First")),
                ("https://a.test/two", Some("Second")),
            ]
        );
    }

    /// The loading screen names each step as it starts, so the steps have to
    /// be reported in the order they run and the crawl still has to come back
    /// whole.
    #[test]
    fn a_load_reports_every_step() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (steps, pages) = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let root = "https://a.test/";
            let crawl_id = create_crawl(&pool, root, "http", &config())
                .await
                .expect("crawl");
            insert_page(
                &crawls.open(crawl_id).await.expect("crawl file"),
                &PageRecord {
                    url: root.into(),
                    status: Some(200),
                    is_page: true,
                    is_internal: true,
                    ..Default::default()
                },
            )
            .await
            .expect("page");

            let steps = std::cell::RefCell::new(Vec::new());
            let pages = load_pages_for_crawl_reporting(
                &crawls.open(crawl_id).await.expect("crawl file"),
                root,
                |step| steps.borrow_mut().push(step),
            )
            .await
            .expect("load");
            let mut steps = steps.into_inner();
            steps.sort();
            (steps, pages)
        });
        assert_eq!(
            steps,
            vec![
                LoadStep::Texts,
                LoadStep::Links,
                LoadStep::PageDetails,
                LoadStep::Inlinks,
                LoadStep::Pages
            ]
        );
        assert_eq!(pages.len(), 1);
    }

    /// A crawl loads its structured data without the JSON, which only the
    /// details panel shows. Selecting the page reads it back, in order.
    #[test]
    fn structured_data_json_is_read_when_its_page_is_selected() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (loaded, on_demand) = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let root = "https://a.test/";
            let crawl_id = create_crawl(&pool, root, "http", &config())
                .await
                .expect("crawl");
            let item = |format, type_name: &str, raw_json: &str| SdItem {
                format,
                type_name: type_name.into(),
                raw_json: raw_json.into(),
            };
            insert_page(
                &crawls.open(crawl_id).await.expect("crawl file"),
                &PageRecord {
                    url: root.into(),
                    status: Some(200),
                    is_page: true,
                    is_internal: true,
                    sd_items: thin_vec::thin_vec![
                        item(SdFormat::JsonLd, "Product", r#"{"@type":"Product"}"#),
                        item(SdFormat::Microdata, "Offer", r#"{"@type":"Offer"}"#),
                    ],
                    ..Default::default()
                },
            )
            .await
            .expect("page");
            let loaded =
                load_pages_for_crawl(&crawls.open(crawl_id).await.expect("crawl file"), root)
                    .await
                    .expect("load");
            let on_demand =
                load_page_on_demand(&crawls.open(crawl_id).await.expect("crawl file"), root)
                    .await
                    .expect("on demand");
            (loaded, on_demand)
        });
        let [page] = loaded.as_slice() else {
            panic!("one page, got {}", loaded.len());
        };
        let loaded_items: Vec<(&str, bool)> = page
            .sd_items
            .iter()
            .map(|item| (item.type_name.as_str(), item.raw_json.is_empty()))
            .collect();
        assert_eq!(loaded_items, vec![("Product", true), ("Offer", true)]);
        assert_eq!((page.sd_jsonld_count, page.sd_microdata_count), (1, 1));
        let selected: Vec<(SdFormat, &str)> = on_demand
            .sd_items
            .iter()
            .map(|item| (item.format, item.raw_json.as_str()))
            .collect();
        assert_eq!(
            selected,
            vec![
                (SdFormat::JsonLd, r#"{"@type":"Product"}"#),
                (SdFormat::Microdata, r#"{"@type":"Offer"}"#),
            ]
        );
    }

    /// The crawl history lists a finished crawl from the counts stored when it
    /// finished, and counts a crawl that has not finished as it lists it.
    #[test]
    fn the_history_lists_the_counts_stored_when_a_crawl_finished() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (finished, running) = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let mut crawl_ids = Vec::new();
            for root in ["https://finished.test/", "https://running.test/"] {
                let crawl_id = create_crawl(&pool, root, "http", &config())
                    .await
                    .expect("crawl");
                for (path, status) in [("", Some(200)), ("unfetched", None)] {
                    insert_page(
                        &crawls.open(crawl_id).await.expect("crawl file"),
                        &PageRecord {
                            url: format!("{root}{path}"),
                            status,
                            is_page: status.is_some(),
                            is_internal: true,
                            ..Default::default()
                        },
                    )
                    .await
                    .expect("page");
                }
                crawl_ids.push(crawl_id);
            }
            let [finished_id, running_id] = crawl_ids[..] else {
                panic!("two crawls");
            };
            finish_crawl(
                &pool,
                &crawls.open(finished_id).await.expect("crawl file"),
                finished_id,
            )
            .await
            .expect("finish");
            // Rows written after the finish are not what the finished crawl
            // found, so its listing keeps the stored counts.
            for crawl_id in [finished_id, running_id] {
                insert_page(
                    &crawls.open(crawl_id).await.expect("crawl file"),
                    &PageRecord {
                        url: format!("https://late.test/{crawl_id}"),
                        status: Some(200),
                        is_page: true,
                        ..Default::default()
                    },
                )
                .await
                .expect("late page");
            }
            let crawls = list_crawls(&pool, &crawls).await.expect("list");
            let counts = |crawl_id: i64| {
                crawls
                    .iter()
                    .find(|crawl| crawl.id == crawl_id)
                    .map(|crawl| {
                        (
                            crawl.page_count,
                            crawl.document_count,
                            crawl.unfetched_count,
                        )
                    })
            };
            (counts(finished_id), counts(running_id))
        });
        assert_eq!(finished, Some((2, 1, 1)));
        assert_eq!(running, Some((3, 2, 1)));
    }

    /// Sitemap entries past one insert's worth of rows are all stored with
    /// their texts, and the orphans recorded from them in one batch keep the
    /// first record of a URL already crawled.
    #[test]
    fn sitemap_entries_and_their_orphans_are_stored_in_batches() {
        use crate::crawl::sitemap::SitemapUrl;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let (orphans, pages) = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let crawl_id = create_crawl(&pool, "https://a.test/", "http", &config())
                .await
                .expect("crawl");
            let entries: Vec<SitemapUrl> = (0..SITEMAP_ROWS_PER_INSERT + 3)
                .map(|index| SitemapUrl {
                    sitemap_url: format!("https://a.test/sitemap_{}.xml", index % 2),
                    page_url: format!("https://a.test/{index}"),
                    lastmod: (index % 3 == 0).then(|| "2026-10-30".to_string()),
                    hreflang: Vec::new(),
                })
                .collect();
            insert_sitemap_urls(&crawls.open(crawl_id).await.expect("crawl file"), &entries)
                .await
                .expect("sitemap");
            insert_page(
                &crawls.open(crawl_id).await.expect("crawl file"),
                &PageRecord {
                    url: "https://a.test/0".into(),
                    status: Some(200),
                    is_page: true,
                    ..Default::default()
                },
            )
            .await
            .expect("crawled page");

            let orphans = load_sitemap_orphans(&crawls.open(crawl_id).await.expect("crawl file"))
                .await
                .expect("orphans");
            let records: Vec<PageRecord> = ["https://a.test/0", "https://a.test/1"]
                .into_iter()
                .map(|url| PageRecord {
                    url: url.into(),
                    in_sitemap: Some(true),
                    ..Default::default()
                })
                .collect();
            insert_pages(&crawls.open(crawl_id).await.expect("crawl file"), &records)
                .await
                .expect("batch");
            let pages = sqlx::query_as::<_, (String, Option<i64>)>(
                "SELECT t.text, p.status FROM pages p JOIN texts t ON t.id = p.url_id ORDER BY p.id",
            )
            .fetch_all(crawls.open(crawl_id).await.expect("crawl file").pool())
            .await
            .expect("pages");
            (orphans, pages)
        });
        assert_eq!(orphans.len(), SITEMAP_ROWS_PER_INSERT + 2);
        let last = orphans.last().expect("an orphan");
        assert_eq!(
            (
                last.page_url.as_str(),
                last.sitemap_url.as_str(),
                last.lastmod.as_deref()
            ),
            ("https://a.test/502", "https://a.test/sitemap_0.xml", None)
        );
        assert_eq!(orphans[2].lastmod.as_deref(), Some("2026-10-30"));
        assert_eq!(
            pages,
            vec![
                ("https://a.test/0".to_string(), Some(200)),
                ("https://a.test/1".to_string(), None),
            ]
        );
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
    }

    #[test]
    fn a_text_whose_hash_is_taken_gets_its_own_id() {
        runtime().block_on(async {
            let (pool, crawls) = test_databases().await;
            let crawl_id = create_crawl(&pool, "https://a.test/", "http", &config())
                .await
                .expect("crawl");
            let db = crawls.open(crawl_id).await.expect("crawl file");
            let wanted = "https://a.test/wanted";
            // Another text already holding the wanted text's hash, as a
            // collision would leave it.
            sqlx::query("INSERT INTO texts (hash, text) VALUES (?, ?)")
                .bind(text_hash(wanted))
                .bind("https://a.test/holder")
                .execute(db.pool())
                .await
                .expect("holder");
            let mut conn = db.pool().acquire().await.expect("connection");
            let first = text_id(&mut conn, wanted).await.expect("first");
            let again = text_id(&mut conn, wanted).await.expect("again");
            let holder = text_id(&mut conn, "https://a.test/holder")
                .await
                .expect("holder id");
            assert_eq!(first, again);
            assert_ne!(first, holder);
            let looked_up: Option<i64> = sqlx::query_scalar(
                "SELECT id FROM texts WHERE (hash = ? OR hash IS NULL) AND text = ?",
            )
            .bind(text_hash(wanted))
            .bind(wanted)
            .fetch_optional(&mut *conn)
            .await
            .expect("lookup");
            assert_eq!(looked_up, Some(first));
        });
    }

    #[test]
    fn json_is_compressed_when_that_is_smaller_and_read_back_either_way() {
        let long =
            serde_json::to_string(&vec![("cache-control", "public, max-age=0"); 40]).expect("json");
        let packed = pack_json(&long);
        assert!(packed.starts_with(&ZSTD_MAGIC));
        assert!(packed.len() < long.len());
        assert_eq!(unpack_json(&packed).expect("unpack"), long);

        let short = r#"[["a","b"]]"#;
        assert_eq!(pack_json(short), short.as_bytes());
        // A row stored before compression holds the JSON as it is.
        assert_eq!(unpack_json(long.as_bytes()).expect("plain"), long);
    }

    /// A crawl stored the way the catalog held crawls before they had files
    /// of their own, its texts unhashed and its JSON plain, comes out of the
    /// migrations in a file of its own, hashed, compressed and whole.
    #[test]
    fn a_crawl_stored_in_the_catalog_moves_into_a_file_of_its_own() {
        runtime().block_on(async {
            let crawls = CrawlStore::temporary().expect("store");
            let catalog = connect_catalog(&crawls.path(0).with_file_name("catalog.db"))
                .await
                .expect("catalog");
            migrate(&catalog, &crawls, Some(TEXT_HASH_MIGRATION))
                .await
                .expect("migrations before hashing");
            let root = "https://a.test/";
            let crawl_id = create_crawl(&catalog, root, "http", &config())
                .await
                .expect("crawl");
            let headers: Vec<(String, String)> = (0..30)
                .map(|index| {
                    (
                        format!("x-header-{index}"),
                        "a repeated header value".to_string(),
                    )
                })
                .collect();
            for (id, text) in [(1, root), (2, "https://a.test/next"), (3, "next")] {
                sqlx::query("INSERT INTO texts (id, crawl_id, text) VALUES (?, ?, ?)")
                    .bind(id)
                    .bind(crawl_id)
                    .bind(text)
                    .execute(&catalog)
                    .await
                    .expect("text");
            }
            sqlx::query(
                "INSERT INTO pages (crawl_id, url_id, status, is_page, crawled_at, headers_json)
                 VALUES (?, 1, 200, 1, 1, ?)",
            )
            .bind(crawl_id)
            .bind(serde_json::to_string(&headers).expect("json"))
            .execute(&catalog)
            .await
            .expect("page");
            sqlx::query(
                "INSERT INTO links (crawl_id, src_url_id, dst_url_id, anchor_id, internal)
                 VALUES (?, 1, 2, 3, 1)",
            )
            .bind(crawl_id)
            .execute(&catalog)
            .await
            .expect("link");
            finish_crawl_in_catalog(&catalog, crawl_id).await;

            run_migrations(&catalog, &crawls).await.expect("migrations");

            assert!(crawls.path(crawl_id).exists());
            let left: Option<String> =
                sqlx::query_scalar("SELECT name FROM sqlite_master WHERE name = 'pages'")
                    .fetch_optional(&catalog)
                    .await
                    .expect("catalog tables");
            assert_eq!(left, None, "the catalog keeps no crawl rows");
            let db = crawls.open(crawl_id).await.expect("crawl file");
            let texts: Vec<(i64, String, Option<i64>)> =
                sqlx::query_as("SELECT id, text, hash FROM texts ORDER BY id")
                    .fetch_all(db.pool())
                    .await
                    .expect("texts");
            assert_eq!(texts.len(), 3);
            for (_, text, hash) in &texts {
                assert_eq!(*hash, Some(text_hash(text)));
            }
            let stored_type: String = sqlx::query_scalar("SELECT typeof(headers_json) FROM pages")
                .fetch_one(db.pool())
                .await
                .expect("type");
            assert_eq!(stored_type, "blob");

            let pages = load_pages_for_crawl(&db, root).await.expect("load");
            let page = pages
                .iter()
                .find(|page| page.url == root)
                .expect("root page");
            assert_eq!(page.headers.to_vec(), headers);
            assert_eq!(page.outlinks.len(), 1);
            assert_eq!(page.outlinks[0].anchor.as_deref(), Some("next"));
            // Moved crawls keep their finish: One Inlink trusts their counts.
            assert!(page.inlinks_complete);

            // A URL stored before the move is found again rather than stored
            // twice.
            insert_page(
                &db,
                &PageRecord {
                    url: root.to_string(),
                    ..Default::default()
                },
            )
            .await
            .expect("again");
            let stored: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM texts WHERE text = ?")
                .bind(root)
                .fetch_one(db.pool())
                .await
                .expect("count");
            assert_eq!(stored, 1);
        });
    }

    /// Marks a crawl finished the way the catalog did before crawls had files.
    async fn finish_crawl_in_catalog(catalog: &SqlitePool, crawl_id: i64) {
        sqlx::query("UPDATE crawls SET finished_at = 1 WHERE id = ?")
            .bind(crawl_id)
            .execute(catalog)
            .await
            .expect("finish");
    }

    /// A crawl left open by an app that went away is closed as aborted, with
    /// its counts, and a finished crawl is left as it was.
    #[test]
    fn an_unfinished_crawl_is_marked_aborted() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let crawls = runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let mut crawl_ids = Vec::new();
            for root in ["https://finished.test/", "https://cut-off.test/"] {
                let crawl_id = create_crawl(&pool, root, "http", &config())
                    .await
                    .expect("crawl");
                insert_page(
                    &crawls.open(crawl_id).await.expect("crawl file"),
                    &PageRecord {
                        url: root.to_string(),
                        status: Some(200),
                        is_page: true,
                        is_internal: true,
                        ..Default::default()
                    },
                )
                .await
                .expect("page");
                crawl_ids.push(crawl_id);
            }
            let [finished_id, _] = crawl_ids[..] else {
                panic!("two crawls");
            };
            finish_crawl(
                &pool,
                &crawls.open(finished_id).await.expect("crawl file"),
                finished_id,
            )
            .await
            .expect("finish");
            let aborted = abort_unfinished_crawls(&pool, &crawls)
                .await
                .expect("abort");
            assert_eq!(aborted, 1);
            list_crawls(&pool, &crawls).await.expect("list")
        });
        let state = |root: &str| {
            crawls
                .iter()
                .find(|crawl| crawl.root_url == root)
                .map(|crawl| (crawl.finished_at.is_some(), crawl.aborted, crawl.page_count))
        };
        assert_eq!(state("https://finished.test/"), Some((true, false, 1)));
        assert_eq!(state("https://cut-off.test/"), Some((true, true, 1)));
    }

    /// Stored counts are only for a finished crawl, only under the version
    /// they were counted with, and go with the crawl.
    #[test]
    fn filter_counts_are_stored_per_finished_crawl_and_version() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let (pool, crawls) = test_databases().await;
            let crawl_id = create_crawl(&pool, "https://a.test/", "http", &config())
                .await
                .expect("crawl");

            let stored = store_filter_counts(&pool, crawl_id, 1, "{}")
                .await
                .expect("store");
            assert!(!stored, "a running crawl's counts can still change");
            assert_eq!(
                load_filter_counts(&pool, crawl_id, 1).await.expect("load"),
                None
            );

            finish_crawl(
                &pool,
                &crawls.open(crawl_id).await.expect("crawl file"),
                crawl_id,
            )
            .await
            .expect("finish");
            assert!(
                store_filter_counts(&pool, crawl_id, 1, "{\"v\":1}")
                    .await
                    .expect("store")
            );
            assert_eq!(
                load_filter_counts(&pool, crawl_id, 1).await.expect("load"),
                Some("{\"v\":1}".to_string())
            );
            assert_eq!(
                load_filter_counts(&pool, crawl_id, 2).await.expect("load"),
                None,
                "counts from another version are counted again"
            );

            assert!(
                store_filter_counts(&pool, crawl_id, 2, "{\"v\":2}")
                    .await
                    .expect("store")
            );
            assert_eq!(
                load_filter_counts(&pool, crawl_id, 1).await.expect("load"),
                None
            );
            assert_eq!(
                load_filter_counts(&pool, crawl_id, 2).await.expect("load"),
                Some("{\"v\":2}".to_string())
            );

            assert!(crawls.path(crawl_id).exists());
            delete_crawl(&pool, &crawls, crawl_id)
                .await
                .expect("delete");
            assert!(
                !crawls.path(crawl_id).exists(),
                "the crawl's file is deleted"
            );
            let (left,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crawl_filter_counts")
                .fetch_one(&pool)
                .await
                .expect("count");
            assert_eq!(left, 0);
        });
    }
}
