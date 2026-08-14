use sqlx::SqlitePool;

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
];

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
    }

    Ok(())
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

pub async fn finish_crawl(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().timestamp();
    sqlx::query("UPDATE crawls SET finished_at = ? WHERE id = ?")
        .bind(now)
        .bind(crawl_id)
        .execute(pool)
        .await?;
    Ok(())
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
        serde_json::to_string(&record.hreflang_tags).ok()
    };
    let hreflang_sources_json = if record.hreflang_sources.is_empty() {
        None
    } else {
        serde_json::to_string(&record.hreflang_sources).ok()
    };
    let sd_types_json = if record.sd_types.is_empty() {
        None
    } else {
        serde_json::to_string(&record.sd_types).ok()
    };
    let headers_json = if record.headers.is_empty() {
        None
    } else {
        serde_json::to_string(&record.headers).ok()
    };

    sqlx::query(
        r#"
        INSERT INTO pages (
            crawl_id, url, status, content_type, size_bytes, response_time_ms,
            depth, title, meta_description, h1, h2, canonical, robots,
            indexability, word_count, hash, crawled_at,
            sd_errors, sd_warnings, hreflang_tags_json, sd_types_json,
            simhash, og_type, a11y_errors, a11y_warnings, headers_json,
            redirect_url, redirect_status,
            title_2, meta_description_2, h1_2, h2_2,
            title_pixel_width, meta_description_pixel_width,
            ssr_word_count, ssr_h1, ssr_content_missing,
            blocked_by_robots, is_resource, resource_initiator, is_page,
            has_mixed_content, hreflang_sources_json
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(crawl_id)
    .bind(&record.url)
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
    .execute(pool)
    .await?;

    insert_structured_data(pool, crawl_id, record).await?;
    insert_performance(pool, crawl_id, record).await?;
    insert_images(pool, crawl_id, record).await?;
    insert_ecommerce(pool, crawl_id, record).await?;
    insert_links(pool, crawl_id, record).await?;
    insert_a11y_violations(pool, crawl_id, record).await?;
    insert_sd_issues(pool, crawl_id, record).await?;

    Ok(())
}

async fn insert_structured_data(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    use crate::crawl::event::SdFormat;
    for item in &record.sd_items {
        let format_str = match item.format {
            SdFormat::JsonLd => "json-ld",
            SdFormat::Microdata => "microdata",
        };
        sqlx::query(
            "INSERT INTO structured_data (crawl_id, page_url, format, type_name, json, errors) VALUES (?, ?, ?, ?, ?, NULL)",
        )
        .bind(crawl_id)
        .bind(&record.url)
        .bind(format_str)
        .bind(&item.type_name)
        .bind(&item.raw_json)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_performance(
    pool: &SqlitePool,
    crawl_id: i64,
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
        "INSERT INTO performance (crawl_id, page_url, lcp_ms, cls, fcp_ms, ttfb_ms, transfer_kb) VALUES (?, ?, ?, ?, ?, ?, NULL)",
    )
    .bind(crawl_id)
    .bind(&record.url)
    .bind(record.lcp_ms.map(|ms| ms as i64))
    .bind(record.cls)
    .bind(record.fcp_ms.map(|ms| ms as i64))
    .bind(record.ttfb_ms.map(|ms| ms as i64))
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_images(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for img in &record.images {
        sqlx::query(
            "INSERT INTO images (crawl_id, page_url, src, alt, width, height, has_alt_attr) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(&record.url)
        .bind(&img.src)
        .bind(img.alt.as_deref())
        .bind(img.width.map(|w| w as i64))
        .bind(img.height.map(|h| h as i64))
        .bind(img.has_alt_attr as i64)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_ecommerce(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    let Some(ref audit) = record.ecommerce else {
        return Ok(());
    };
    sqlx::query(
        r#"
        INSERT INTO ecommerce (
            crawl_id, page_url, page_type, price, currency, availability,
            sku, gtin, brand, has_image, has_description, has_review_or_rating
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(crawl_id)
    .bind(&record.url)
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
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_links(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for link in &record.outlinks {
        let kind = if crate::crawl::engine::is_same_domain(&record.url, &link.dst_url) {
            "internal"
        } else {
            "external"
        };
        sqlx::query(
            "INSERT INTO links (crawl_id, src_url, dst_url, anchor, rel, kind, csr_only) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(&record.url)
        .bind(&link.dst_url)
        .bind(link.anchor.as_deref())
        .bind(link.rel.as_deref())
        .bind(kind)
        .bind(link.csr_only as i64)
        .execute(pool)
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
    let rows: Vec<(String,)> = sqlx::query_as("SELECT url FROM pages WHERE crawl_id = ?")
        .bind(crawl_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(url,)| url).collect())
}

#[allow(dead_code)]
pub async fn compute_inlink_counts(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<std::collections::HashMap<String, u32>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT dst_url, COUNT(*) as cnt FROM links WHERE crawl_id = ? GROUP BY dst_url",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(url, count)| (url, count as u32))
        .collect())
}

async fn insert_a11y_violations(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for issue in &record.a11y_issues {
        sqlx::query(
            "INSERT INTO a11y_violations (crawl_id, page_url, rule, impact, target, html) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(&record.url)
        .bind(&issue.rule)
        .bind(&issue.impact)
        .bind(issue.target.as_deref())
        .bind(issue.html.as_deref())
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn insert_sd_issues(
    pool: &SqlitePool,
    crawl_id: i64,
    record: &PageRecord,
) -> Result<(), sqlx::Error> {
    for issue in &record.sd_issues {
        let severity_str = match issue.severity {
            SdSeverity::Error => "error",
            SdSeverity::Warning => "warning",
        };
        sqlx::query(
            "INSERT INTO sd_issues (crawl_id, page_url, severity, type_name, code, message) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(&record.url)
        .bind(severity_str)
        .bind(&issue.type_name)
        .bind(&issue.code)
        .bind(&issue.message)
        .execute(pool)
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
        sqlx::query("UPDATE pages SET link_score = ? WHERE crawl_id = ? AND url = ?")
            .bind(*score)
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
        sqlx::query("UPDATE pages SET depth = ? WHERE crawl_id = ? AND url = ?")
            .bind(*depth as i64)
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
        sqlx::query("UPDATE pages SET hreflang_issues_json = ? WHERE crawl_id = ? AND url = ?")
            .bind(json)
            .bind(crawl_id)
            .bind(url)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn load_backlinks_for_crawl(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<std::collections::HashMap<String, Vec<crate::crawl::event::Backlink>>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT dst_url, src_url, anchor, rel FROM links WHERE crawl_id = ? AND kind = 'internal'",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let mut backlinks: std::collections::HashMap<String, Vec<crate::crawl::event::Backlink>> =
        std::collections::HashMap::new();
    for (dst_url, src_url, anchor, rel) in rows {
        backlinks
            .entry(dst_url)
            .or_default()
            .push(crate::crawl::event::Backlink {
                source_url: src_url,
                anchor,
                rel,
            });
    }
    Ok(backlinks)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageAggregate {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub has_alt_attr: bool,
    pub used_on: Vec<String>,
}

#[allow(dead_code)]
pub async fn load_image_refs(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<ImageAggregate>, sqlx::Error> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            Option<i64>,
            Option<i64>,
            i64,
            String,
        ),
    >(
        r#"
        SELECT src, alt, width, height, has_alt_attr, page_url
        FROM images WHERE crawl_id = ?
        ORDER BY src, page_url
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    #[allow(clippy::type_complexity)]
    let mut map: std::collections::HashMap<
        String,
        (Option<String>, Option<i64>, Option<i64>, bool, Vec<String>),
    > = std::collections::HashMap::new();
    for (src, alt, width, height, has_alt_attr, page_url) in &rows {
        let entry = map
            .entry(src.clone())
            .or_insert_with(|| (alt.clone(), *width, *height, *has_alt_attr != 0, Vec::new()));
        entry.4.push(page_url.clone());
    }

    Ok(map
        .into_iter()
        .map(
            |(src, (alt, width, height, has_alt_attr, used_on))| ImageAggregate {
                src,
                alt,
                width: width.map(|w| w as u32),
                height: height.map(|h| h as u32),
                has_alt_attr,
                used_on,
            },
        )
        .collect())
}

#[derive(Debug, Clone)]
pub struct CrawlRow {
    pub id: i64,
    pub root_url: String,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub page_count: i64,
    pub render_mode: String,
}

pub async fn list_crawls(pool: &SqlitePool) -> Result<Vec<CrawlRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, i64, Option<i64>, i64, String)>(
        r#"
        SELECT c.id, c.root_url, c.started_at, c.finished_at,
               COUNT(p.id) as page_count, c.render_mode
        FROM crawls c
        LEFT JOIN pages p ON p.crawl_id = c.id
        GROUP BY c.id
        ORDER BY c.started_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, root_url, started_at, finished_at, page_count, render_mode)| CrawlRow {
                id,
                root_url,
                started_at,
                finished_at,
                page_count,
                render_mode,
            },
        )
        .collect())
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
            sqlx::query_as::<_, (i64, String, i64, Option<i64>, i64, String)>(
                r#"
                SELECT c.id, c.root_url, c.started_at, c.finished_at,
                       COUNT(p.id) as page_count, c.render_mode
                FROM crawls c
                LEFT JOIN pages p ON p.crawl_id = c.id
                WHERE c.root_url = ?
                  AND c.started_at < (SELECT started_at FROM crawls WHERE id = ?)
                GROUP BY c.id
                ORDER BY c.started_at DESC
                LIMIT 1
                "#,
            )
            .bind(root_url)
            .bind(id)
            .fetch_optional(pool)
            .await?
        }
        None => {
            sqlx::query_as::<_, (i64, String, i64, Option<i64>, i64, String)>(
                r#"
                SELECT c.id, c.root_url, c.started_at, c.finished_at,
                       COUNT(p.id) as page_count, c.render_mode
                FROM crawls c
                LEFT JOIN pages p ON p.crawl_id = c.id
                WHERE c.root_url = ?
                GROUP BY c.id
                ORDER BY c.started_at DESC
                LIMIT 1 OFFSET 1
                "#,
            )
            .bind(root_url)
            .fetch_optional(pool)
            .await?
        }
    };

    Ok(row.map(
        |(id, root_url, started_at, finished_at, page_count, render_mode)| CrawlRow {
            id,
            root_url,
            started_at,
            finished_at,
            page_count,
            render_mode,
        },
    ))
}

pub async fn load_pages_for_crawl(
    pool: &SqlitePool,
    crawl_id: i64,
    root_url: &str,
) -> Result<Vec<PageRecord>, sqlx::Error> {
    let base_rows = sqlx::query_as::<
        _,
        (
            String,
            Option<i64>,
            Option<String>,
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<i64>,
            Option<String>,
            i64,
            i64,
        ),
    >(
        r#"
        SELECT url, status, title, size_bytes, content_type,
               meta_description, h1, h2, canonical, word_count,
               depth, indexability, response_time_ms, og_type, a11y_errors, a11y_warnings
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let header_rows = sqlx::query_as::<_, (String, Option<String>)>(
        r#"
        SELECT url, headers_json
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let robots_rows = sqlx::query_as::<_, (String, Option<String>)>(
        r#"
        SELECT url, robots
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let resource_rows = sqlx::query_as::<_, (String, i64, Option<String>, i64, i64)>(
        r#"
        SELECT url, is_resource, resource_initiator, is_page, has_mixed_content
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let outlink_rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, i64)>(
        r#"
        SELECT src_url, dst_url, anchor, rel, csr_only
        FROM links WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let redirect_rows = sqlx::query_as::<_, (String, Option<String>, Option<i64>)>(
        r#"
        SELECT url, redirect_url, redirect_status
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let secondary_rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<i64>,
            Option<i64>,
        ),
    >(
        r#"
        SELECT url, title_2, meta_description_2, h1_2, h2_2,
               title_pixel_width, meta_description_pixel_width,
               ssr_word_count, ssr_h1, ssr_content_missing,
               blocked_by_robots
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let near_dup_rows = sqlx::query_as::<
        _,
        (
            String,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
        ),
    >(
        r#"
        SELECT url, hash, simhash, closest_similarity, near_duplicate_count, near_duplicate_urls_json
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let sd_meta_rows = sqlx::query_as::<
        _,
        (
            String,
            i64,
            i64,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(
        r#"
        SELECT url, sd_errors, sd_warnings, hreflang_tags_json, sd_types_json, hreflang_issues_json,
               hreflang_sources_json
        FROM pages WHERE crawl_id = ?
        ORDER BY id
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let perf_rows =
        sqlx::query_as::<_, (String, Option<i64>, Option<i64>, Option<f64>, Option<i64>)>(
            r#"
        SELECT page_url, ttfb_ms, lcp_ms, cls, fcp_ms
        FROM performance WHERE crawl_id = ?
        "#,
        )
        .bind(crawl_id)
        .fetch_all(pool)
        .await?;

    let sd_type_rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT page_url, format, type_name, json FROM structured_data WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let img_rows = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<i64>,
            i64,
        ),
    >(
        "SELECT page_url, src, alt, width, height, has_alt_attr FROM images WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let a11y_rows = sqlx::query_as::<_, (String, String, String, Option<String>, Option<String>)>(
        "SELECT page_url, rule, impact, target, html FROM a11y_violations WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let sd_issue_rows = sqlx::query_as::<_, (String, String, String, String, String)>(
        "SELECT page_url, severity, type_name, code, message FROM sd_issues WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    #[allow(clippy::type_complexity)]
    let mut perf_by_url: std::collections::HashMap<
        String,
        (Option<i64>, Option<i64>, Option<f64>, Option<i64>),
    > = std::collections::HashMap::new();
    for (url, ttfb, lcp, cls, fcp) in &perf_rows {
        perf_by_url.insert(url.clone(), (*ttfb, *lcp, *cls, *fcp));
    }

    let mut sd_by_url: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut sd_items_by_url: std::collections::HashMap<String, Vec<SdItem>> =
        std::collections::HashMap::new();
    for (url, format, type_name, raw_json) in &sd_type_rows {
        sd_by_url
            .entry(url.clone())
            .or_default()
            .push((format.clone(), type_name.clone()));
        if let Some(json) = raw_json {
            let sd_format = match format.as_str() {
                "microdata" => SdFormat::Microdata,
                _ => SdFormat::JsonLd,
            };
            sd_items_by_url
                .entry(url.clone())
                .or_default()
                .push(SdItem {
                    format: sd_format,
                    type_name: type_name.clone(),
                    raw_json: json.clone(),
                });
        }
    }

    let mut images_by_url: std::collections::HashMap<String, Vec<ImageRef>> =
        std::collections::HashMap::new();
    for (page_url, src, alt, width, height, has_alt_attr) in &img_rows {
        images_by_url
            .entry(page_url.clone())
            .or_default()
            .push(ImageRef {
                src: src.clone(),
                alt: alt.clone(),
                width: width.map(|w| w as u32),
                height: height.map(|h| h as u32),
                has_alt_attr: *has_alt_attr != 0,
            });
    }

    let mut a11y_by_url: std::collections::HashMap<String, Vec<A11yIssue>> =
        std::collections::HashMap::new();
    for (page_url, rule, impact, target, html) in &a11y_rows {
        a11y_by_url
            .entry(page_url.clone())
            .or_default()
            .push(A11yIssue {
                rule: rule.clone(),
                impact: impact.clone(),
                target: target.clone(),
                html: html.clone(),
            });
    }

    let mut sd_issues_by_url: std::collections::HashMap<String, Vec<SdIssue>> =
        std::collections::HashMap::new();
    for (page_url, severity, type_name, code, message) in &sd_issue_rows {
        let sd_severity = match severity.as_str() {
            "error" => SdSeverity::Error,
            _ => SdSeverity::Warning,
        };
        sd_issues_by_url
            .entry(page_url.clone())
            .or_default()
            .push(SdIssue {
                severity: sd_severity,
                type_name: type_name.clone(),
                code: code.clone(),
                message: message.clone(),
            });
    }

    let sitemap_rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT page_url, sitemap_url, lastmod FROM sitemap_urls WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let mut sitemap_by_url: std::collections::HashMap<String, (String, Option<String>)> =
        std::collections::HashMap::new();
    for (page_url, sitemap_url, lastmod) in &sitemap_rows {
        sitemap_by_url
            .entry(page_url.clone())
            .or_insert_with(|| (sitemap_url.clone(), lastmod.clone()));
    }

    let ecom_rows = sqlx::query_as::<
        _,
        (
            String,
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
        ),
    >(
        r#"
        SELECT page_url, page_type, price, currency, availability,
               sku, gtin, brand, has_image, has_description, has_review_or_rating
        FROM ecommerce WHERE crawl_id = ?
        "#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let mut ecom_by_url: std::collections::HashMap<String, crate::crawl::event::EcommerceAudit> =
        std::collections::HashMap::new();
    for (
        page_url,
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
    ) in &ecom_rows
    {
        ecom_by_url.insert(
            page_url.clone(),
            crate::crawl::event::EcommerceAudit {
                price: price.map(|p| p.to_string()),
                currency: currency.clone(),
                availability: availability.clone(),
                sku: sku.clone(),
                gtin: gtin.clone(),
                brand: brand.clone(),
                has_image: *has_image != 0,
                has_description: *has_description != 0,
                has_review_or_rating: *has_review_or_rating != 0,
            },
        );
    }

    let inlink_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT dst_url, COUNT(*) FROM links WHERE crawl_id = ? GROUP BY dst_url",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let inlink_counts: std::collections::HashMap<String, u32> = inlink_rows
        .into_iter()
        .map(|(url, count)| (url, count as u32))
        .collect();

    let unique_inlink_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT dst_url, COUNT(DISTINCT src_url) FROM links WHERE crawl_id = ? GROUP BY dst_url",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let unique_inlink_counts: std::collections::HashMap<String, u32> = unique_inlink_rows
        .into_iter()
        .map(|(url, count)| (url, count as u32))
        .collect();

    let csr_inlink_rows = sqlx::query_as::<_, (String, i64)>(
        "SELECT dst_url, SUM(csr_only) FROM links WHERE crawl_id = ? GROUP BY dst_url",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let csr_inlink_counts: std::collections::HashMap<String, u32> = csr_inlink_rows
        .into_iter()
        .map(|(url, count)| (url, count as u32))
        .collect();

    let link_score_rows = sqlx::query_as::<_, (String, Option<f32>)>(
        "SELECT url, link_score FROM pages WHERE crawl_id = ? ORDER BY id",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let link_scores: std::collections::HashMap<String, f32> = link_score_rows
        .into_iter()
        .filter_map(|(url, score)| score.map(|s| (url, s)))
        .collect();

    let mut backlinks = load_backlinks_for_crawl(pool, crawl_id).await?;

    let mut headers_by_url: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    for (url, headers_json) in &header_rows {
        let headers: Vec<(String, String)> = headers_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        headers_by_url.insert(url.clone(), headers);
    }

    let robots_by_url: std::collections::HashMap<String, String> = robots_rows
        .into_iter()
        .filter_map(|(url, robots)| robots.map(|r| (url, r)))
        .collect();

    let resource_meta_by_url: std::collections::HashMap<
        String,
        (bool, Option<String>, bool, bool),
    > = resource_rows
        .into_iter()
        .map(
            |(url, is_resource, initiator, is_page, has_mixed_content)| {
                (
                    url,
                    (
                        is_resource != 0,
                        initiator,
                        is_page != 0,
                        has_mixed_content != 0,
                    ),
                )
            },
        )
        .collect();

    let mut outlinks_by_url: std::collections::HashMap<String, Vec<Outlink>> =
        std::collections::HashMap::new();
    for (src_url, dst_url, anchor, rel, csr_only) in outlink_rows {
        outlinks_by_url.entry(src_url).or_default().push(Outlink {
            dst_url,
            anchor,
            rel,
            csr_only: csr_only != 0,
        });
    }

    let redirect_by_url: std::collections::HashMap<String, (String, Option<u16>)> = redirect_rows
        .into_iter()
        .filter_map(|(url, redirect, status)| {
            redirect.map(|r| (url, (r, status.map(|s| s as u16))))
        })
        .collect();

    struct SecondaryData {
        title_2: Option<String>,
        meta_description_2: Option<String>,
        h1_2: Option<String>,
        h2_2: Option<String>,
        title_pixel_width: Option<u32>,
        meta_description_pixel_width: Option<u32>,
        ssr_word_count: Option<u32>,
        ssr_h1: Option<String>,
        ssr_content_missing: Option<bool>,
        blocked_by_robots: Option<bool>,
    }
    let secondary_by_url: std::collections::HashMap<String, SecondaryData> = secondary_rows
        .into_iter()
        .map(
            |(
                url,
                title_2,
                meta_description_2,
                h1_2,
                h2_2,
                title_pw,
                meta_pw,
                ssr_word_count,
                ssr_h1,
                ssr_content_missing,
                blocked_by_robots,
            )| {
                (
                    url,
                    SecondaryData {
                        title_2,
                        meta_description_2,
                        h1_2,
                        h2_2,
                        title_pixel_width: title_pw.map(|w| w as u32),
                        meta_description_pixel_width: meta_pw.map(|w| w as u32),
                        ssr_word_count: ssr_word_count.map(|w| w as u32),
                        ssr_h1,
                        ssr_content_missing: ssr_content_missing.map(|b| b != 0),
                        blocked_by_robots: blocked_by_robots.map(|b| b != 0),
                    },
                )
            },
        )
        .collect();

    Ok(base_rows
        .into_iter()
        .zip(sd_meta_rows)
        .zip(near_dup_rows)
        .map(
            |(
                (
                    (
                        url,
                        status,
                        title,
                        size_bytes,
                        content_type,
                        meta_description,
                        h1,
                        h2,
                        canonical,
                        word_count,
                        depth,
                        indexability,
                        response_time_ms,
                        og_type,
                        a11y_errors,
                        a11y_warnings,
                    ),
                    (
                        _,
                        sd_errors,
                        sd_warnings,
                        hreflang_tags_json,
                        sd_types_json,
                        hreflang_issues_json,
                        hreflang_sources_json,
                    ),
                ),
                (
                    _,
                    content_hash,
                    simhash,
                    closest_similarity,
                    near_duplicate_count,
                    near_duplicate_urls_json,
                ),
            )| {
                let is_internal = crate::crawl::engine::is_same_domain(root_url, &url);
                let (is_resource, resource_initiator, is_page, has_mixed_content) =
                    resource_meta_by_url
                        .get(&url)
                        .cloned()
                        .unwrap_or((false, None, true, false));
                let images = images_by_url.remove(&url).unwrap_or_default();
                let hreflang_sources: Vec<crate::crawl::event::HreflangSource> =
                    hreflang_sources_json
                        .as_deref()
                        .and_then(|j| serde_json::from_str(j).ok())
                        .unwrap_or_default();
                let hreflang_tags: Vec<(String, String)> = hreflang_tags_json
                    .as_deref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();
                let hreflang_issues: Vec<crate::crawl::event::HreflangIssue> = hreflang_issues_json
                    .as_deref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();
                let sd_types: Vec<String> = sd_types_json
                    .as_deref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default();
                let sd_entries = sd_by_url.get(&url);
                let sd_jsonld_count = sd_entries
                    .map(|v| v.iter().filter(|(f, _)| f == "json-ld").count() as u32)
                    .unwrap_or(0);
                let sd_microdata_count = sd_entries
                    .map(|v| v.iter().filter(|(f, _)| f == "microdata").count() as u32)
                    .unwrap_or(0);
                let (ttfb_ms, lcp_ms, cls, fcp_ms) = perf_by_url
                    .get(&url)
                    .copied()
                    .unwrap_or((None, None, None, None));

                let page_in_sitemap = if sitemap_by_url.contains_key(&url) {
                    Some(true)
                } else if !sitemap_by_url.is_empty() {
                    Some(false)
                } else {
                    None
                };
                let page_sitemap = sitemap_by_url.get(&url).cloned();
                let page_sitemap_url = page_sitemap.as_ref().map(|(url, _)| url.clone());
                let page_sitemap_lastmod = page_sitemap
                    .as_ref()
                    .and_then(|(_, lastmod)| lastmod.clone());
                let page_ecommerce = ecom_by_url.remove(&url);
                let page_inlinks = inlink_counts.get(&url).copied().unwrap_or(0);
                let page_unique_inlinks = unique_inlink_counts.get(&url).copied().unwrap_or(0);
                let page_csr_inlinks = csr_inlink_counts.get(&url).copied().unwrap_or(0);
                let page_sd_items = sd_items_by_url.remove(&url).unwrap_or_default();
                let page_a11y_issues = a11y_by_url.remove(&url).unwrap_or_default();
                let page_sd_issues = sd_issues_by_url.remove(&url).unwrap_or_default();
                let page_headers = headers_by_url.remove(&url).unwrap_or_default();
                let page_robots = robots_by_url.get(&url).cloned();
                let page_outlinks = outlinks_by_url.remove(&url).unwrap_or_default();
                let page_redirect = redirect_by_url.get(&url).cloned();
                let page_secondary = secondary_by_url.get(&url);
                let page_link_score = link_scores.get(&url).copied();
                let page_backlinks = backlinks.remove(&url).unwrap_or_default();

                PageRecord {
                    url,
                    status: status.map(|s| s as u16),
                    title,
                    size_bytes: size_bytes as u64,
                    content_type,
                    meta_description,
                    h1,
                    h2,
                    canonical,
                    robots: page_robots,
                    outlinks: page_outlinks,
                    word_count: word_count.map(|w| w as u32),
                    depth: depth.map(|d| d as u32),
                    is_internal,
                    is_page,
                    is_resource,
                    resource_initiator,
                    has_mixed_content,
                    indexability,
                    response_time: std::time::Duration::from_millis(
                        response_time_ms.unwrap_or(0) as u64
                    ),
                    sd_errors: sd_errors as u32,
                    sd_warnings: sd_warnings as u32,
                    sd_types,
                    hreflang_tags,
                    ttfb_ms: ttfb_ms.map(|ms| ms as u64),
                    lcp_ms: lcp_ms.map(|ms| ms as u64),
                    cls,
                    fcp_ms: fcp_ms.map(|ms| ms as u64),
                    sd_jsonld_count,
                    sd_microdata_count,
                    images,
                    content_hash,
                    simhash: simhash.map(|h| h as u64),
                    closest_similarity: closest_similarity.map(|s| s as u8),
                    near_duplicate_count: near_duplicate_count.map(|c| c as u32),
                    near_duplicate_urls: near_duplicate_urls_json
                        .as_deref()
                        .and_then(|j| serde_json::from_str(j).ok())
                        .unwrap_or_default(),
                    hreflang_sources,
                    in_sitemap: page_in_sitemap,
                    sitemap_url: page_sitemap_url,
                    sitemap_lastmod: page_sitemap_lastmod,
                    og_type,
                    ecommerce: page_ecommerce,
                    a11y_errors: a11y_errors as u32,
                    a11y_warnings: a11y_warnings as u32,
                    a11y_issues: page_a11y_issues,
                    inlinks_count: page_inlinks,
                    unique_inlinks_count: page_unique_inlinks,
                    csr_inlinks_count: page_csr_inlinks,
                    sd_items: page_sd_items,
                    sd_issues: page_sd_issues,
                    headers: page_headers,
                    redirect_url: page_redirect.as_ref().map(|(url, _)| url.clone()),
                    redirect_status: page_redirect.and_then(|(_, status)| status),
                    title_2: page_secondary.and_then(|s| s.title_2.clone()),
                    meta_description_2: page_secondary.and_then(|s| s.meta_description_2.clone()),
                    h1_2: page_secondary.and_then(|s| s.h1_2.clone()),
                    h2_2: page_secondary.and_then(|s| s.h2_2.clone()),
                    title_pixel_width: page_secondary.and_then(|s| s.title_pixel_width),
                    meta_description_pixel_width: page_secondary
                        .and_then(|s| s.meta_description_pixel_width),
                    ssr_word_count: page_secondary.and_then(|s| s.ssr_word_count),
                    ssr_h1: page_secondary.and_then(|s| s.ssr_h1.clone()),
                    ssr_content_missing: page_secondary.and_then(|s| s.ssr_content_missing),
                    blocked_by_robots: page_secondary.and_then(|s| s.blocked_by_robots),
                    link_score: page_link_score,
                    backlinks: page_backlinks,
                    hreflang_issues,
                    ..Default::default()
                }
            },
        )
        .collect())
}

pub async fn load_simhashes_for_crawl(
    pool: &SqlitePool,
    crawl_id: i64,
) -> Result<Vec<(String, u64, Option<String>)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, Option<i64>, Option<String>)>(
        r#"
        SELECT url, simhash, hash
        FROM pages
        WHERE crawl_id = ? AND simhash IS NOT NULL
        ORDER BY id
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
            "UPDATE pages SET closest_similarity = ?, near_duplicate_count = ?, near_duplicate_urls_json = ? WHERE crawl_id = ? AND url = ?",
        )
        .bind(*similarity as i64)
        .bind(*count as i64)
        .bind(dup_urls_json)
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
    for entry in sitemap_urls {
        sqlx::query(
            "INSERT INTO sitemap_urls (crawl_id, sitemap_url, page_url, lastmod) VALUES (?, ?, ?, ?)",
        )
        .bind(crawl_id)
        .bind(&entry.sitemap_url)
        .bind(&entry.page_url)
        .bind(entry.lastmod.as_deref())
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
        SELECT su.page_url, su.sitemap_url, su.lastmod
        FROM sitemap_urls su
        LEFT JOIN pages p ON p.crawl_id = su.crawl_id AND p.url = su.page_url
        WHERE su.crawl_id = ? AND p.id IS NULL
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
