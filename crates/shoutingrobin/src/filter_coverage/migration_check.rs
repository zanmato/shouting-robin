//! Throwaway check for migration 0036, which stores texts once per crawl and
//! points to them by id. Delete this file once the migration has shipped.
//!
//! The fixture has to come from the code before the migration, since the code
//! after it writes the new layout. So the check runs in two steps:
//!
//! ```sh
//! # with the code before 0036: crawl the test site, save the databases and a dump
//! cargo test -p shoutingrobin --bin shoutingrobin record_before_migration_0036 -- --ignored
//! # with the code after: migrate copies, read them back, compare the dumps
//! cargo test -p shoutingrobin --bin shoutingrobin migration_0036_keeps_the_test_crawl -- --ignored
//! ```

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::crawl::render_mode::RenderMode;
use crate::views::ResultTab;
use crate::views::results_grid::tab_filter_counts_for_test;

use super::{crawl_into, spawn_site, test_runtime};

const MODES: [(&str, RenderMode); 2] = [("http", RenderMode::Http), ("chrome", RenderMode::Chrome)];

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/migration-0036")
}

async fn open(path: &std::path::Path, create: bool) -> sqlx::SqlitePool {
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(create)
        .foreign_keys(true);
    let pool = sqlx::SqlitePool::connect_with(options)
        .await
        .expect("open the database");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("migrations table");
    crate::storage::run_migrations(&pool)
        .await
        .expect("migrations");
    pool
}

/// Everything the app reads back from a crawl, as text: every tab's counts,
/// every record in load order, each page's backlinks and structured data, the
/// history's counts and the readers the post-crawl passes use.
async fn dump(pool: &sqlx::SqlitePool) -> serde_json::Value {
    let (crawl_id, root_url): (i64, String) =
        sqlx::query_as("SELECT id, root_url FROM crawls ORDER BY id DESC LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("the crawl");
    let pages = crate::storage::load_pages_for_crawl(pool, crawl_id, &root_url)
        .await
        .expect("load the crawl");
    let origin = url::Url::parse(&root_url)
        .ok()
        .map(|url| url.origin().ascii_serialization());
    let counts: Vec<(ResultTab, serde_json::Value)> = ResultTab::ALL
        .iter()
        .map(|&tab| {
            let counts = tab_filter_counts_for_test(tab, &pages, origin.as_deref());
            (
                tab,
                serde_json::to_value(counts).expect("encode the counts"),
            )
        })
        .collect();
    let mut on_demand = Vec::new();
    for page in &pages {
        let loaded = crate::storage::load_page_on_demand(pool, crawl_id, &page.url)
            .await
            .expect("load on demand");
        on_demand.push(format!(
            "{} {:?} {:?}",
            page.url, loaded.backlinks, loaded.sd_items
        ));
    }
    let history: Vec<String> = crate::storage::list_crawls(pool)
        .await
        .expect("list crawls")
        .iter()
        .map(|crawl| {
            format!(
                "{} {} {} {}",
                crawl.id, crawl.page_count, crawl.document_count, crawl.unfetched_count
            )
        })
        .collect();
    let mut page_urls: Vec<String> = crate::storage::load_page_urls(pool, crawl_id)
        .await
        .expect("page urls")
        .into_iter()
        .collect();
    page_urls.sort();
    let orphans: Vec<String> = crate::storage::load_sitemap_orphans(pool, crawl_id)
        .await
        .expect("orphans")
        .iter()
        .map(|orphan| format!("{orphan:?}"))
        .collect();
    let simhashes = format!(
        "{:?}",
        crate::storage::load_simhashes_for_crawl(pool, crawl_id)
            .await
            .expect("simhashes")
    );
    serde_json::json!({
        "counts": counts,
        "records": pages.iter().map(|page| format!("{page:?}")).collect::<Vec<_>>(),
        "on_demand": on_demand,
        "history": history,
        "page_urls": page_urls,
        "orphans": orphans,
        "simhashes": simhashes,
    })
}

#[test]
#[ignore = "throwaway: run with the code before migration 0036"]
fn record_before_migration_0036() {
    let dir = fixture_dir();
    std::fs::create_dir_all(&dir).expect("fixture directory");
    for (name, render_mode) in MODES {
        let path = dir.join(format!("{name}.db"));
        if path.exists() {
            std::fs::remove_file(&path).expect("remove the old fixture");
        }
        let (handle, port, stop) = spawn_site();
        let root_url = format!("http://127.0.0.1:{port}/");
        let rt = test_runtime();
        let pool = rt.block_on(open(&path, true));
        let timeout = match render_mode {
            RenderMode::Http => Duration::from_secs(60),
            RenderMode::Chrome => Duration::from_secs(180),
        };
        crawl_into(&rt, &pool, &root_url, render_mode, timeout);
        stop.store(true, Ordering::Relaxed);
        handle.join().expect("site thread");
        let dumped = rt.block_on(dump(&pool));
        rt.block_on(pool.close());
        std::fs::write(
            dir.join(format!("{name}.json")),
            serde_json::to_string_pretty(&dumped).expect("encode"),
        )
        .expect("write the dump");
        eprintln!(
            "{name}: {} records",
            dumped["records"].as_array().map_or(0, Vec::len)
        );
    }
}

#[test]
#[ignore = "throwaway: needs the fixture record_before_migration_0036 saves"]
fn migration_0036_keeps_the_test_crawl() {
    let dir = fixture_dir();
    for (name, _) in MODES {
        let fixture = dir.join(format!("{name}.db"));
        let migrated = dir.join(format!("{name}-migrated.db"));
        std::fs::copy(&fixture, &migrated).expect("copy the fixture");
        let before: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join(format!("{name}.json"))).expect("read the dump"),
        )
        .expect("decode the dump");
        let rt = test_runtime();
        let after = rt.block_on(async {
            let pool = open(&migrated, false).await;
            let applied: Option<String> =
                sqlx::query_scalar("SELECT name FROM schema_migrations WHERE name LIKE '0036%'")
                    .fetch_optional(&pool)
                    .await
                    .expect("migrations");
            assert!(applied.is_some(), "migration 0036 ran");
            let dumped = dump(&pool).await;
            pool.close().await;
            dumped
        });
        for key in [
            "counts",
            "records",
            "on_demand",
            "history",
            "page_urls",
            "orphans",
            "simhashes",
        ] {
            assert_eq!(after[key], before[key], "{name}: {key} changed");
        }
        assert!(
            before["records"]
                .as_array()
                .is_some_and(|records| records.len() > 30),
            "{name}: the fixture has the test site"
        );
        std::fs::remove_file(&migrated).expect("remove the migrated copy");
    }
}

/// A large crawl's counts and records, hashed, plus the backlinks of every
/// hundredth page. The full dump of a 400k row crawl would be gigabytes.
async fn dump_large(pool: &sqlx::SqlitePool, crawl_id: i64) -> serde_json::Value {
    use sha2::{Digest as _, Sha256};
    let root_url: String = sqlx::query_scalar("SELECT root_url FROM crawls WHERE id = ?")
        .bind(crawl_id)
        .fetch_one(pool)
        .await
        .expect("the crawl");
    let pages = crate::storage::load_pages_for_crawl(pool, crawl_id, &root_url)
        .await
        .expect("load the crawl");
    let origin = url::Url::parse(&root_url)
        .ok()
        .map(|url| url.origin().ascii_serialization());
    let counts: Vec<(ResultTab, serde_json::Value)> = ResultTab::ALL
        .iter()
        .map(|&tab| {
            let counts = tab_filter_counts_for_test(tab, &pages, origin.as_deref());
            (
                tab,
                serde_json::to_value(counts).expect("encode the counts"),
            )
        })
        .collect();
    let records: Vec<String> = pages
        .iter()
        .map(|page| hex::encode(Sha256::digest(format!("{page:?}").as_bytes())))
        .collect();
    let mut on_demand = Vec::new();
    for page in pages.iter().step_by(100) {
        let loaded = crate::storage::load_page_on_demand(pool, crawl_id, &page.url)
            .await
            .expect("load on demand");
        on_demand.push(hex::encode(Sha256::digest(
            format!("{} {:?} {:?}", page.url, loaded.backlinks, loaded.sd_items).as_bytes(),
        )));
    }
    serde_json::json!({ "counts": counts, "records": records, "on_demand": on_demand })
}

fn large_crawl() -> Option<(PathBuf, i64)> {
    let path = std::env::var("MIGRATION_DB").ok()?;
    let crawl_id = std::env::var("MIGRATION_CRAWL")
        .ok()
        .and_then(|value| value.parse().ok())
        .expect("MIGRATION_CRAWL names a crawl");
    Some((PathBuf::from(path), crawl_id))
}

/// `MIGRATION_DB` is a copy of a real database, `MIGRATION_CRAWL` a crawl in it.
#[test]
#[ignore = "throwaway: run with the code before migration 0036"]
fn record_large_crawl_before_migration_0036() {
    let Some((path, crawl_id)) = large_crawl() else {
        eprintln!("MIGRATION_DB is not set, skipping");
        return;
    };
    let rt = test_runtime();
    let dumped = rt.block_on(async {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(&path)
            .read_only(true);
        let pool = sqlx::SqlitePool::connect_with(options)
            .await
            .expect("open the database");
        dump_large(&pool, crawl_id).await
    });
    std::fs::create_dir_all(fixture_dir()).expect("fixture directory");
    std::fs::write(
        fixture_dir().join(format!("large-{crawl_id}.json")),
        serde_json::to_string(&dumped).expect("encode"),
    )
    .expect("write the dump");
}

/// Migrates `MIGRATION_DB` in place, so point it at a copy, and compares the
/// crawl with what `record_large_crawl_before_migration_0036` saw.
#[test]
#[ignore = "throwaway: migrates MIGRATION_DB in place"]
fn migration_0036_keeps_a_large_crawl() {
    let Some((path, crawl_id)) = large_crawl() else {
        eprintln!("MIGRATION_DB is not set, skipping");
        return;
    };
    let before: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fixture_dir().join(format!("large-{crawl_id}.json")))
            .expect("read the dump"),
    )
    .expect("decode the dump");
    let rt = test_runtime();
    let after = rt.block_on(async {
        let started = std::time::Instant::now();
        let pool = open(&path, false).await;
        eprintln!("migrated in {:.1} s", started.elapsed().as_secs_f64());
        dump_large(&pool, crawl_id).await
    });
    for key in ["counts", "records", "on_demand"] {
        assert_eq!(after[key], before[key], "{key} changed");
    }
}
