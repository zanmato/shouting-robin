use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use flume::{Receiver, Sender};
use gpui::{App, Global};
use spider::configuration::WaitForIdleNetwork;
use spider::features::chrome_common::{RequestInterceptConfiguration, WebAutomation};
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
        let fut = async move {
            let crawl_id = match storage::create_crawl(&pool, &root_url).await {
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
                    root_url: root_url.clone(),
                })
                .await;

            let sitemap_urls = crate::crawl::sitemap::discover_sitemaps(&root_url).await;
            let sitemap_entries = crate::crawl::sitemap::fetch_sitemap_urls(&sitemap_urls, 3).await;
            if let Err(e) = storage::insert_sitemap_urls(&pool, crawl_id, &sitemap_entries).await {
                tracing::warn!(error=%e, "failed to persist sitemap URLs");
            }

            let mut sitemap_lookup: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            for entry in &sitemap_entries {
                sitemap_lookup
                    .entry(entry.page_url.clone())
                    .or_insert_with(|| entry.sitemap_url.clone());
            }

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

            match render_mode {
                RenderMode::Http => {}
                RenderMode::Chrome => {
                    let mut automation_map =
                        spider::features::chrome_common::AutomationScriptsMap::default();
                    automation_map.insert(
                        "/".to_string(),
                        vec![WebAutomation::Evaluate(METRICS_AUTOMATION_JS.to_string())],
                    );
                    website
                        .with_chrome_intercept(RequestInterceptConfiguration::new(true))
                        .with_stealth(true)
                        .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(
                            Duration::from_secs(30),
                        ))))
                        .with_automation_scripts(Some(automation_map));
                }
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
            let pump = tokio::spawn(async move {
                let mut subscribe_guard = subscribe_guard;
                let mut total: u64 = 0;
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
                        response_time: page
                            .get_response_duration()
                            .unwrap_or_else(|| page.get_duration_elapsed()),
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
                                .to_string(),
                        );
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
                    if original_url != final_url {
                        record.redirect_url = Some(final_url.to_string());
                        record.redirect_status = Some(301);
                    }
                    if !html_bytes.is_empty()
                        && let Ok(html) = std::str::from_utf8(html_bytes)
                    {
                        tracing::debug!(
                            url = %record.url,
                            html_len = html.len(),
                            "received html for analysis"
                        );
                        crate::crawl::analyzers::analyze_html(
                            &mut record,
                            html,
                            &content_selector_for_pump,
                        );
                    }

                    if chrome_mode {
                        if page.get_chrome_page().is_some() {
                            collect_performance_metrics(&page, &mut record).await;
                            collect_a11y_violations(&page, &mut record).await;
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
                for target in 0..url_count {
                    new_scores[target] += share;
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

            if let Some(target_info) = target_info {
                if let Some(ref canonical) = target_info.canonical {
                    if canonical != target_url {
                        issues.push(crate::crawl::event::HreflangIssue::NonCanonicalUrl {
                            hreflang_url: target_url.clone(),
                        });
                    }
                }
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

const METRICS_AUTOMATION_JS: &str = r#"
(function() {
    try {
        var nav = performance.getEntriesByType('navigation')[0];
        var ttfb = nav ? Math.max(0, Math.round(nav.responseStart - nav.requestStart)) : null;
        var lcp = null;
        var lcpEntries = performance.getEntriesByType('largest-contentful-paint');
        if (lcpEntries && lcpEntries.length) {
            lcp = Math.round(lcpEntries[lcpEntries.length - 1].startTime);
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
        var cls = 0;
        var shifts = performance.getEntriesByType('layout-shift');
        for (var j = 0; j < shifts.length; j++) {
            if (!shifts[j].hadRecentInput) cls += shifts[j].value;
        }
        cls = Math.round(cls * 1000) / 1000;
        var inp = null;
        var events = performance.getEntriesByType('event');
        var maxDur = 0;
        for (var k = 0; k < events.length; k++) {
            if (events[k].duration > maxDur) maxDur = events[k].duration;
        }
        if (maxDur > 0) inp = Math.round(maxDur);
        var data = { ttfb: ttfb, lcp: lcp, cls: cls, inp: inp };
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

pub fn is_same_domain(root: &str, url: &str) -> bool {
    let Ok(root_parsed) = url::Url::parse(root) else {
        return true;
    };
    let Ok(url_parsed) = url::Url::parse(url) else {
        return false;
    };
    root_parsed.host_str() == url_parsed.host_str()
}

async fn collect_performance_metrics(page: &spider::page::Page, record: &mut PageRecord) {
    let Some(chrome_page) = page.get_chrome_page() else {
        return;
    };

    let js = r#"
    (async function() {
        await new Promise(function(resolve) { setTimeout(resolve, 0); });
        var result = { ttfb: null, lcp: null, cls: null, inp: null };
        try {
            var nav = performance.getEntriesByType('navigation')[0];
            if (nav) result.ttfb = Math.round(nav.responseStart - nav.requestStart);
        } catch(e) {}
        try {
            var lcpEntries = performance.getEntriesByType('largest-contentful-paint');
            if (lcpEntries.length > 0) result.lcp = Math.round(lcpEntries[lcpEntries.length - 1].startTime);
        } catch(e) {}
        try {
            var shifts = performance.getEntriesByType('layout-shift');
            var cls = 0;
            for (var i = 0; i < shifts.length; i++) if (!shifts[i].hadRecentInput) cls += shifts[i].value;
            result.cls = Math.round(cls * 1000) / 1000;
        } catch(e) {}
        try {
            var events = performance.getEntriesByType('event');
            var maxDur = 0;
            for (var i = 0; i < events.length; i++) if (events[i].duration > maxDur) maxDur = events[i].duration;
            if (maxDur > 0) result.inp = Math.round(maxDur);
        } catch(e) {}
        return result;
    })()
    "#;

    let params = match spider::chromiumoxide::cdp::js_protocol::runtime::EvaluateParams::builder()
        .expression(js)
        .await_promise(true)
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
        record.inp_ms = obj.get("inp").and_then(|v| v.as_u64());
        tracing::debug!(
            url = %page.get_url(),
            ttfb = ?record.ttfb_ms,
            lcp = ?record.lcp_ms,
            cls = ?record.cls,
            inp = ?record.inp_ms,
            "collected perf metrics"
        );
    }
}

async fn collect_a11y_violations(page: &spider::page::Page, record: &mut PageRecord) {
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

    let axe_url = "https://cdnjs.cloudflare.com/ajax/libs/axe-core/4.10.2/axe.min.js";
    let axe_js = match reqwest::get(axe_url).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(_) => return,
        },
        Err(_) => return,
    };

    if chrome_page.evaluate(axe_js.as_str()).await.is_err() {
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
