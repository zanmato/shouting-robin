use sqlx::{Row, SqlitePool};

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
];

/// Stores every URL and repeated text once per crawl, in `texts`, and points
/// to it by id. It rewrites most of the database, which only gets smaller on
/// disk once it is vacuumed.
const TEXTS_MIGRATION: &str = "0036_texts";

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
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

    for (name, sql) in MIGRATIONS {
        let already: Option<(String,)> =
            sqlx::query_as("SELECT name FROM schema_migrations WHERE name = ?")
                .bind(name)
                .fetch_optional(pool)
                .await?;
        if already.is_some() {
            continue;
        }
        let mut tx = pool.begin().await?;
        for stmt in sql.split(";\n").map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO schema_migrations (name, applied_at) VALUES (?, ?)")
            .bind(name)
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        tracing::info!("Applied migration {}", name);
        if *name == TEXTS_MIGRATION {
            // The migration's result is kept whether or not this works: the
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
    }

    Ok(())
}

/// The ids of the texts one write stores. A page repeats its own URL on every
/// side table row and often the same anchor or alt text, so each is looked up
/// once per write.
struct TextIds<'a> {
    crawl_id: i64,
    known: std::collections::HashMap<&'a str, i64>,
}

impl<'a> TextIds<'a> {
    fn new(crawl_id: i64) -> Self {
        Self {
            crawl_id,
            known: std::collections::HashMap::new(),
        }
    }

    /// The text's id, stored first if the crawl has not met it yet. The
    /// insert comes first so the write holds SQLite's write lock before it
    /// reads, which keeps a concurrent write from slipping in between.
    async fn id(
        &mut self,
        conn: &mut sqlx::SqliteConnection,
        text: &'a str,
    ) -> Result<i64, sqlx::Error> {
        if let Some(&id) = self.known.get(text) {
            return Ok(id);
        }
        sqlx::query("INSERT OR IGNORE INTO texts (crawl_id, text) VALUES (?, ?)")
            .bind(self.crawl_id)
            .bind(text)
            .execute(&mut *conn)
            .await?;
        let id: i64 = sqlx::query_scalar("SELECT id FROM texts WHERE crawl_id = ? AND text = ?")
            .bind(self.crawl_id)
            .bind(text)
            .fetch_one(&mut *conn)
            .await?;
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
    async fn load(pool: &SqlitePool, crawl_id: i64) -> Result<Self, sqlx::Error> {
        use futures::TryStreamExt as _;
        let mut rows =
            sqlx::query_as::<_, (i64, String)>("SELECT id, text FROM texts WHERE crawl_id = ?")
                .bind(crawl_id)
                .fetch(pool);
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

/// Stores one page's layout with the key and style table entries it added,
/// in one transaction so a page never refers to entries that were not saved.
pub async fn save_page_layout(
    pool: &SqlitePool,
    crawl_id: i64,
    url: &str,
    encoded_layout: &[u8],
    added: &crate::crawl::layout::NewTableEntries,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (id, key) in &added.keys {
        sqlx::query("INSERT OR REPLACE INTO layout_keys (crawl_id, id, key) VALUES (?, ?, ?)")
            .bind(crawl_id)
            .bind(i64::from(*id))
            .bind(key)
            .execute(&mut *tx)
            .await?;
    }
    for (id, style) in &added.styles {
        let style_json = serde_json::to_string(style).unwrap_or_else(|_| "[]".to_string());
        sqlx::query(
            "INSERT OR REPLACE INTO layout_styles (crawl_id, id, style_json) VALUES (?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(i64::from(*id))
        .bind(style_json)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("INSERT OR REPLACE INTO page_layouts (crawl_id, url, layout) VALUES (?, ?, ?)")
        .bind(crawl_id)
        .bind(url)
        .bind(encoded_layout)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn load_layout_tables(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<crate::crawl::layout::LayoutTables, sqlx::Error> {
    let keys =
        sqlx::query_as::<_, (i64, String)>("SELECT id, key FROM layout_keys WHERE crawl_id = ?")
            .bind(crawl_id)
            .fetch_all(pool)
            .await?;
    let styles = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, style_json FROM layout_styles WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
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
                    tracing::warn!(error=%e, crawl_id, id, "stored layout style could not be parsed");
                    None
                }
            }
        })
        .collect();
    Ok(crate::crawl::layout::LayoutTables::from_rows(keys, styles))
}

/// One page's stored layout, still encoded.
pub async fn load_page_layout(
    pool: &SqlitePool,
    crawl_id: i64,
    url: &str,
) -> Result<Option<Vec<u8>>, sqlx::Error> {
    let row = sqlx::query_as::<_, (Vec<u8>,)>(
        "SELECT layout FROM page_layouts WHERE crawl_id = ? AND url = ?",
    )
    .bind(crawl_id)
    .bind(url)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(layout,)| layout))
}

/// A page of stored layouts in URL order, starting after `after_url`. Read
/// in pages so a pass over a large crawl never holds every layout at once.
pub async fn load_page_layouts_after(
    pool: &SqlitePool,
    crawl_id: i64,
    after_url: &str,
    limit: i64,
) -> Result<Vec<(String, Vec<u8>)>, sqlx::Error> {
    sqlx::query_as::<_, (String, Vec<u8>)>(
        "SELECT url, layout FROM page_layouts WHERE crawl_id = ? AND url > ? ORDER BY url LIMIT ?",
    )
    .bind(crawl_id)
    .bind(after_url)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// Replaces the crawl's templates and every page's place in them.
pub async fn save_layout_grouping(
    pool: &SqlitePool,
    crawl_id: i64,
    grouping: &crate::crawl::layout_groups::Grouping,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM layout_templates WHERE crawl_id = ?")
        .bind(crawl_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE page_layouts SET template_id = NULL, assigned_by = NULL WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .execute(&mut *tx)
    .await?;
    for template in &grouping.templates {
        sqlx::query(
            "INSERT INTO layout_templates (crawl_id, id, name, type_key, page_count) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(template.id)
        .bind(&template.name)
        .bind(&template.type_key)
        .bind(template.page_count as i64)
        .execute(&mut *tx)
        .await?;
    }
    for (url, (template_id, assignment)) in &grouping.pages {
        sqlx::query(
            "UPDATE page_layouts SET template_id = ?, assigned_by = ? WHERE crawl_id = ? AND url = ?",
        )
        .bind(template_id)
        .bind(assignment.as_str())
        .bind(crawl_id)
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn count_page_layouts(pool: &SqlitePool, crawl_id: i64) -> Result<i64, sqlx::Error> {
    let (count,) =
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM page_layouts WHERE crawl_id = ?")
            .bind(crawl_id)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

/// Marks a crawl finished and stores its row counts, which the crawl history
/// lists. Counting them per listing scanned every crawl's pages, seconds on a
/// database holding a few large crawls, and a finished crawl's rows no longer
/// change.
pub async fn finish_crawl(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query(
        r#"
        UPDATE crawls SET
            finished_at = ?,
            page_count = (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = crawls.id),
            document_count =
                (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = crawls.id AND p.is_page = 1),
            unfetched_count = (SELECT COUNT(*) FROM pages p
                WHERE p.crawl_id = crawls.id AND p.status IS NULL AND p.redirect_url IS NULL)
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(crawl_id)
    .execute(pool)
    .await?;
    Ok(())
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

pub async fn delete_crawl(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM crawls WHERE id = ?")
        .bind(crawl_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn insert_page(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
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

    // One transaction per page: a page with a hundred links and forty images
    // is otherwise two hundred autocommits, each its own fsync, and a crash
    // mid-page leaves a row with half its links.
    let mut transaction = pool.begin().await?;
    let mut texts = TextIds::new(crawl_id);
    let url_id = texts.id(&mut transaction, &record.url).await?;
    let inserted = sqlx::query(
        r#"
        INSERT INTO pages (
            crawl_id, url_id, status, content_type, size_bytes, response_time_ms,
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
            h2_non_sequential
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(crawl_id, url_id) DO NOTHING
        "#,
    )
    .bind(crawl_id)
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
    .bind(hreflang_json)
    .bind(sd_types_json)
    .bind(record.simhash.map(|h| h as i64))
    .bind(&record.og_type)
    .bind(record.a11y_errors as i64)
    .bind(record.a11y_warnings as i64)
    .bind(headers_json)
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
    .execute(&mut *transaction)
    .await?;
    // The first record of a URL wins: a page that Chrome later also reports
    // as a subresource of another page is already fully described.
    if inserted.rows_affected() == 0 {
        transaction.rollback().await?;
        return Ok(());
    }

    insert_structured_data(&mut transaction, crawl_id, url_id, record).await?;
    insert_performance(&mut transaction, crawl_id, url_id, record).await?;
    insert_images(&mut transaction, url_id, record, &mut texts).await?;
    insert_ecommerce(&mut transaction, crawl_id, url_id, record).await?;
    insert_links(&mut transaction, url_id, record, &mut texts).await?;
    insert_a11y_violations(&mut transaction, url_id, record, &mut texts).await?;
    insert_sd_issues(&mut transaction, crawl_id, url_id, record).await?;

    transaction.commit().await
}

async fn insert_structured_data(
    conn: &mut sqlx::SqliteConnection,
    crawl_id: i64,
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
            "INSERT INTO structured_data (crawl_id, page_url_id, format, type_name, json, errors) VALUES (?, ?, ?, ?, ?, NULL)",
        )
        .bind(crawl_id)
        .bind(url_id)
        .bind(format_str)
        .bind(&item.type_name)
        .bind(&item.raw_json)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

async fn insert_performance(
    conn: &mut sqlx::SqliteConnection,
    crawl_id: i64,
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
        "INSERT INTO performance (crawl_id, page_url_id, lcp_ms, cls, fcp_ms, ttfb_ms, transfer_kb) VALUES (?, ?, ?, ?, ?, ?, NULL)",
    )
    .bind(crawl_id)
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
            "INSERT INTO images (crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(texts.crawl_id)
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
    crawl_id: i64,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    let Some(ref audit) = record.ecommerce else {
        return Ok(());
    };
    sqlx::query(
        r#"
        INSERT INTO ecommerce (
            crawl_id, page_url_id, page_type, price, currency, availability,
            sku, gtin, brand, has_image, has_description, has_review_or_rating
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(crawl_id)
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
            "INSERT INTO links (crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(texts.crawl_id)
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
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id WHERE p.crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// The URLs of a crawl that need no further request: every page, and every
/// resource whose response the crawl actually saw. A resource Chrome reported
/// through the Resource Timing API with no size and no headers is a
/// cross-origin asset the browser was not allowed to describe, and is worth
/// a HEAD of its own.
pub async fn load_described_page_urls(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<std::collections::HashSet<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE p.crawl_id = ?
          AND NOT (p.is_resource = 1 AND p.size_bytes = 0 AND p.headers_json IS NULL)
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// The resources Chrome reported without describing: rows from the Resource
/// Timing API with no size and no headers, as (url, initiator).
pub async fn load_undescribed_resources(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    let rows: Vec<(String, Option<String>)> = sqlx::query_as(
        r#"
        SELECT t.text, p.resource_initiator FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE p.crawl_id = ? AND p.is_resource = 1 AND p.size_bytes = 0
          AND p.headers_json IS NULL
        ORDER BY p.id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(url, initiator)| (url, initiator.unwrap_or_default()))
        .collect())
}

/// Records the outcome of a resource check. Updates the row a Resource Timing
/// entry already created for the URL, otherwise inserts it.
pub async fn record_resource_check(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
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
        WHERE crawl_id = ? AND is_resource = 1
          AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)
        "#,
    )
    .bind(record.status.map(|s| s as i64))
    .bind(record.size_bytes as i64)
    .bind(record.size_bytes as i64)
    .bind(record.content_type.as_deref())
    .bind(headers_json)
    .bind(record.redirect_url.as_deref())
    .bind(record.redirect_status.map(|s| s as i64))
    .bind(record.indexability.as_deref().unwrap_or("N/A"))
    .bind(crawl_id)
    .bind(crawl_id)
    .bind(&record.url)
    .execute(pool)
    .await?;
    if updated.rows_affected() == 0 {
        insert_page(pool, crawl_id, record).await?;
    }
    Ok(())
}

/// Every document of a crawl that declares a canonical, as (url, canonical).
/// The canonical is returned exactly as the page wrote it, so the caller
/// resolves it against the page's own URL before doing anything with it.
pub async fn load_declared_canonicals(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT t.text, p.canonical FROM pages p JOIN texts t ON t.id = p.url_id
         WHERE p.crawl_id = ? AND p.is_page = 1 AND p.canonical IS NOT NULL
           AND trim(p.canonical) != ''",
    )
    .bind(crawl_id)
    .fetch_all(pool)
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
pub async fn load_redirects(pool: &SqlitePool, crawl_id: i64) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT t.text FROM pages p JOIN texts t ON t.id = p.url_id
         WHERE p.crawl_id = ?
           AND (p.redirect_url IS NOT NULL OR (p.status >= 300 AND p.status < 400))
         ORDER BY t.text",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

/// Records where a redirect went, once we have resolved it ourselves.
pub async fn set_redirect_target(
    pool: &SqlitePool,
    crawl_id: i64,
    url: &str,
    target: &str,
    status: u16,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE pages SET redirect_url = ?, redirect_status = ?
         WHERE crawl_id = ? AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)",
    )
    .bind(target)
    .bind(status as i64)
    .bind(crawl_id)
    .bind(crawl_id)
    .bind(url)
    .execute(pool)
    .await?;
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
            "INSERT INTO a11y_violations (crawl_id, page_url_id, rule_id, impact_id, target_id, html_id) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(texts.crawl_id)
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
    crawl_id: i64,
    url_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for issue in &record.sd_issues {
        let severity_str = match issue.severity {
            SdSeverity::Error => "error",
            SdSeverity::Warning => "warning",
        };
        sqlx::query(
            "INSERT INTO sd_issues (crawl_id, page_url_id, severity, type_name, code, message) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
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

pub async fn update_link_scores(
    pool: &SqlitePool,
    crawl_id: i64,
    scores: &[(String, f32)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (url, score) in scores {
        sqlx::query(
            "UPDATE pages SET link_score = ?
             WHERE crawl_id = ? AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)",
        )
        .bind(*score)
        .bind(crawl_id)
        .bind(crawl_id)
        .bind(url)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn update_crawl_depths(
    pool: &SqlitePool,
    crawl_id: i64,
    depths: &[(String, u32)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (url, depth) in depths {
        sqlx::query(
            "UPDATE pages SET depth = ?
             WHERE crawl_id = ? AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)",
        )
        .bind(*depth as i64)
        .bind(crawl_id)
        .bind(crawl_id)
        .bind(url)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn update_hreflang_issues(
    pool: &SqlitePool,
    crawl_id: i64,
    issues: &[(String, Vec<crate::crawl::event::HreflangIssue>)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (url, page_issues) in issues {
        let json = if page_issues.is_empty() {
            None
        } else {
            serde_json::to_string(page_issues).ok()
        };
        sqlx::query(
            "UPDATE pages SET hreflang_issues_json = ?
             WHERE crawl_id = ? AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)",
        )
        .bind(json)
        .bind(crawl_id)
        .bind(crawl_id)
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

pub async fn load_page_on_demand(
    pool: &SqlitePool,
    crawl_id: i64,
    url: &str,
) -> Result<PageOnDemand, sqlx::Error> {
    let backlinks = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        r#"
        SELECT s.text, a.text, r.text FROM links l
        JOIN texts s ON s.id = l.src_url_id
        LEFT JOIN texts a ON a.id = l.anchor_id
        LEFT JOIN texts r ON r.id = l.rel_id
        WHERE l.crawl_id = ?
          AND l.dst_url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)
        ORDER BY l.id
        "#,
    )
    .bind(crawl_id)
    .bind(crawl_id)
    .bind(url)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(source_url, anchor, rel)| crate::crawl::event::Backlink {
        source_url,
        anchor,
        rel,
    })
    .collect();
    let sd_items = sqlx::query_as::<_, (String, String, String)>(
        "SELECT format, type_name, json FROM structured_data \
         WHERE crawl_id = ? \
           AND page_url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?) \
         ORDER BY id",
    )
    .bind(crawl_id)
    .bind(crawl_id)
    .bind(url)
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|(format, type_name, raw_json)| SdItem {
        format: sd_format(&format),
        type_name,
        raw_json,
    })
    .collect();
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
/// baseline lookup describe a crawl the same way.
// Correlated subqueries rather than a join: a join grouped by crawl
// aggregates every page of every crawl ever made on each sidebar refresh,
// where these only touch the pages of the crawls the outer query selects.
// A finished crawl's counts are stored by `finish_crawl`. COALESCE only runs
// the subquery when they are missing, for a crawl still running or one that
// ended without finishing.
const CRAWL_ROW_COLUMNS: &str = r#"
    c.id, c.root_url, c.started_at, c.finished_at,
    COALESCE(c.page_count, (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = c.id)) as page_count,
    COALESCE(c.document_count,
        (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = c.id AND p.is_page = 1)) as document_count,
    COALESCE(c.unfetched_count, (SELECT COUNT(*) FROM pages p
     WHERE p.crawl_id = c.id AND p.status IS NULL AND p.redirect_url IS NULL)) as unfetched_count,
    c.render_mode,
    json_extract(c.config_json, '$.layout_capture') as layout_capture
"#;

type CrawlRowColumns = (
    i64,
    String,
    i64,
    Option<i64>,
    i64,
    i64,
    i64,
    String,
    Option<String>,
);

fn crawl_row(columns: CrawlRowColumns) -> CrawlRow {
    let (
        id,
        root_url,
        started_at,
        finished_at,
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
    CrawlRow {
        id,
        root_url,
        started_at,
        finished_at,
        page_count,
        document_count,
        unfetched_count,
        render_mode,
        layout_capture,
    }
}

pub async fn list_crawls(pool: &SqlitePool) -> Result<Vec<CrawlRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CrawlRowColumns>(&format!(
        r#"
        SELECT {CRAWL_ROW_COLUMNS}
        FROM crawls c
        ORDER BY c.started_at DESC
        LIMIT 100
        "#
    ))
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(crawl_row).collect())
}

/// Finds the crawl to use as a comparison baseline for a given root URL.
///
/// When `current_crawl_id` is `Some`, returns the most recent crawl of the same
/// `root_url` that started strictly before the given crawl (the sidebar flow).
/// When `None`, the current crawl is assumed to be the most recent row for the
/// URL (a just-finished live crawl), so we skip it with `OFFSET 1`.
pub async fn find_previous_crawl(
    pool: &SqlitePool,
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
            .fetch_optional(pool)
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
            .fetch_optional(pool)
            .await?
        }
    };

    Ok(row.map(crawl_row))
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
    pool: &SqlitePool,
    crawl_id: i64,
    root_url: &str,
) -> Result<Vec<PageRecord>, sqlx::Error> {
    load_pages_for_crawl_reporting(pool, crawl_id, root_url, |_| {}).await
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
    pool: &SqlitePool,
    crawl_id: i64,
    root_url: &str,
    report: impl Fn(LoadStep),
) -> Result<Vec<PageRecord>, sqlx::Error> {
    let report = &report;
    let texts = std::sync::Arc::new(CrawlTexts::load(pool, crawl_id).await?);
    report(LoadStep::Texts);
    let links = async_std::task::spawn({
        let pool = pool.clone();
        let texts = texts.clone();
        async move {
            use futures::TryStreamExt as _;
            let mut rows = sqlx::query_as::<_, (i64, i64, Option<i64>, Option<i64>, i64)>(
                r#"
                SELECT src_url_id, dst_url_id, anchor_id, rel_id, csr_only
                FROM links WHERE crawl_id = ?
                ORDER BY id
                "#,
            )
            .bind(crawl_id)
            .fetch(&pool);
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
        let pool = pool.clone();
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
                load_performance_rows(&pool, crawl_id),
                load_structured_data_rows(&pool, crawl_id),
                load_images_by_url(&pool, crawl_id, &texts),
                load_a11y_by_url(&pool, crawl_id, &texts),
                load_sd_issue_rows(&pool, crawl_id),
                load_layout_rows(&pool, crawl_id),
                load_sitemap_rows(&pool, crawl_id),
                load_ecommerce_rows(&pool, crawl_id),
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
        let pool = pool.clone();
        async move {
            let rows = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
                r#"
                SELECT dst_url_id,
                       COUNT(*),
                       COUNT(DISTINCT src_url_id),
                       COALESCE(SUM(csr_only), 0),
                       COUNT(DISTINCT CASE WHEN csr_only = 1 THEN src_url_id END)
                FROM links WHERE crawl_id = ? GROUP BY dst_url_id
                "#,
            )
            .bind(crawl_id)
            .fetch_all(&pool)
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
        let pool = pool.clone();
        let texts = texts.clone();
        let root_url = root_url.to_string();
        async move {
            use futures::TryStreamExt as _;
            let mut rows = sqlx::query(
                r#"
                SELECT url_id, status, title, size_bytes, content_type, meta_description, h1, h2,
                       canonical, word_count, depth, indexability, response_time_ms, og_type,
                       a11y_errors, a11y_warnings, headers_json, robots, is_resource,
                       resource_initiator, is_page, has_mixed_content, has_body_tag, redirect_url,
                       redirect_status, title_2, meta_description_2, h1_2, h2_2, title_pixel_width,
                       meta_description_pixel_width, ssr_word_count, ssr_h1, ssr_content_missing,
                       blocked_by_robots, h2_non_sequential, hash, simhash, closest_similarity,
                       near_duplicate_count, near_duplicate_urls_json, sd_errors, sd_warnings,
                       hreflang_tags_json, sd_types_json, hreflang_issues_json, hreflang_sources_json,
                       link_score
                FROM pages WHERE crawl_id = ?
                ORDER BY id
                "#,
            )
            .bind(crawl_id)
            .fetch(&pool);
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
        hreflang_tags: json_list(row.try_get::<Option<&str>, _>("hreflang_tags_json")?),
        hreflang_issues: json_list(row.try_get::<Option<&str>, _>("hreflang_issues_json")?),
        hreflang_sources: json_list(row.try_get::<Option<&str>, _>("hreflang_sources_json")?),
        content_hash: row.try_get("hash")?,
        simhash: simhash.map(|h| h as u64),
        closest_similarity: closest_similarity.map(|s| s as u8),
        near_duplicate_count: near_duplicate_count.map(|c| c as u32),
        near_duplicate_urls: json_list(row.try_get::<Option<&str>, _>("near_duplicate_urls_json")?),
        og_type: row.try_get("og_type")?,
        a11y_errors: a11y_errors as u32,
        a11y_warnings: a11y_warnings as u32,
        headers: json_list(row.try_get::<Option<&str>, _>("headers_json")?),
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

async fn load_performance_rows(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<PerformanceRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, ttfb_ms, lcp_ms, cls, fcp_ms FROM performance WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
}

/// Every structured data item of a crawl, without its JSON: only the details
/// panel shows that, and it reads it per page, see [`load_page_on_demand`].
async fn load_structured_data_rows(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(i64, String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, format, type_name FROM structured_data WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch_all(pool)
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
    pool: &SqlitePool,
    crawl_id: i64,
    texts: &CrawlTexts,
) -> Result<std::collections::HashMap<i64, Vec<ImageRef>>, sqlx::Error> {
    use futures::TryStreamExt as _;
    let mut rows = sqlx::query_as::<_, ImageRow>(
        "SELECT page_url_id, src_id, alt_id, width, height, has_alt_attr FROM images \
         WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch(pool);
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
    pool: &SqlitePool,
    crawl_id: i64,
    texts: &CrawlTexts,
) -> Result<std::collections::HashMap<i64, Vec<A11yIssue>>, sqlx::Error> {
    use futures::TryStreamExt as _;
    let mut rows = sqlx::query_as::<_, (i64, i64, i64, Option<i64>, Option<i64>)>(
        "SELECT page_url_id, rule_id, impact_id, target_id, html_id FROM a11y_violations \
         WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch(pool);
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
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(i64, String, String, String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, severity, type_name, code, message FROM sd_issues \
         WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
}

async fn load_layout_rows(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(String, Option<String>, Option<String>, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT pl.url, t.name, t.type_key, pl.assigned_by
        FROM page_layouts pl
        LEFT JOIN layout_templates t ON t.crawl_id = pl.crawl_id AND t.id = pl.template_id
        WHERE pl.crawl_id = ?
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
}

async fn load_sitemap_rows(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(i64, i64, Option<i64>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT page_url_id, sitemap_url_id, lastmod_id FROM sitemap_urls \
         WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
}

async fn load_ecommerce_rows(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<EcommerceRow>, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT page_url_id, page_type, price, currency, availability,
               sku, gtin, brand, has_image, has_description, has_review_or_rating
        FROM ecommerce WHERE crawl_id = ?
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
}

pub async fn load_simhashes_for_crawl(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(String, u64, Option<String>)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, Option<i64>, Option<String>)>(
        r#"
        SELECT t.text, p.simhash, p.hash
        FROM pages p JOIN texts t ON t.id = p.url_id
        WHERE p.crawl_id = ? AND p.simhash IS NOT NULL
        ORDER BY p.id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .filter_map(|(url, simhash, content_hash)| simhash.map(|sh| (url, sh as u64, content_hash)))
        .collect())
}

pub async fn update_near_duplicates(
    pool: &SqlitePool,
    crawl_id: i64,
    results: &[(String, u8, u32, Vec<String>)],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    for (url, similarity, count, dup_urls) in results {
        let dup_urls_json = if dup_urls.is_empty() {
            None
        } else {
            serde_json::to_string(dup_urls).ok()
        };
        sqlx::query(
            "UPDATE pages SET closest_similarity = ?, near_duplicate_count = ?, near_duplicate_urls_json = ?
             WHERE crawl_id = ? AND url_id = (SELECT id FROM texts WHERE crawl_id = ? AND text = ?)",
        )
        .bind(*similarity as i64)
        .bind(*count as i64)
        .bind(dup_urls_json)
        .bind(crawl_id)
        .bind(crawl_id)
        .bind(url)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn insert_sitemap_urls(
    pool: &SqlitePool,
    crawl_id: i64,
    sitemap_urls: &[crate::crawl::sitemap::SitemapUrl],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut texts = TextIds::new(crawl_id);
    for entry in sitemap_urls {
        let sitemap_id = texts.id(&mut tx, &entry.sitemap_url).await?;
        let page_id = texts.id(&mut tx, &entry.page_url).await?;
        let lastmod_id = texts.optional_id(&mut tx, entry.lastmod.as_deref()).await?;
        sqlx::query(
            "INSERT INTO sitemap_urls (crawl_id, sitemap_url_id, page_url_id, lastmod_id) VALUES (?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(sitemap_id)
        .bind(page_id)
        .bind(lastmod_id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct SitemapStatus {
    pub page_url: String,
    pub sitemap_url: String,
    pub lastmod: Option<String>,
}

pub async fn load_sitemap_orphans(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<SitemapStatus>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        r#"
        SELECT page.text, sitemap.text, lastmod.text
        FROM sitemap_urls su
        JOIN texts page ON page.id = su.page_url_id
        JOIN texts sitemap ON sitemap.id = su.sitemap_url_id
        LEFT JOIN texts lastmod ON lastmod.id = su.lastmod_id
        LEFT JOIN pages p ON p.crawl_id = su.crawl_id AND p.url_id = su.page_url_id
        WHERE su.crawl_id = ? AND p.id IS NULL
        ORDER BY su.id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
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

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").expect("pool");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("migrations table");
        run_migrations(&pool).await.expect("migrations");
        pool
    }

    fn link(dst_url: &str, anchor: &str) -> crate::crawl::event::Outlink {
        crate::crawl::event::Outlink {
            dst_url: dst_url.into(),
            anchor: Some(anchor.into()),
            rel: None,
            csr_only: false,
        }
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
            let pool = test_pool().await;
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
                    &pool,
                    crawl_id,
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
            load_page_on_demand(&pool, crawl_id, "https://a.test/target")
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
            let pool = test_pool().await;
            let root = "https://a.test/";
            let crawl_id = create_crawl(&pool, root, "http", &config())
                .await
                .expect("crawl");
            insert_page(
                &pool,
                crawl_id,
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
            let pages = load_pages_for_crawl_reporting(&pool, crawl_id, root, |step| {
                steps.borrow_mut().push(step)
            })
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
            let pool = test_pool().await;
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
                &pool,
                crawl_id,
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
            let loaded = load_pages_for_crawl(&pool, crawl_id, root)
                .await
                .expect("load");
            let on_demand = load_page_on_demand(&pool, crawl_id, root)
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
            let pool = test_pool().await;
            let mut crawl_ids = Vec::new();
            for root in ["https://finished.test/", "https://running.test/"] {
                let crawl_id = create_crawl(&pool, root, "http", &config())
                    .await
                    .expect("crawl");
                for (path, status) in [("", Some(200)), ("unfetched", None)] {
                    insert_page(
                        &pool,
                        crawl_id,
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
            finish_crawl(&pool, finished_id).await.expect("finish");
            // Rows written after the finish are not what the finished crawl
            // found, so its listing keeps the stored counts.
            for crawl_id in [finished_id, running_id] {
                insert_page(
                    &pool,
                    crawl_id,
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
            let crawls = list_crawls(&pool).await.expect("list");
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

    /// Stored counts are only for a finished crawl, only under the version
    /// they were counted with, and go with the crawl.
    #[test]
    fn filter_counts_are_stored_per_finished_crawl_and_version() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        runtime.block_on(async {
            let pool = test_pool().await;
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

            finish_crawl(&pool, crawl_id).await.expect("finish");
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

            delete_crawl(&pool, crawl_id).await.expect("delete");
            let (left,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM crawl_filter_counts")
                .fetch_one(&pool)
                .await
                .expect("count");
            assert_eq!(left, 0);
        });
    }
}
