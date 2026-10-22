//! The crawl config is persisted so a crawl can be replayed from the sidebar's
//! Recrawl action with the settings it actually ran with.

use shoutingrobin::crawl::{CrawlConfig, LayoutCapture};

fn sample_config() -> CrawlConfig {
    CrawlConfig {
        max_pages: 500,
        max_concurrent: 4,
        delay_ms: 250,
        timeout_seconds: 45,
        respect_robots_txt: false,
        follow_sitemaps: true,
        block_images: true,
        near_duplicate_threshold: 80,
        content_selector: "main".to_string(),
        user_agent: Some("shouting-robin-test".to_string()),
        extra_headers: vec![("Authorization".to_string(), "Bearer token".to_string())],
        include_patterns: vec!["/blog/.*".to_string()],
        exclude_patterns: vec!["/tag/.*".to_string()],
        crawl_subdomains: true,
        list_mode: true,
        check_resources: true,
        layout_capture: LayoutCapture::Scrolled,
        seed_urls: vec![
            "https://example.com/one".to_string(),
            "https://example.com/two".to_string(),
        ],
    }
}

async fn memory_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
    )
    .execute(&pool)
    .await
    .unwrap();
    shoutingrobin::storage::run_migrations(&pool).await.unwrap();
    pool
}

#[test]
fn crawl_config_round_trips_through_storage() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let pool = memory_pool().await;
        let config = sample_config();

        let crawl_id =
            shoutingrobin::storage::create_crawl(&pool, "https://example.com/", "chrome", &config)
                .await
                .expect("create_crawl should succeed");

        let loaded = shoutingrobin::storage::load_crawl_config(&pool, crawl_id)
            .await
            .expect("load_crawl_config should succeed")
            .expect("config should have been recorded");

        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&config).unwrap(),
            "the replayed config should match the one the crawl ran with"
        );
    });
}

/// Crawls recorded before the config was written back leave `config_json` NULL.
/// Recrawl falls back to the current settings in that case, so this must be a
/// clean `None` rather than an error.
#[test]
fn missing_config_reads_back_as_none() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let pool = memory_pool().await;

        let result =
            sqlx::query("INSERT INTO crawls (root_url, started_at, render_mode) VALUES (?, ?, ?)")
                .bind("https://example.com/")
                .bind(0_i64)
                .bind("http")
                .execute(&pool)
                .await
                .expect("legacy insert should succeed");

        let loaded = shoutingrobin::storage::load_crawl_config(&pool, result.last_insert_rowid())
            .await
            .expect("load_crawl_config should succeed");

        assert!(loaded.is_none(), "a legacy crawl has no config to replay");
    });
}

#[test]
fn unknown_crawl_id_reads_back_as_none() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let pool = memory_pool().await;
        let loaded = shoutingrobin::storage::load_crawl_config(&pool, 4242)
            .await
            .expect("a missing row is not an error");
        assert!(loaded.is_none());
    });
}

/// A config stored before layout capture existed has no `layout_capture`
/// key. Replaying it must not turn capture on.
#[test]
fn config_stored_before_layout_capture_reads_back_as_off() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let pool = memory_pool().await;
        let mut stored = serde_json::to_value(sample_config()).unwrap();
        stored
            .as_object_mut()
            .expect("the config serializes as an object")
            .remove("layout_capture");

        let result = sqlx::query(
            "INSERT INTO crawls (root_url, started_at, render_mode, config_json) VALUES (?, ?, ?, ?)",
        )
        .bind("https://example.com/")
        .bind(0_i64)
        .bind("chrome")
        .bind(stored.to_string())
        .execute(&pool)
        .await
        .expect("insert should succeed");

        let loaded = shoutingrobin::storage::load_crawl_config(&pool, result.last_insert_rowid())
            .await
            .expect("load_crawl_config should succeed")
            .expect("the config should parse without the new key");
        assert_eq!(loaded.layout_capture, LayoutCapture::Off);

        let crawls = shoutingrobin::storage::list_crawls(&pool).await.unwrap();
        assert_eq!(crawls[0].layout_capture, LayoutCapture::Off);
    });
}

/// The sidebar shows whether a crawl captured layout, read from the config it
/// ran with.
#[test]
fn listed_crawls_carry_their_layout_capture() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async {
        let pool = memory_pool().await;
        for capture in [
            LayoutCapture::Off,
            LayoutCapture::Loaded,
            LayoutCapture::Scrolled,
        ] {
            let config = CrawlConfig {
                layout_capture: capture,
                ..sample_config()
            };
            shoutingrobin::storage::create_crawl(&pool, "https://example.com/", "chrome", &config)
                .await
                .expect("create_crawl should succeed");
        }

        let mut listed: Vec<LayoutCapture> = shoutingrobin::storage::list_crawls(&pool)
            .await
            .unwrap()
            .into_iter()
            .map(|crawl| crawl.layout_capture)
            .collect();
        listed.sort_by_key(|capture| *capture as u8);
        assert_eq!(
            listed,
            [
                LayoutCapture::Off,
                LayoutCapture::Loaded,
                LayoutCapture::Scrolled
            ]
        );
    });
}
