//! Reproducible CSV export of a live crawl, for the parity comparison.
//!
//! The app is a desktop UI with no command line, so the exports in
//! `scripts/local/comparison/` were produced by hand: crawl, then export each
//! tab. That is slow to repeat and easy to do differently the second time,
//! which matters when the whole document is a diff between two runs.
//!
//! This drives the same code the UI does (the crawl engine, then
//! `load_pages_for_crawl`, then `ResultsDelegate::export_csv` per tab) and
//! writes one CSV per tab to a directory.
//!
//! It crawls a live third-party site, so it is `#[ignore]`d and never runs as
//! part of the suite:
//!
//! ```text
//! SR_PARITY_URL=https://www.bylynga.com/ \
//! SR_PARITY_OUT=scripts/local/comparison/sr3 \
//!   cargo test --bin shoutingrobin -- --ignored --nocapture parity_export
//! ```

use std::path::Path;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::crawl::CrawlConfig;
use crate::crawl::event::{CrawlEvent, PageRecord};
use crate::crawl::render_mode::RenderMode;
use crate::views::results_grid::ResultsDelegate;

/// Crawls `root_url` and returns the records as the app has them after the
/// crawl finishes: reloaded from the database, so the post-crawl passes are
/// included.
fn crawl_and_load(root_url: &str, timeout: Duration) -> Vec<PageRecord> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .expect("build runtime");

    let pool = rt.block_on(async {
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").expect("open pool");
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create migrations table");
        crate::storage::run_migrations(&pool)
            .await
            .expect("run migrations");
        pool
    });

    let (tx, rx) = crate::crawl::engine::channel();
    let (cancel, fut) = {
        let mut engine = crate::crawl::engine::CrawlEngine::new();
        engine.start(
            root_url.to_string(),
            tx,
            pool.clone(),
            RenderMode::Http,
            CrawlConfig {
                max_pages: 0,
                max_concurrent: 10,
                delay_ms: 0,
                timeout_seconds: 30,
                respect_robots_txt: true,
                follow_sitemaps: true,
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
                check_resources: true,
            },
        )
    };

    rt.spawn(async move {
        fut.await;
    });

    let start = std::time::Instant::now();
    let mut crawled = 0usize;
    loop {
        let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
            eprintln!("parity crawl timed out after {timeout:?}");
            cancel.store(true, Ordering::Relaxed);
            break;
        };
        match rx.recv_timeout(remaining) {
            Ok(CrawlEvent::Finished { .. }) => break,
            Ok(CrawlEvent::Error { url, message }) => eprintln!("crawl error {url}: {message}"),
            Ok(CrawlEvent::Page(_)) => {
                crawled += 1;
                if crawled.is_multiple_of(25) {
                    eprintln!("  {crawled} rows so far ({:?})", start.elapsed());
                }
            }
            Ok(_) => {}
            Err(flume::RecvTimeoutError::Timeout) => {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            Err(flume::RecvTimeoutError::Disconnected) => break,
        }
    }

    let pages = rt.block_on(async {
        let crawl_id: i64 = sqlx::query_scalar("SELECT id FROM crawls ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("crawl row should exist");
        crate::storage::load_pages_for_crawl(&pool, crawl_id, root_url)
            .await
            .expect("load pages")
    });

    cancel.store(true, Ordering::Relaxed);
    pages
}

/// Writes one CSV per tab, named after the tab, exactly as the UI's export
/// button produces them.
fn export_all_tabs(pages: Vec<PageRecord>, root_url: &str, out_dir: &Path) {
    std::fs::create_dir_all(out_dir).expect("create output directory");

    let mut delegate = ResultsDelegate::new();
    delegate.set_root_url(root_url);
    delegate.replace_records(pages);

    for (tab, csv) in crate::views::results_grid::export_every_tab(delegate.snapshot()) {
        let csv = match csv {
            Ok(csv) => csv,
            Err(e) => {
                eprintln!("export failed for {tab:?}: {e}");
                continue;
            }
        };
        let name = format!("{tab:?}").to_lowercase();
        let path = out_dir.join(format!("sr-{name}.csv"));
        std::fs::write(&path, &csv).expect("write csv");
        eprintln!(
            "{:>16}  {:>6} rows  {}",
            format!("{tab:?}"),
            csv.lines().count().saturating_sub(1),
            path.display()
        );
    }
}

#[test]
#[ignore = "crawls a live site; run explicitly with SR_PARITY_URL set"]
fn parity_export() {
    let root_url = std::env::var("SR_PARITY_URL")
        .expect("set SR_PARITY_URL to the site to crawl, e.g. https://www.example.com/");
    let out_dir = std::env::var("SR_PARITY_OUT").unwrap_or_else(|_| "parity-export".to_string());
    let timeout = std::env::var("SR_PARITY_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(900));

    eprintln!("crawling {root_url} (timeout {timeout:?})");
    let started = std::time::Instant::now();
    let pages = crawl_and_load(&root_url, timeout);
    eprintln!("crawled {} rows in {:?}", pages.len(), started.elapsed());
    assert!(!pages.is_empty(), "the crawl produced no rows");

    export_all_tabs(pages, &root_url, Path::new(&out_dir));
}
