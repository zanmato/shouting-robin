use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use flume::{Receiver, Sender};
use gpui::{App, Global};
use spider::features::chrome_common::{
    RequestInterceptConfiguration, WaitForIdleNetwork, WaitForSelector, WebAutomation,
};
use spider::website::Website;
use sqlx::SqlitePool;

use crate::crawl::CrawlConfig;
use crate::crawl::event::{A11yIssue, CrawlEvent, PageRecord};
use crate::crawl::render_mode::RenderMode;
use crate::storage;

pub struct CrawlEngine {
    cancel: Option<Arc<AtomicBool>>,
}

impl Global for CrawlEngine {}

impl Default for CrawlEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl CrawlEngine {
    pub fn new() -> Self {
        Self { cancel: None }
    }

    #[allow(dead_code)]
    pub fn global_mut(cx: &mut App) -> &mut Self {
        cx.global_mut::<Self>()
    }

    #[allow(dead_code)]
    pub fn is_running(&self) -> bool {
        self.cancel
            .as_ref()
            .map(|c| !c.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    pub fn stop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Build a (cancel_flag, future) pair to be spawned on the tokio runtime via
    /// `gpui_tokio::Tokio::spawn`. The future drives the spider crawl and pushes
    /// events into `tx`.
    pub fn start(
        &mut self,
        root_url: String,
        tx: Sender<CrawlEvent>,
        pool: SqlitePool,
        render_mode: RenderMode,
        config: CrawlConfig,
    ) -> (
        Arc<AtomicBool>,
        impl std::future::Future<Output = ()> + Send + 'static,
    ) {
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());

        let cancel_flag = cancel.clone();
        let mode_str = match render_mode {
            RenderMode::Http => "http",
            RenderMode::Chrome => "chrome",
        };
        let fut = async move {
            let crawl_id = match storage::create_crawl(&pool, &root_url, mode_str).await {
                Ok(id) => id,
                Err(e) => {
                    let _ = tx
                        .send_async(CrawlEvent::Error {
                            url: root_url.clone(),
                            message: format!("Failed to create crawl: {e}"),
                        })
                        .await;
                    return;
                }
            };

            let _ = tx
                .send_async(CrawlEvent::Started {
                    crawl_id,
                    root_url: root_url.clone(),
                })
                .await;

            let sitemap_entries = if config.follow_sitemaps {
                let sitemap_urls = crate::crawl::sitemap::discover_sitemaps(&root_url).await;
                let entries = crate::crawl::sitemap::fetch_sitemap_urls(&sitemap_urls, 3).await;
                if let Err(e) = storage::insert_sitemap_urls(&pool, crawl_id, &entries).await {
                    tracing::warn!(error=%e, "failed to persist sitemap URLs");
                }
                entries
            } else {
                Vec::new()
            };

            let mut sitemap_lookup: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for entry in &sitemap_entries {
                sitemap_lookup
                    .entry(entry.page_url.clone())
                    .or_insert_with(|| entry.sitemap_url.clone());
            }

            let root_url = if config.list_mode && !config.seed_urls.is_empty() {
                config.seed_urls.first().cloned().unwrap_or_default()
            } else {
                root_url
            };

            let mut website = Website::new(&root_url);
            website.with_respect_robots_txt(config.respect_robots_txt);

            if config.max_pages > 0 {
                website.with_limit(config.max_pages);
            }
            website.with_concurrency_limit(Some(config.max_concurrent as usize));
            if config.delay_ms > 0 {
                website.with_delay(config.delay_ms);
            }
            website.with_request_timeout(Some(Duration::from_secs(config.timeout_seconds as u64)));

            if let Some(ref ua) = config.user_agent
                && !ua.is_empty()
            {
                website.with_user_agent(Some(ua.as_str()));
            }

            if config.crawl_subdomains {
                website.with_subdomains(true);
            }

            if !config.include_patterns.is_empty() {
                let patterns: Vec<spider::compact_str::CompactString> = config
                    .include_patterns
                    .iter()
                    .map(|s| spider::compact_str::CompactString::from(s.as_str()))
                    .collect();
                website.with_whitelist_url(Some(patterns));
            }
            if !config.exclude_patterns.is_empty() {
                let patterns: Vec<spider::compact_str::CompactString> = config
                    .exclude_patterns
                    .iter()
                    .map(|s| spider::compact_str::CompactString::from(s.as_str()))
                    .collect();
                website.with_blacklist_url(Some(patterns));
            }

            if !config.extra_headers.is_empty() {
                let mut headers = reqwest::header::HeaderMap::new();
                for (key, value) in &config.extra_headers {
                    if let Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                        && let Ok(val) = reqwest::header::HeaderValue::from_str(value)
                    {
                        headers.insert(name, val);
                    }
                }
                if !headers.is_empty() {
                    website.with_headers(Some(headers));
                }
            }

            if config.list_mode {
                website.with_depth(0);
            }

            match render_mode {
                RenderMode::Http => {
                    website.with_disable_chrome(true);
                }
                RenderMode::Chrome => {
                    let mut automation_map =
                        spider::features::chrome_common::AutomationScriptsMap::default();
                    automation_map.insert(
                        "/".to_string(),
                        vec![WebAutomation::Evaluate(METRICS_AUTOMATION_JS.to_string())],
                    );
                    let wait_cap = (config.timeout_seconds as u64)
                        .saturating_sub(5)
                        .clamp(3, 15);
                    website
                        .with_chrome_intercept(RequestInterceptConfiguration::new(
                            config.block_images,
                        ))
                        .with_stealth(true)
                        .with_wait_for_idle_dom(Some(WaitForSelector::new(
                            Some(Duration::from_secs(wait_cap)),
                            "body".into(),
                        )))
                        // Wait until the network is almost idle so that the
                        // resources Chrome fetches to render the page are
                        // present in the Resource Timing buffer when we harvest
                        // them. Idle DOM alone returns before assets settle.
                        .with_wait_for_almost_idle_network0(Some(WaitForIdleNetwork::new(Some(
                            Duration::from_secs(wait_cap),
                        ))))
                        .with_automation_scripts(Some(automation_map))
                        .with_evaluate_on_new_document(Some(Box::new(
                            PERF_OBSERVER_JS.to_string(),
                        )));
                }
            }

            let blocked_urls: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
                std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            {
                let blocked_urls = blocked_urls.clone();
                website.with_on_link_blocked_callback(Some(move |url: String| {
                    if let Ok(mut guard) = blocked_urls.lock() {
                        guard.push(url);
                    }
                }));
            }

            if website.build().is_err() {
                let _ = tx
                    .send_async(CrawlEvent::Error {
                        url: root_url.clone(),
                        message: "Failed to parse root URL".into(),
                    })
                    .await;
                return;
            }

            let mut rx = website.subscribe(1024);
            // Keeps spider from advancing past each broadcast (and closing
            // the chrome tab) until our pump calls `inc()`. Without this,
            // `Page::get_chrome_page()` returns a handle to an already-
            // closed tab on sub-pages and CDP `evaluate()` yields "channel
            // closed". Mirrors spider's `chrome_screenshot` example.
            let subscribe_guard = website.subscribe_guard();

            let tx_pages = tx.clone();
            let cancel_pages = cancel_flag.clone();
            let pool_pages = pool.clone();
            let root_for_pump = root_url.clone();
            let chrome_mode = matches!(render_mode, RenderMode::Chrome);
            let content_selector_for_pump = config.content_selector.clone();
            let sitemap_for_pump = sitemap_lookup;
            // In Chrome mode the analyzed HTML is the post-JS rendered DOM. We
            // fetch the raw server HTML separately so we can diff SSR vs CSR.
            let ssr_client = if chrome_mode {
                match build_ssr_client(&config) {
                    Ok(client) => Some(client),
                    Err(e) => {
                        tracing::warn!(error=%e, "failed to build SSR client; skipping SSR diff");
                        None
                    }
                }
            } else {
                None
            };
            // axe.min.js is a static asset identical for every page, so fetch it
            // once per crawl and reuse it for all a11y scans.
            let axe_js = if chrome_mode {
                fetch_axe_js().await
            } else {
                None
            };
            let pump = tokio::spawn(async move {
                let mut subscribe_guard = subscribe_guard;
                let mut total: u64 = 0;
                let mut inlink_counts: std::collections::HashMap<String, (u32, u32)> =
                    std::collections::HashMap::new();
                // Resource URLs already emitted as their own rows. The same
                // asset (a shared stylesheet, say) shows up on most pages, so
                // we record each one once.
                let mut seen_resources: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                tracing::info!("page pump started, waiting for pages...");
                loop {
                    let mut page = match rx.recv().await {
                        Ok(p) => p,
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!(
                                skipped,
                                total,
                                "page pump lagged, dropping older pages"
                            );
                            continue;
                        }
                        Err(e) => {
                            tracing::info!(error=%e, total, "page pump exiting recv loop");
                            break;
                        }
                    };
                    if cancel_pages.load(Ordering::Relaxed) {
                        break;
                    }
                    total += 1;
                    let html_bytes = page.get_html_bytes_u8();
                    let mut record = PageRecord {
                        url: page.get_url().to_string(),
                        status: Some(page.status_code.as_u16()),
                        size_bytes: html_bytes.len() as u64,
                        response_time: page.get_duration_elapsed(),
                        is_page: true,
                        ..Default::default()
                    };
                    if let Some(ref headers) = page.headers
                        && let Some(ct) = headers.get(reqwest::header::CONTENT_TYPE)
                        && let Ok(ct_str) = ct.to_str()
                    {
                        record.content_type = Some(
                            ct_str
                                .split(';')
                                .next()
                                .unwrap_or(ct_str)
                                .trim()
                                // Chrome's CDP can hand back header values still
                                // wrapped in JSON quotes (e.g. "\"text/html\"").
                                .trim_matches('"')
                                .to_string(),
                        );
                    }
                    // Subresources fetched via full-resource crawling arrive
                    // without response headers, so fall back to the URL's file
                    // extension. Without this every resource is later defaulted
                    // to text/html, leaving the Internal tab's CSS/JS/Images
                    // filters empty.
                    if record.content_type.is_none() {
                        record.content_type = content_type_from_url(&record.url);
                    }
                    if let Some(ref headers) = page.headers {
                        record.headers = headers
                            .iter()
                            .filter_map(|(name, value)| {
                                value
                                    .to_str()
                                    .ok()
                                    .map(|v| (name.to_string(), v.to_string()))
                            })
                            .collect();
                    }
                    let original_url = page.get_url();
                    let final_url = page.get_url_final();
                    let redirected = original_url != final_url;
                    if redirected {
                        record.redirect_url = Some(final_url.to_string());
                        record.redirect_status = Some(301);
                    }
                    // The body we hold belongs to whatever URL spider ultimately
                    // landed on, not necessarily this row's URL. Two cases must
                    // skip SEO analysis:
                    //   * a redirect (e.g. `/` -> `/home`): the body is the
                    //     target's, which is crawled and analyzed as its own row,
                    //     so analyzing it here too double-counts its title/H1 and
                    //     manufactures duplicate-content warnings;
                    //   * an off-domain landing (e.g. `/app/ios` -> App Store):
                    //     it isn't our site to audit.
                    // We still emit the row so the redirect itself is visible.
                    let analyzed_url = final_url;
                    let analyzed_external = !is_same_domain(&root_for_pump, analyzed_url);
                    let skip_analysis = redirected || analyzed_external;
                    if !skip_analysis
                        && !html_bytes.is_empty()
                        && let Ok(html) = std::str::from_utf8(html_bytes)
                    {
                        tracing::debug!(
                            url = %record.url,
                            html_len = html.len(),
                            "received html for analysis"
                        );
                        // In Chrome render mode with request interception, spider
                        // can hand us the response headers of an arbitrary
                        // subresource rather than the main document, so the
                        // Content-Type above is unreliable (e.g. a fully-rendered
                        // page reported as application/json or text/css). When the
                        // body is actually HTML, trust the body over that header.
                        if looks_like_html(html) {
                            record.content_type = Some("text/html".to_string());
                        }
                        crate::crawl::analyzers::analyze_html(
                            &mut record,
                            html,
                            &content_selector_for_pump,
                        );
                    }

                    let mut resource_timings: Vec<ResourceTiming> = Vec::new();
                    if chrome_mode {
                        if skip_analysis {
                            // Redirect source or off-domain landing: don't measure
                            // web vitals, scan a11y, or harvest its subresources.
                            page.close_page().await;
                        } else if page.get_chrome_page().is_some() {
                            collect_performance_metrics(&page, &mut record).await;
                            if let Some(axe_js) = axe_js.as_deref() {
                                collect_a11y_violations(&page, &mut record, axe_js).await;
                            }
                            resource_timings = collect_resource_timings(&page).await;
                            page.close_page().await;
                        } else {
                            tracing::warn!(
                                url = %record.url,
                                "chrome page handle missing; spider used the HTTP path \
                                 (check /tmp/chromiumoxide-runner/SingletonLock or chrome config)"
                            );
                        }
                        page.close_page().await;
                    }

                    if let Some(guard) = subscribe_guard.as_mut() {
                        guard.inc();
                    }

                    // The SSR diff needs the raw server HTML, not chrome. Run it
                    // after releasing the chrome tab and advancing the guard so
                    // it never holds spider back or keeps a tab open. Skipped for
                    // redirects and off-domain landings, which carry no analysis.
                    if !skip_analysis && let Some(client) = ssr_client.as_ref() {
                        let ssr_url = record.url.clone();
                        fetch_and_analyze_ssr(
                            client,
                            &ssr_url,
                            &content_selector_for_pump,
                            &mut record,
                        )
                        .await;
                    }

                    record.compute_indexability();
                    record.is_internal = is_same_domain(&root_for_pump, &record.url);
                    if let Some(sm_url) = sitemap_for_pump.get(&record.url) {
                        record.in_sitemap = Some(true);
                        record.sitemap_url = Some(sm_url.clone());
                    } else if !sitemap_for_pump.is_empty() {
                        record.in_sitemap = Some(false);
                    }
                    if let Err(e) = storage::insert_page(&pool_pages, crawl_id, &record).await {
                        tracing::warn!(error=%e, url=%record.url, "failed to persist page");
                    }
                    for link in &record.outlinks {
                        let entry = inlink_counts.entry(link.dst_url.clone()).or_insert((0, 0));
                        entry.0 += 1;
                        if link.csr_only {
                            entry.1 += 1;
                        }
                    }
                    if let Some((in_count, csr_in_count)) = inlink_counts.get(&record.url) {
                        record.inlinks_count = *in_count;
                        record.csr_inlinks_count = *csr_in_count;
                    }
                    tracing::info!(
                        total,
                        url = %record.url,
                        status = ?record.status,
                        content_type = ?record.content_type,
                        resp_time_ms = record.response_time.as_millis() as u64,
                        "sending page to UI"
                    );
                    if let Err(e) = tx_pages
                        .send_async(CrawlEvent::Page(Box::new(record)))
                        .await
                    {
                        tracing::error!(error=%e, "failed to send page event");
                    }

                    // Emit the resources Chrome loaded for this page as their
                    // own rows (CSS/JS/images/fonts), deduplicated across the
                    // crawl. No body is fetched or parsed here.
                    for resource in resource_timings {
                        if !resource.url.starts_with("http")
                            || !seen_resources.insert(resource.url.clone())
                        {
                            continue;
                        }
                        let ext_content_type = content_type_from_url(&resource.url);
                        // `<link>`-initiated entries without a recognizable asset
                        // extension are route prefetches/preloads (common in SPAs),
                        // not real subresources. We already crawl those routes as
                        // their own pages, so harvesting them here would duplicate
                        // them and, lacking a real content type, mislabel them.
                        if resource.initiator == "link" && ext_content_type.is_none() {
                            continue;
                        }
                        let content_type = resource
                            .content_type
                            .clone()
                            .or(ext_content_type)
                            .or_else(|| match resource.initiator.as_str() {
                                "script" => Some("text/javascript".to_string()),
                                "css" => Some("text/css".to_string()),
                                _ => None,
                            });
                        let mut resource_record = PageRecord {
                            url: resource.url.clone(),
                            status: (resource.status > 0).then_some(resource.status),
                            size_bytes: resource.size,
                            content_type,
                            response_time: Duration::from_millis(resource.duration),
                            is_internal: is_same_domain(&root_for_pump, &resource.url),
                            is_resource: true,
                            resource_initiator: (!resource.initiator.is_empty())
                                .then(|| resource.initiator.clone()),
                            ..Default::default()
                        };
                        if let Some(sm_url) = sitemap_for_pump.get(&resource.url) {
                            resource_record.in_sitemap = Some(true);
                            resource_record.sitemap_url = Some(sm_url.clone());
                        } else if !sitemap_for_pump.is_empty() {
                            resource_record.in_sitemap = Some(false);
                        }
                        if let Err(e) =
                            storage::insert_page(&pool_pages, crawl_id, &resource_record).await
                        {
                            tracing::warn!(error=%e, url=%resource_record.url, "failed to persist resource");
                        }
                        if let Err(e) = tx_pages
                            .send_async(CrawlEvent::Page(Box::new(resource_record)))
                            .await
                        {
                            tracing::error!(error=%e, "failed to send resource event");
                        }
                    }

                    if total.is_multiple_of(10) {
                        let _ = tx_pages
                            .send_async(CrawlEvent::Progress {
                                crawled: total,
                                queued: 0,
                            })
                            .await;
                    }
                }
                total
            });

            let crawl = tokio::spawn(async move {
                website.crawl().await;
                website.unsubscribe();
            });

            // Cooperative cancellation poll.
            loop {
                if cancel_flag.load(Ordering::Relaxed) {
                    crawl.abort();
                    break;
                }
                if crawl.is_finished() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            let total = pump.await.unwrap_or(0);
            tracing::info!(total, "page pump finished");
            if let Err(e) = storage::finish_crawl(&pool, crawl_id).await {
                tracing::warn!(error=%e, "failed to finalize crawl record");
            }

            run_near_duplicate_analysis(&pool, crawl_id, config.near_duplicate_threshold).await;

            run_pagerank_analysis(&pool, crawl_id).await;

            run_hreflang_validation(&pool, crawl_id).await;

            if let Ok(orphans) = storage::load_sitemap_orphans(&pool, crawl_id).await {
                for orphan in &orphans {
                    let record = PageRecord {
                        url: orphan.page_url.clone(),
                        sitemap_url: Some(orphan.sitemap_url.clone()),
                        in_sitemap: Some(true),
                        is_internal: is_same_domain(&root_url, &orphan.page_url),
                        ..Default::default()
                    };
                    let _ = tx.send_async(CrawlEvent::Page(Box::new(record))).await;
                }
                if !orphans.is_empty() {
                    tracing::info!(count = orphans.len(), "found sitemap orphan URLs");
                }
            }

            {
                let blocked =
                    std::mem::take(&mut *blocked_urls.lock().unwrap_or_else(|e| e.into_inner()));
                let mut seen = std::collections::HashSet::new();
                for url in &blocked {
                    if seen.insert(url.clone()) {
                        let record = PageRecord {
                            url: url.clone(),
                            is_internal: is_same_domain(&root_url, url),
                            blocked_by_robots: Some(true),
                            ..Default::default()
                        };
                        let _ = tx.send_async(CrawlEvent::Page(Box::new(record))).await;
                    }
                }
                if !blocked.is_empty() {
                    tracing::info!(count = blocked.len(), "found robots.txt-blocked URLs");
                }
            }

            let _ = tx.send_async(CrawlEvent::Finished { total }).await;
        };

        (cancel, fut)
    }
}

pub fn channel() -> (Sender<CrawlEvent>, Receiver<CrawlEvent>) {
    flume::unbounded()
}

async fn run_near_duplicate_analysis(pool: &SqlitePool, crawl_id: i64, threshold: u8) {
    let pages = match storage::load_simhashes_for_crawl(pool, crawl_id).await {
        Ok(pages) => pages,
        Err(e) => {
            tracing::warn!(error=%e, "failed to load simhashes for near-duplicate analysis");
            return;
        }
    };

    if pages.len() < 2 {
        return;
    }

    let results = crate::crawl::similarity::find_near_duplicates(&pages, threshold);

    if let Err(e) = storage::update_near_duplicates(
        pool,
        crawl_id,
        &results
            .iter()
            .map(|r| {
                (
                    r.url.clone(),
                    r.closest_similarity_percent,
                    r.near_duplicate_count,
                    r.near_duplicate_urls.clone(),
                )
            })
            .collect::<Vec<_>>(),
    )
    .await
    {
        tracing::warn!(error=%e, "failed to persist near-duplicate results");
    }
}

async fn run_pagerank_analysis(pool: &SqlitePool, crawl_id: i64) {
    let link_rows = match sqlx::query_as::<_, (String, String)>(
        "SELECT src_url, dst_url FROM links WHERE crawl_id = ? AND kind = 'internal'",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error=%e, "failed to load links for PageRank");
            return;
        }
    };

    if link_rows.is_empty() {
        return;
    }

    let all_urls: std::collections::HashSet<String> = link_rows
        .iter()
        .flat_map(|(src, dst)| [src.clone(), dst.clone()])
        .collect();

    let url_count = all_urls.len();
    let url_index: std::collections::HashMap<String, usize> = all_urls
        .iter()
        .enumerate()
        .map(|(i, url)| (url.clone(), i))
        .collect();

    let mut outlinks_count = vec![0usize; url_count];
    let mut inlinks: Vec<Vec<usize>> = vec![Vec::new(); url_count];
    for (src, dst) in &link_rows {
        let Some(&src_idx) = url_index.get(src) else {
            continue;
        };
        let Some(&dst_idx) = url_index.get(dst) else {
            continue;
        };
        outlinks_count[src_idx] += 1;
        inlinks[dst_idx].push(src_idx);
    }

    let damping = 0.85f32;
    let iterations = 30;
    let mut scores = vec![1.0f32 / url_count as f32; url_count];

    for _ in 0..iterations {
        let mut new_scores = vec![(1.0 - damping) / url_count as f32; url_count];
        for node in 0..url_count {
            if outlinks_count[node] == 0 {
                let share = scores[node] * damping / url_count as f32;
                for new_score in &mut new_scores {
                    *new_score += share;
                }
            } else {
                let share = scores[node] * damping / outlinks_count[node] as f32;
                for &target in &inlinks[node] {
                    new_scores[target] += share;
                }
            }
        }
        scores = new_scores;
    }

    let max_score = scores.iter().copied().fold(0.0f32, f32::max);
    if max_score > 0.0 {
        for score in &mut scores {
            *score = (*score / max_score) * 100.0;
        }
    }

    let results: Vec<(String, f32)> = all_urls
        .into_iter()
        .enumerate()
        .map(|(i, url)| (url, scores[i]))
        .collect();

    if let Err(e) = storage::update_link_scores(pool, crawl_id, &results).await {
        tracing::warn!(error=%e, "failed to persist PageRank scores");
    }
}

async fn run_hreflang_validation(pool: &SqlitePool, crawl_id: i64) {
    let rows = match sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        r#"SELECT url, hreflang_tags_json, canonical FROM pages WHERE crawl_id = ?"#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error=%e, "failed to load pages for hreflang validation");
            return;
        }
    };

    struct PageInfo {
        hreflang_tags: Vec<(String, String)>,
        canonical: Option<String>,
    }

    let mut page_map: std::collections::HashMap<String, PageInfo> =
        std::collections::HashMap::new();

    for (url, hreflang_json, canonical) in &rows {
        let tags: Vec<(String, String)> = hreflang_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        if !tags.is_empty() {
            page_map.insert(
                url.clone(),
                PageInfo {
                    hreflang_tags: tags,
                    canonical: canonical.clone(),
                },
            );
        }
    }

    if page_map.is_empty() {
        return;
    }

    let mut all_issues: Vec<(String, Vec<crate::crawl::event::HreflangIssue>)> = Vec::new();

    for (page_url, info) in &page_map {
        let mut issues: Vec<crate::crawl::event::HreflangIssue> = Vec::new();

        let has_x_default = info
            .hreflang_tags
            .iter()
            .any(|(lang, _)| lang == "x-default");
        if !has_x_default {
            issues.push(crate::crawl::event::HreflangIssue::MissingXDefault);
        }

        for (lang, target_url) in &info.hreflang_tags {
            if lang != "x-default" && !is_valid_bcp47(lang) {
                issues.push(crate::crawl::event::HreflangIssue::InvalidLanguageCode {
                    code: lang.clone(),
                });
            }

            let target_info = page_map.get(target_url);
            let return_tag_exists = target_info.is_some_and(|target| {
                target
                    .hreflang_tags
                    .iter()
                    .any(|(return_lang, return_url)| {
                        return_url == page_url
                            && (return_lang == lang || lang.starts_with(&format!("{return_lang}-")))
                    })
            });
            if !return_tag_exists {
                issues.push(crate::crawl::event::HreflangIssue::MissingReturnTag {
                    lang: lang.clone(),
                    target_url: target_url.clone(),
                });
            }

            if let Some(target_info) = target_info
                && let Some(ref canonical) = target_info.canonical
                && canonical != target_url
            {
                issues.push(crate::crawl::event::HreflangIssue::NonCanonicalUrl {
                    hreflang_url: target_url.clone(),
                });
            }
        }

        if !issues.is_empty() {
            all_issues.push((page_url.clone(), issues));
        }
    }

    if let Err(e) = storage::update_hreflang_issues(pool, crawl_id, &all_issues).await {
        tracing::warn!(error=%e, "failed to persist hreflang issues");
    }
}

fn is_valid_bcp47(code: &str) -> bool {
    if code == "x-default" {
        return true;
    }
    let parts: Vec<&str> = code.split('-').collect();
    if parts.is_empty() {
        return false;
    }
    let primary = parts[0];
    if primary.len() != 2 && primary.len() != 3 {
        return false;
    }
    if !primary.chars().all(|c| c.is_ascii_lowercase()) {
        return false;
    }
    for part in parts.iter().skip(1) {
        if part.len() < 2 || part.len() > 8 {
            return false;
        }
        if !part.chars().all(|c| c.is_ascii_alphanumeric()) {
            return false;
        }
    }
    true
}

const PERF_OBSERVER_JS: &str = r#"
window.__sr_cls = 0;
try {
    new PerformanceObserver(function(list) {
        var e = list.getEntries();
        for (var i = 0; i < e.length; i++) {
            if (!e[i].hadRecentInput) window.__sr_cls += e[i].value;
        }
    }).observe({ type: 'layout-shift', buffered: true });
} catch(e) {}
try {
    window.__sr_lcp_entries = [];
    new PerformanceObserver(function(list) {
        window.__sr_lcp_entries = window.__sr_lcp_entries.concat(list.getEntries());
    }).observe({ type: 'largest-contentful-paint', buffered: true });
} catch(e) {}
"#;

const METRICS_AUTOMATION_JS: &str = r#"
(function() {
    try {
        var nav = performance.getEntriesByType('navigation')[0];
        var ttfb = nav ? Math.max(0, Math.round(nav.responseStart - nav.requestStart)) : null;
        var paint = performance.getEntriesByType('paint');
        var fcp = null;
        for (var i = 0; i < paint.length; i++) {
            if (paint[i].name === 'first-contentful-paint') {
                fcp = Math.round(paint[i].startTime);
                break;
            }
        }
        var lcp = null;
        var lcpEntries = performance.getEntriesByType('largest-contentful-paint');
        if (lcpEntries && lcpEntries.length) {
            lcp = Math.round(lcpEntries[lcpEntries.length - 1].startTime);
        }
        if (lcp == null) lcp = fcp;
        var cls = 0;
        var shifts = performance.getEntriesByType('layout-shift');
        for (var j = 0; j < shifts.length; j++) {
            if (!shifts[j].hadRecentInput) cls += shifts[j].value;
        }
        cls = Math.round(cls * 1000) / 1000;
        var data = { ttfb: ttfb, lcp: lcp, cls: cls, fcp: fcp };
        var existing = document.getElementById('__sr_metrics');
        if (existing) existing.remove();
        var s = document.createElement('script');
        s.id = '__sr_metrics';
        s.type = 'application/json';
        s.textContent = JSON.stringify(data);
        (document.head || document.documentElement).appendChild(s);
    } catch (e) {}
})()
"#;

/// Maps a URL's file extension to a MIME type. Used as a fallback for
/// subresources that spider fetches without response headers, so the Internal
/// tab can still categorize them by content type.
/// Heuristic body sniff: does this look like an HTML document? Used to recover
/// the correct content type for navigated pages when Chrome's request
/// interception reports a subresource's headers for the document. Checks the
/// leading bytes (after any BOM/whitespace) for the doctype or an `<html>` tag.
fn looks_like_html(body: &str) -> bool {
    let head = body
        .trim_start_matches('\u{feff}')
        .trim_start()
        .get(..512)
        .unwrap_or(body)
        .to_ascii_lowercase();
    head.contains("<!doctype html") || head.contains("<html")
}

fn content_type_from_url(url: &str) -> Option<String> {
    let path = url::Url::parse(url)
        .ok()
        .map(|u| u.path().to_ascii_lowercase())
        .unwrap_or_else(|| url.to_ascii_lowercase());
    let ext = path.rsplit_once('.').map(|(_, ext)| ext)?;
    let mime = match ext {
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "eot" => "application/vnd.ms-fontobject",
        "xml" => "application/xml",
        "txt" => "text/plain",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        _ => return None,
    };
    Some(mime.to_string())
}

pub fn is_same_domain(root: &str, url: &str) -> bool {
    let Ok(root_parsed) = url::Url::parse(root) else {
        return true;
    };
    let Ok(url_parsed) = url::Url::parse(url) else {
        return false;
    };
    root_parsed.host_str() == url_parsed.host_str()
}

fn build_ssr_client(config: &CrawlConfig) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1) as u64));
    if let Some(ua) = config.user_agent.as_deref().filter(|ua| !ua.is_empty()) {
        builder = builder.user_agent(ua);
    }
    if !config.extra_headers.is_empty() {
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.extra_headers {
            if let Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                && let Ok(val) = reqwest::header::HeaderValue::from_str(value)
            {
                headers.insert(name, val);
            }
        }
        builder = builder.default_headers(headers);
    }
    builder.build()
}

async fn fetch_and_analyze_ssr(
    client: &reqwest::Client,
    url: &str,
    content_selector: &str,
    record: &mut PageRecord,
) {
    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(e) => {
            tracing::warn!(url = %url, error=%e, "SSR fetch failed; skipping SSR diff");
            return;
        }
    };
    match response.text().await {
        Ok(raw_html) => {
            crate::crawl::analyzers::analyze_ssr(record, &raw_html, content_selector);
        }
        Err(e) => {
            tracing::warn!(url = %url, error=%e, "reading SSR response body failed");
        }
    }
}

async fn collect_performance_metrics(page: &spider::page::Page, record: &mut PageRecord) {
    let Some(chrome_page) = page.get_chrome_page() else {
        return;
    };

    let js = r#"
    (function() {
        var result = { ttfb: null, lcp: null, cls: null, fcp: null };
        try {
            var nav = performance.getEntriesByType('navigation')[0];
            if (nav) result.ttfb = Math.max(0, Math.round(nav.responseStart - nav.requestStart));
        } catch(e) {}
        try {
            var paint = performance.getEntriesByType('paint');
            for (var p = 0; p < paint.length; p++) {
                if (paint[p].name === 'first-contentful-paint') {
                    result.fcp = Math.round(paint[p].startTime);
                    break;
                }
            }
        } catch(e) {}
        try {
            var lcp = null;
            if (window.__sr_lcp_entries && window.__sr_lcp_entries.length > 0) {
                lcp = Math.round(window.__sr_lcp_entries[window.__sr_lcp_entries.length - 1].startTime);
            }
            if (lcp == null) {
                var entries = performance.getEntriesByType('largest-contentful-paint');
                if (entries && entries.length > 0) lcp = Math.round(entries[entries.length - 1].startTime);
            }
            if (lcp == null) {
                var paint = performance.getEntriesByType('paint');
                for (var i = 0; i < paint.length; i++) {
                    if (paint[i].name === 'first-contentful-paint') {
                        lcp = Math.round(paint[i].startTime);
                        break;
                    }
                }
            }
            result.lcp = lcp;
        } catch(e) {}
        try {
            var cls = window.__sr_cls || 0;
            if (cls === 0) {
                var shifts = performance.getEntriesByType('layout-shift');
                for (var j = 0; j < shifts.length; j++) {
                    if (!shifts[j].hadRecentInput) cls += shifts[j].value;
                }
            }
            result.cls = Math.round(cls * 1000) / 1000;
        } catch(e) {}
        return result;
    })()
    "#;

    let params = match spider::chromiumoxide::cdp::js_protocol::runtime::EvaluateParams::builder()
        .expression(js)
        .await_promise(false)
        .return_by_value(true)
        .build()
    {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build perf metrics EvaluateParams");
            return;
        }
    };

    let eval_result = match chrome_page.evaluate(params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(url = %page.get_url(), error=%e, "perf metrics evaluate failed");
            return;
        }
    };

    let Some(value) = eval_result.value() else {
        return;
    };

    if let Some(obj) = value.as_object() {
        record.ttfb_ms = obj.get("ttfb").and_then(|v| v.as_u64());
        record.lcp_ms = obj.get("lcp").and_then(|v| v.as_u64());
        record.cls = obj.get("cls").and_then(|v| v.as_f64());
        record.fcp_ms = obj.get("fcp").and_then(|v| v.as_u64());
        tracing::debug!(
            url = %page.get_url(),
            ttfb = ?record.ttfb_ms,
            lcp = ?record.lcp_ms,
            cls = ?record.cls,
            fcp = ?record.fcp_ms,
            "collected perf metrics"
        );
    }
}

/// A single resource Chrome loaded while rendering a page, read from the
/// Resource Timing API. Lets us record CSS/JS/image/font resources without
/// re-fetching them.
#[derive(serde::Deserialize)]
struct ResourceTiming {
    url: String,
    #[serde(default, rename = "type")]
    initiator: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    status: u16,
    #[serde(default)]
    duration: u64,
    #[serde(default)]
    content_type: Option<String>,
}

/// Reads `performance.getEntriesByType('resource')` from the rendered page.
/// This is a single lightweight evaluate plus a small JSON parse; it never
/// touches the network or parses any resource body.
async fn collect_resource_timings(page: &spider::page::Page) -> Vec<ResourceTiming> {
    let Some(chrome_page) = page.get_chrome_page() else {
        return Vec::new();
    };

    let js = r#"
    (function() {
        try {
            var out = [];
            var entries = performance.getEntriesByType('resource');
            for (var i = 0; i < entries.length; i++) {
                var e = entries[i];
                out.push({
                    url: e.name,
                    type: e.initiatorType || '',
                    size: Math.round(e.transferSize || e.encodedBodySize || e.decodedBodySize || 0),
                    status: (typeof e.responseStatus === 'number') ? e.responseStatus : 0,
                    duration: Math.round(e.duration || 0),
                    content_type: e.contentType || null
                });
            }
            return JSON.stringify(out);
        } catch (e) { return "[]"; }
    })()
    "#;

    let params = match spider::chromiumoxide::cdp::js_protocol::runtime::EvaluateParams::builder()
        .expression(js)
        .await_promise(false)
        .return_by_value(true)
        .build()
    {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build resource timing EvaluateParams");
            return Vec::new();
        }
    };

    let eval_result = match chrome_page.evaluate(params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(url = %page.get_url(), error=%e, "resource timing evaluate failed");
            return Vec::new();
        }
    };

    eval_result
        .value()
        .and_then(|v| v.as_str())
        .and_then(|json| serde_json::from_str::<Vec<ResourceTiming>>(json).ok())
        .unwrap_or_default()
}

async fn fetch_axe_js() -> Option<String> {
    let axe_url = "https://cdnjs.cloudflare.com/ajax/libs/axe-core/4.10.2/axe.min.js";
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build axe client; a11y scans disabled");
            return None;
        }
    };
    match client.get(axe_url).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => Some(text),
            Err(e) => {
                tracing::warn!(error=%e, "reading axe.js body failed; a11y scans disabled");
                None
            }
        },
        Err(e) => {
            tracing::warn!(error=%e, "axe.js fetch failed; a11y scans disabled");
            None
        }
    }
}

async fn collect_a11y_violations(page: &spider::page::Page, record: &mut PageRecord, axe_js: &str) {
    let Some(chrome_page) = page.get_chrome_page() else {
        return;
    };

    let flush_js = "(async function() { await new Promise(function(resolve) { setTimeout(resolve, 0); }); })()";
    let flush_params = spider::chromiumoxide::cdp::js_protocol::runtime::EvaluateParams::builder()
        .expression(flush_js)
        .await_promise(true)
        .return_by_value(true)
        .build();
    if let Ok(params) = flush_params
        && let Err(e) = chrome_page.evaluate(params).await
    {
        tracing::debug!(error=%e, "rAF flush failed before a11y scan");
    }

    if chrome_page.evaluate(axe_js).await.is_err() {
        return;
    }

    let run_js = "(() => new Promise(resolve => axe.run({resultTypes: ['violations'], rules: {region: {enabled: false}}}, (err, results) => resolve(err ? null : results.violations.map(v => ({id: v.id, impact: v.impact, tags: v.tags, nodes: v.nodes.map(n => ({target: n.target.join(','), html: n.html}))}))))))()";
    let Ok(eval_result) = chrome_page.evaluate(run_js).await else {
        return;
    };

    let Some(value) = eval_result.value() else {
        return;
    };

    let Some(violations) = value.as_array() else {
        return;
    };

    for violation in violations {
        let Some(obj) = violation.as_object() else {
            continue;
        };
        let rule = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let impact = obj
            .get("impact")
            .and_then(|v| v.as_str())
            .unwrap_or("minor")
            .to_string();

        let is_error = impact == "critical" || impact == "serious";
        if is_error {
            record.a11y_errors = record.a11y_errors.saturating_add(1);
        } else {
            record.a11y_warnings = record.a11y_warnings.saturating_add(1);
        }

        if let Some(nodes) = obj.get("nodes").and_then(|v| v.as_array()) {
            for node in nodes {
                let target = node
                    .as_object()
                    .and_then(|n| n.get("target"))
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string());
                let html = node
                    .as_object()
                    .and_then(|n| n.get("html"))
                    .and_then(|h| h.as_str())
                    .map(|s| s.to_string());
                record.a11y_issues.push(A11yIssue {
                    rule: rule.clone(),
                    impact: impact.clone(),
                    target,
                    html,
                });
            }
        } else {
            record.a11y_issues.push(A11yIssue {
                rule,
                impact,
                target: None,
                html: None,
            });
        }
    }
}
