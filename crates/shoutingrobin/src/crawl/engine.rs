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
use crate::crawl::event::{A11yIssue, CrawlEvent, HreflangSource, PageRecord, SubresourceKind};
use crate::crawl::render_mode::RenderMode;
use crate::crawl::resources::ResourceKind;
use crate::crawl::url_norm::urls_equivalent;
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
            let root_url = resolve_start_url(&root_url, &config).await;

            let crawl_id = match storage::create_crawl(&pool, &root_url, mode_str, &config).await {
                Ok(id) => id,
                Err(e) => {
                    if let Err(send_err) = tx
                        .send_async(CrawlEvent::Error {
                            url: root_url.clone(),
                            message: format!("Failed to create crawl: {e}"),
                        })
                        .await
                    {
                        tracing::warn!(error=%send_err, "failed to send crawl error event");
                    }
                    return;
                }
            };

            if let Err(e) = tx
                .send_async(CrawlEvent::Started {
                    crawl_id,
                    root_url: root_url.clone(),
                })
                .await
            {
                tracing::warn!(error=%e, "failed to send started event");
            }

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

            let mut sitemap_lookup: SitemapLookup = std::collections::HashMap::new();
            for entry in &sitemap_entries {
                sitemap_lookup
                    .entry(entry.page_url.clone())
                    .or_insert_with(|| {
                        (
                            entry.sitemap_url.clone(),
                            entry.lastmod.clone(),
                            entry.hreflang.clone(),
                        )
                    });
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
            // A 429 or 5xx is retried after a jittered back-off (spider honours
            // `Retry-After`) before it is recorded, so a burst of rate limiting
            // is absorbed rather than reported as the site's error pages.
            website.with_retry(RATE_LIMIT_RETRIES);

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
                RenderMode::Http => {}
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
                    // Each page in a window of its own: a background tab is a
                    // hidden document, and Chrome reports no LCP for a page
                    // that was never visible, which left every page after the
                    // first with LCP equal to FCP. A desktop viewport, rather
                    // than the 800x600 default, is also what the vitals are
                    // measured against.
                    let mut viewport = spider::configuration::Viewport::new(1366, 768);
                    viewport.own_window = true;
                    // `RequestInterceptConfiguration::new(enabled)` ties every
                    // block to the one switch, so "block images" also dropped
                    // stylesheets (wrecking LCP and CLS) and leaving it off let
                    // every analytics beacon through as a resource row: 2,240
                    // of a reference crawl's 2,436 resources were Google
                    // Analytics and ad-network pings with per-request query
                    // strings. Analytics and ads are never the site's assets.
                    let intercept = RequestInterceptConfiguration {
                        enabled: true,
                        block_visuals: config.block_images,
                        block_analytics: true,
                        block_ads: true,
                        block_stylesheets: false,
                        block_javascript: false,
                        ..Default::default()
                    };
                    website
                        .with_viewport(Some(viewport))
                        .with_chrome_intercept(intercept)
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

            // spider extracts hrefs with a streaming HTML rewriter, which hands
            // back the raw attribute text with its character references intact.
            // Left alone, a href of `?a=1&amp;b=2` is queued verbatim, so the
            // crawler requests a URL nobody linked to and never reaches the real
            // one (missing any redirect it serves). Decode on the way into the
            // frontier, which is the hook spider provides for rewriting URLs.
            website.with_on_link_find_callback(Some(std::sync::Arc::new(
                |url: spider::CaseInsensitiveString, source: Option<String>| {
                    let raw = url.to_string();
                    let decoded = crate::crawl::url_norm::decode_entities(&raw);
                    if decoded == raw {
                        (url, source)
                    } else {
                        (
                            spider::CaseInsensitiveString::from(decoded.as_str()),
                            source,
                        )
                    }
                },
            )));

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
                if let Err(e) = tx
                    .send_async(CrawlEvent::Error {
                        url: root_url.clone(),
                        message: "Failed to parse root URL".into(),
                    })
                    .await
                {
                    tracing::warn!(error=%e, "failed to send root URL error event");
                }
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
            let axe_js = chrome_mode.then_some(AXE_JS);
            let pump = tokio::spawn(async move {
                let mut subscribe_guard = subscribe_guard;
                let mut total: u64 = 0;
                // dst_url -> (total inlinks, csr-only inlinks, distinct source URLs)
                let mut inlink_counts: std::collections::HashMap<
                    String,
                    (u32, u32, std::collections::HashSet<String>),
                > = std::collections::HashMap::new();
                // Resource URLs already emitted as their own rows. The same
                // asset (a shared stylesheet, say) shows up on most pages, so
                // we record each one once.
                let mut seen_resources: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                // Every resource URL the pages point at, in discovery order,
                // for the status-check pass once the crawl is done.
                let mut discovered_resources: Vec<(String, ResourceKind)> = Vec::new();
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
                                value.to_str().ok().map(|v| {
                                    // Chrome's CDP wraps every header value in JSON
                                    // quotes; strip a single surrounding pair so
                                    // they display cleanly. Only one layer is
                                    // removed so legitimately quoted values (e.g. an
                                    // ETag) keep their inner quotes.
                                    let trimmed = if chrome_mode {
                                        v.strip_prefix('"')
                                            .and_then(|s| s.strip_suffix('"'))
                                            .unwrap_or(v)
                                    } else {
                                        v
                                    };
                                    (name.to_string(), trimmed.to_string())
                                })
                            })
                            .collect();
                    }
                    let original_url = page.get_url();
                    let final_url = page.get_url_final();
                    // Only treat this as a redirect when spider reports a
                    // genuinely different, non-empty final URL. In Chrome mode
                    // get_url_final() can come back empty or normalized even when
                    // nothing redirected; mistaking that for a redirect would
                    // wrongly skip analysis of a perfectly normal page. Compare
                    // by parsed URL so a trailing-slash normalization (Chrome
                    // turns `https://example.com` into `https://example.com/`)
                    // is not mistaken for a redirect, which would drop the start
                    // page's outlinks/title/H1 entirely.
                    let has_final_url =
                        !final_url.is_empty() && !urls_equivalent(original_url, final_url);
                    // A redirect the crawler didn't follow arrives as a 3xx with
                    // no final URL. It still has no document of its own, so it
                    // must skip analysis too, or its empty body is audited as the
                    // page's own content and reported as a missing title and H1.
                    let status_is_redirect =
                        record.status.is_some_and(|code| (300..400).contains(&code));
                    let redirected = has_final_url || status_is_redirect;
                    if has_final_url {
                        record.redirect_url = Some(final_url.to_string());
                        // When spider followed the redirect, `status` is the
                        // target's, so fall back to the usual 301.
                        record.redirect_status = record
                            .status
                            .filter(|code| (300..400).contains(code))
                            .or(Some(301));
                        // A redirect response has no document of its own. Any
                        // content type inferred above belongs to the target, or
                        // (under Chrome interception) to an unrelated subresource
                        // whose headers spider handed us, so it would mislabel the
                        // redirect (e.g. `/` -> `/sv` shown as text/css). Leave it
                        // blank rather than showing a bogus mime type.
                        record.content_type = None;
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
                    // When there's no redirect, audit the row's own URL rather
                    // than a possibly-empty final URL.
                    let analyzed_url = if has_final_url {
                        final_url
                    } else {
                        record.url.as_str()
                    };
                    let analyzed_external = !is_same_domain(&root_for_pump, analyzed_url);
                    // A PDF or feed parsed as HTML reports a missing title,
                    // H1, body and content on a document that has none of
                    // those by design.
                    let skip_analysis =
                        redirected || analyzed_external || !record.is_html_document();
                    // A 2xx HTML document with no bytes at all is a page the
                    // crawler could not read: Chrome gave up on the render, or
                    // the server answered with nothing. Left alone it reports
                    // as a healthy page with a missing title; recording it as
                    // bodiless and empty puts it where a reader looks for
                    // broken pages.
                    if !skip_analysis
                        && html_bytes.is_empty()
                        && record
                            .status
                            .is_some_and(|status| (200..300).contains(&status))
                    {
                        record.has_body_tag = Some(false);
                        record.word_count = Some(0);
                    }
                    if !skip_analysis
                        && !html_bytes.is_empty()
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

                    let mut resource_timings: Vec<ResourceTiming> = Vec::new();
                    // The server's HTML for this page, if Chrome still has it.
                    let mut chrome_raw_html: Option<String> = None;
                    if chrome_mode {
                        if skip_analysis {
                            // Redirect source or off-domain landing: don't measure
                            // web vitals, scan a11y, or harvest its subresources.
                            page.close_page().await;
                        } else if page.get_chrome_page().is_some() {
                            collect_performance_metrics(&page, &mut record).await;
                            // `get_duration_elapsed` spans the whole render
                            // including paced auxiliary requests, which read as
                            // five-second "response times" on a 70 ms server.
                            // The navigation entry's TTFB is the comparable
                            // number to an HTTP crawl.
                            if let Some(ttfb) = record.ttfb_ms {
                                record.response_time = Duration::from_millis(ttfb);
                            }
                            if let Some(axe_js) = axe_js {
                                collect_a11y_violations(&page, &mut record, axe_js).await;
                            }
                            resource_timings = collect_resource_timings(&page).await;
                            // Read while the tab is open; the SSR diff below runs
                            // once it is closed.
                            chrome_raw_html = raw_html_from_chrome(&page, &record.url).await;
                            page.close_page().await;
                        } else {
                            tracing::warn!(
                                url = %record.url,
                                "chrome page handle missing; spider used the HTTP path \
                                 (check /tmp/chromiumoxide-runner/SingletonLock or chrome config)"
                            );
                        }
                    }

                    if let Some(guard) = subscribe_guard.as_mut() {
                        guard.inc();
                    }

                    // The SSR diff needs the raw server HTML, not chrome. Chrome
                    // usually still holds it, so the diff costs nothing; the
                    // fetch is for when it does not, and for a row left with no
                    // headers of its own, which only a request can supply.
                    // Skipped for redirects and off-domain landings, which carry
                    // no analysis.
                    if !skip_analysis {
                        match chrome_raw_html {
                            Some(ref raw_html) if !record.headers.is_empty() => {
                                crate::crawl::analyzers::analyze_ssr(
                                    &mut record,
                                    raw_html,
                                    &content_selector_for_pump,
                                );
                            }
                            _ => {
                                if let Some(client) = ssr_client.as_ref() {
                                    let ssr_url = record.url.clone();
                                    fetch_and_analyze_ssr(
                                        client,
                                        &ssr_url,
                                        &content_selector_for_pump,
                                        &mut record,
                                    )
                                    .await;
                                }
                            }
                        }
                    }

                    // hreflang can also arrive in a `Link:` response header,
                    // which is how non-HTML documents annotate alternates and
                    // how some sites annotate every page without touching the
                    // markup. Merged into the same set the HTML produced. After
                    // the SSR fetch, so it reads the document's own headers on a
                    // row whose reported ones turned out to be another
                    // request's.
                    if !skip_analysis && let Ok(base) = url::Url::parse(&record.url) {
                        let header_tags: Vec<(String, String)> = record
                            .headers
                            .iter()
                            .filter(|(name, _)| name.eq_ignore_ascii_case("link"))
                            .flat_map(|(_, value)| {
                                crate::crawl::analyzers::parse_link_header_hreflang(value, &base)
                            })
                            .collect();
                        crate::crawl::analyzers::merge_hreflang_tags(
                            &mut record,
                            header_tags,
                            HreflangSource::HttpHeader,
                        );
                    }

                    record.compute_indexability();
                    record.is_internal = is_same_domain(&root_for_pump, &record.url);
                    let sitemap_key = crate::crawl::url_norm::normalize_url(&record.url)
                        .unwrap_or_else(|| record.url.clone());
                    if let Some((sm_url, lastmod, hreflang)) = sitemap_for_pump.get(&sitemap_key) {
                        record.in_sitemap = Some(true);
                        record.sitemap_url = Some(sm_url.clone());
                        record.sitemap_lastmod = lastmod.clone();
                        if !skip_analysis {
                            crate::crawl::analyzers::merge_hreflang_tags(
                                &mut record,
                                hreflang.clone(),
                                HreflangSource::Sitemap,
                            );
                        }
                    } else if !sitemap_for_pump.is_empty() {
                        record.in_sitemap = Some(false);
                    }
                    if let Err(e) = storage::insert_page(&pool_pages, crawl_id, &record).await {
                        tracing::warn!(error=%e, url=%record.url, "failed to persist page");
                    }
                    for image in &record.images {
                        discovered_resources.push((image.src.clone(), ResourceKind::Image));
                    }
                    for subresource in &record.subresources {
                        let kind = match subresource.kind {
                            SubresourceKind::Stylesheet => ResourceKind::Stylesheet,
                            SubresourceKind::Script => ResourceKind::Script,
                        };
                        discovered_resources.push((subresource.url.clone(), kind));
                    }
                    for link in &record.outlinks {
                        if !is_same_domain(&root_for_pump, &link.dst_url) {
                            discovered_resources
                                .push((link.dst_url.clone(), ResourceKind::ExternalLink));
                        }
                        let entry = inlink_counts.entry(link.dst_url.clone()).or_insert((
                            0,
                            0,
                            std::collections::HashSet::new(),
                        ));
                        entry.0 += 1;
                        if link.csr_only {
                            entry.1 += 1;
                        }
                        entry.2.insert(record.url.clone());
                    }
                    // Only pages crawled so far have contributed their links, so
                    // this is a lower bound that grows as the crawl proceeds. It
                    // exists to give the live grid a non-zero figure; the real
                    // counts come from `load_pages_for_crawl`, which aggregates
                    // the whole `links` table once the crawl finishes.
                    if let Some((in_count, csr_in_count, sources)) = inlink_counts.get(&record.url)
                    {
                        record.inlinks_count = *in_count;
                        record.csr_inlinks_count = *csr_in_count;
                        record.unique_inlinks_count = sources.len() as u32;
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
                        let sitemap_key = crate::crawl::url_norm::normalize_url(&resource.url)
                            .unwrap_or_else(|| resource.url.clone());
                        if let Some((sm_url, lastmod, _)) = sitemap_for_pump.get(&sitemap_key) {
                            resource_record.in_sitemap = Some(true);
                            resource_record.sitemap_url = Some(sm_url.clone());
                            resource_record.sitemap_lastmod = lastmod.clone();
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
                }
                (total, discovered_resources)
            });

            // Chrome mode renders each page in a browser; Http mode forces
            // spider's plain-HTTP path (crawl_raw), which also keeps the
            // sitemap pass off Chrome.
            let use_chrome = matches!(render_mode, RenderMode::Chrome);
            let crawl = tokio::spawn(async move {
                if use_chrome {
                    website.crawl().await;
                } else {
                    website.crawl_raw().await;
                }
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

            let (total, discovered_resources) = pump.await.unwrap_or_else(|e| {
                tracing::warn!(error=%e, "page pump task failed");
                (0, Vec::new())
            });
            tracing::info!(total, "page pump finished");

            resolve_redirect_chains(&pool, crawl_id, &tx, &config, &root_url, &cancel_flag).await;

            fetch_declared_canonical_targets(
                &pool,
                crawl_id,
                &tx,
                &config,
                &root_url,
                &cancel_flag,
            )
            .await;

            check_discovered_resources(
                &pool,
                crawl_id,
                &tx,
                &config,
                &root_url,
                discovered_resources,
                &cancel_flag,
            )
            .await;

            if let Err(e) = storage::finish_crawl(&pool, crawl_id).await {
                tracing::warn!(error=%e, "failed to finalize crawl record");
            }

            // A failed pass leaves its columns empty, which reads as "no
            // issues" unless the reader is told otherwise.
            let passes: [(&str, Result<(), sqlx::Error>); 4] = [
                (
                    "near-duplicate analysis",
                    run_near_duplicate_analysis(&pool, crawl_id, config.near_duplicate_threshold)
                        .await,
                ),
                (
                    "crawl depth analysis",
                    run_crawl_depth_analysis(&pool, crawl_id, &root_url).await,
                ),
                (
                    "link score analysis",
                    run_pagerank_analysis(&pool, crawl_id).await,
                ),
                (
                    "hreflang validation",
                    run_hreflang_validation(&pool, crawl_id).await,
                ),
            ];
            for (pass, outcome) in passes {
                if let Err(e) = outcome {
                    tracing::warn!(error=%e, pass, "post-crawl pass failed");
                    if let Err(send_error) = tx
                        .send_async(CrawlEvent::Error {
                            url: root_url.clone(),
                            message: format!("{pass} failed: {e}"),
                        })
                        .await
                    {
                        tracing::warn!(error=%send_error, "failed to send pass failure event");
                    }
                }
            }

            // Sitemap orphans and robots-blocked URLs: recorded, not just
            // announced. Both used to be live-only events, so reopening a crawl
            // or exporting it lost them, and a site with 422 sitemap URLs
            // against 105 crawled lost about three hundred rows.
            {
                let blocked: std::collections::HashSet<String> =
                    std::mem::take(&mut *blocked_urls.lock().unwrap_or_else(|e| e.into_inner()))
                        .into_iter()
                        .collect();
                // Every URL already a row, so a URL listed by two sitemaps, or
                // both listed and blocked, is one row rather than two.
                let mut recorded = storage::load_page_urls(&pool, crawl_id)
                    .await
                    .unwrap_or_default();

                // Read the orphans before recording any: the query is "listed
                // in a sitemap and absent from `pages`", and we are about to
                // add them to `pages`.
                let orphans = storage::load_sitemap_orphans(&pool, crawl_id)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::warn!(error=%e, "failed to load sitemap orphans");
                        Vec::new()
                    });
                let mut orphan_count = 0usize;
                for orphan in &orphans {
                    if !recorded.insert(orphan.page_url.clone()) {
                        continue;
                    }
                    orphan_count += 1;
                    let record = PageRecord {
                        url: orphan.page_url.clone(),
                        sitemap_url: Some(orphan.sitemap_url.clone()),
                        sitemap_lastmod: orphan.lastmod.clone(),
                        in_sitemap: Some(true),
                        // A URL can be both listed and disallowed, and the one
                        // row should say so.
                        blocked_by_robots: blocked.contains(&orphan.page_url).then_some(true),
                        is_internal: is_same_domain(&root_url, &orphan.page_url),
                        ..Default::default()
                    };
                    if let Err(e) = storage::insert_page(&pool, crawl_id, &record).await {
                        tracing::warn!(error=%e, url=%record.url, "failed to persist sitemap orphan");
                    }
                    if let Err(e) = tx.send_async(CrawlEvent::Page(Box::new(record))).await {
                        tracing::warn!(error=%e, "failed to send sitemap orphan page event");
                    }
                }
                if orphan_count > 0 {
                    tracing::info!(count = orphan_count, "found sitemap orphan URLs");
                }

                let mut blocked_count = 0usize;
                for url in &blocked {
                    if !recorded.insert(url.clone()) {
                        continue;
                    }
                    blocked_count += 1;
                    let record = PageRecord {
                        url: url.clone(),
                        is_internal: is_same_domain(&root_url, url),
                        blocked_by_robots: Some(true),
                        ..Default::default()
                    };
                    if let Err(e) = storage::insert_page(&pool, crawl_id, &record).await {
                        tracing::warn!(error=%e, url=%record.url, "failed to persist blocked URL");
                    }
                    if let Err(e) = tx.send_async(CrawlEvent::Page(Box::new(record))).await {
                        tracing::warn!(error=%e, "failed to send blocked URL page event");
                    }
                }
                if blocked_count > 0 {
                    tracing::info!(count = blocked_count, "found robots.txt-blocked URLs");
                }
            }

            if let Err(e) = tx
                .send_async(CrawlEvent::Finished { crawl_id, total })
                .await
            {
                tracing::warn!(error=%e, "failed to send finished event");
            }
        };

        (cancel, fut)
    }
}

/// Requests every resource the crawl discovered (images, stylesheets, scripts
/// and links to other sites) once, and records each as a row of its own.
///
/// Runs after the page pump so it never competes with the crawl for the target
/// site's attention, and skips anything already recorded as a page: an internal
/// URL the crawler reached, or a resource Chrome already reported through the
/// Resource Timing API.
async fn check_discovered_resources(
    pool: &SqlitePool,
    crawl_id: i64,
    tx: &Sender<CrawlEvent>,
    config: &CrawlConfig,
    root_url: &str,
    discovered: Vec<(String, ResourceKind)>,
    cancel: &Arc<AtomicBool>,
) {
    if !config.check_resources || cancel.load(Ordering::Relaxed) {
        return;
    }
    let recorded = match storage::load_described_page_urls(pool, crawl_id).await {
        Ok(urls) => urls,
        Err(e) => {
            tracing::warn!(error=%e, "failed to read recorded URLs; skipping resource checks");
            return;
        }
    };
    // Images Chrome loaded but could not describe (a cross-origin CDN
    // without Timing-Allow-Origin reports no size) get a HEAD of their own,
    // since the Images tab and the 100 kB rule need the size. Fetches, XHR
    // calls and script chunks are left as Chrome reported them: their size
    // and headers say nothing about SEO, and on a catalogue with two API
    // calls per product that is a thousand requests for nothing.
    let mut discovered = discovered;
    match storage::load_undescribed_resources(pool, crawl_id).await {
        Ok(undescribed) => discovered.extend(
            undescribed
                .into_iter()
                .map(|(url, initiator)| (url, ResourceKind::from_initiator(&initiator)))
                .filter(|(_, kind)| *kind == ResourceKind::Image),
        ),
        Err(e) => tracing::warn!(error=%e, "failed to read undescribed resources"),
    }
    let planned = crate::crawl::resources::plan_checks(&discovered, &recorded);
    if planned.is_empty() {
        return;
    }
    let client = match build_ssr_client(config) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build resource client; skipping resource checks");
            return;
        }
    };
    tracing::info!(
        count = planned.len(),
        "status-checking discovered resources"
    );

    crate::crawl::resources::check_all(
        &client,
        planned,
        config.max_concurrent as usize,
        Duration::from_millis(config.delay_ms),
        cancel,
        |check| async move {
            let mut record = PageRecord {
                url: check.url.clone(),
                status: check.status,
                size_bytes: check.size_bytes,
                content_type: check.content_type.clone(),
                headers: check.headers.clone(),
                response_time: check.response_time,
                is_internal: is_same_domain(root_url, &check.url),
                is_resource: true,
                resource_initiator: Some(check.kind.initiator().to_string()),
                redirect_url: check.redirect_url.clone(),
                ..Default::default()
            };
            if record.redirect_url.is_some() {
                record.redirect_status = record
                    .status
                    .filter(|code| (300..400).contains(code))
                    .or(Some(301));
            }
            if let Some(error) = &check.error {
                tracing::debug!(url = %check.url, error = %error, "resource check failed");
            }
            record.compute_indexability();
            if let Err(e) = storage::record_resource_check(pool, crawl_id, &record).await {
                tracing::warn!(error=%e, url=%record.url, "failed to persist resource");
            }
            if let Err(e) = tx.send_async(CrawlEvent::Page(Box::new(record))).await {
                tracing::error!(error=%e, "failed to send resource event");
            }
        },
    )
    .await;
}

/// Waits out the configured delay between requests. Spider paces the page
/// crawl itself, but the passes that run after it make their own requests to
/// the same server, so they hold to the same pace rather than bursting.
async fn pace(config: &CrawlConfig) {
    if config.delay_ms > 0 {
        tokio::time::sleep(Duration::from_millis(config.delay_ms)).await;
    }
}

/// How many hops one redirect chain is followed for. Chains longer than this
/// are broken by any browser too, so the cap is a real limit rather than a
/// safety valve.
const MAX_REDIRECT_HOPS: usize = 10;

/// Walks every redirect to its destination, recording each hop as a row and
/// crawling the page at the end of it.
///
/// Two things spider leaves out. A 3xx it declined to follow arrives as
/// "something redirected, destination unknown", and the destination is
/// reachable no other way. And a chain it *did* follow arrives as a single row
/// pointing at the *final* URL, so `/` → `/sv/` → `/sv` is recorded as
/// `/` → `/sv` and the hop in between is not recorded at all — while a chain is
/// itself worth reporting, since every hop spends crawl budget and dilutes what
/// passes through it.
///
/// So each 3xx row is walked from the start with redirects disabled, reading
/// `Location` ourselves: every row ends up pointing at its own immediate
/// target, every intermediate hop becomes a row of its own, and the page the
/// chain ends at is fetched and analysed like any other.
async fn resolve_redirect_chains(
    pool: &SqlitePool,
    crawl_id: i64,
    tx: &Sender<CrawlEvent>,
    config: &CrawlConfig,
    root_url: &str,
    cancel: &Arc<AtomicBool>,
) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    let redirects = match storage::load_redirects(pool, crawl_id).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error=%e, "failed to read redirects");
            return;
        }
    };
    if redirects.is_empty() {
        return;
    }
    let client = match build_redirect_client(config) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build redirect client");
            return;
        }
    };
    let mut recorded = match storage::load_page_urls(pool, crawl_id).await {
        Ok(urls) => urls,
        Err(e) => {
            tracing::warn!(error=%e, "failed to read recorded URLs");
            return;
        }
    };

    for start in redirects {
        let mut url = start;

        for _ in 0..MAX_REDIRECT_HOPS {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            pace(config).await;
            let Some((status, target)) = fetch_redirect_target(&client, &url).await else {
                break;
            };
            // The row points at the hop it actually serves, which is not
            // necessarily where the chain ends.
            if let Err(e) =
                storage::set_redirect_target(pool, crawl_id, &url, &target, status).await
            {
                tracing::warn!(error=%e, url=%url, "failed to record redirect target");
            }
            tracing::info!(from = %url, to = %target, "followed a redirect");

            // Off-site targets are left to the resource pass, which
            // status-checks them without auditing them, and a target already
            // recorded ends the walk: that is either a URL the crawl reached
            // by another route or a loop closing on itself.
            if !is_same_domain(root_url, &target) || !recorded.insert(target.clone()) {
                break;
            }
            pace(config).await;
            let Some(record) = fetch_uncrawled_page(&client, &target, config, root_url).await
            else {
                break;
            };
            let next_status = record.status;
            if let Err(e) = storage::insert_page(pool, crawl_id, &record).await {
                tracing::warn!(error=%e, url=%record.url, "failed to persist redirect target");
            }
            if let Err(e) = tx.send_async(CrawlEvent::Page(Box::new(record))).await {
                tracing::error!(error=%e, "failed to send redirect target page event");
            }

            // Another 3xx is another hop; anything else is the end of the
            // chain, and it has just been recorded as the page it is.
            match next_status {
                Some(code) if (300..400).contains(&code) => url = target,
                _ => break,
            }
        }
    }
}

/// The canonical targets worth requesting: absolute, on this site, not the
/// declaring page itself, and not already a row of the crawl.
///
/// Split out from the pass so the decision is testable without a database or a
/// server. Comparison is normalised throughout, because a page that writes its
/// canonical without the trailing slash is pointing at itself, and a target
/// already recorded under a different spelling of the same URL is already
/// recorded.
fn plan_canonical_fetches(
    declared: &[(String, String)],
    recorded: &std::collections::HashSet<String>,
    root_url: &str,
) -> Vec<String> {
    let recorded_normalized: std::collections::HashSet<String> = recorded
        .iter()
        .map(|url| crate::crawl::url_norm::normalize_url(url).unwrap_or_else(|| url.clone()))
        .collect();
    let mut planned = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for (page_url, canonical) in declared {
        let Some(target) = crate::crawl::url_norm::resolve_url(page_url, canonical) else {
            continue;
        };
        if urls_equivalent(&target, page_url) || !is_same_domain(root_url, &target) {
            continue;
        }
        let normalized =
            crate::crawl::url_norm::normalize_url(&target).unwrap_or_else(|| target.clone());
        if recorded_normalized.contains(&normalized) || !seen.insert(normalized) {
            continue;
        }
        planned.push(target);
    }
    planned
}

/// Fetches the URL a document declares as its canonical, when the crawl never
/// reached that URL.
///
/// A canonical is the site's own statement of which URL is authoritative, and
/// nothing obliges anyone to link to it: `/Integritetspolicy` canonicalises to
/// `/integritetspolicy`, our content rules skip the former for being
/// canonicalised, and without this pass the latter is never fetched, so between
/// the two we audit neither.
///
/// Bounded by construction, which was the concern: the URL set is fixed before
/// the pass starts, each URL is fetched once, and **its own links are not
/// queued**. One hop, no frontier expansion, the same shape as
/// [`resolve_unknown_redirects`].
///
/// The client does not follow redirects, so a canonical pointing at a redirect
/// is recorded as the 3xx it is rather than silently resolved.
async fn fetch_declared_canonical_targets(
    pool: &SqlitePool,
    crawl_id: i64,
    tx: &Sender<CrawlEvent>,
    config: &CrawlConfig,
    root_url: &str,
    cancel: &Arc<AtomicBool>,
) {
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    let declared = match storage::load_declared_canonicals(pool, crawl_id).await {
        Ok(rows) => rows,
        Err(e) => {
            tracing::warn!(error=%e, "failed to read declared canonicals");
            return;
        }
    };
    if declared.is_empty() {
        return;
    }
    let recorded = match storage::load_page_urls(pool, crawl_id).await {
        Ok(urls) => urls,
        Err(e) => {
            tracing::warn!(error=%e, "failed to read recorded URLs");
            return;
        }
    };
    let planned = plan_canonical_fetches(&declared, &recorded, root_url);
    if planned.is_empty() {
        return;
    }
    let client = match build_redirect_client(config) {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build client for canonical targets");
            return;
        }
    };
    tracing::info!(
        count = planned.len(),
        "fetching uncrawled canonical targets"
    );

    for target in planned {
        if cancel.load(Ordering::Relaxed) {
            return;
        }
        pace(config).await;
        let Some(record) = fetch_uncrawled_page(&client, &target, config, root_url).await else {
            continue;
        };
        if let Err(e) = storage::insert_page(pool, crawl_id, &record).await {
            tracing::warn!(error=%e, url=%record.url, "failed to persist canonical target");
        }
        if let Err(e) = tx.send_async(CrawlEvent::Page(Box::new(record))).await {
            tracing::error!(error=%e, "failed to send canonical target page event");
        }
    }
}

/// A client that reports redirects instead of following them, so `Location` is
/// readable.
fn build_redirect_client(config: &CrawlConfig) -> Result<reqwest::Client, reqwest::Error> {
    let mut builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.timeout_seconds.max(1) as u64))
        .redirect(reqwest::redirect::Policy::none());
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

/// The status a URL answers with and the absolute URL it points at, from its
/// `Location` header.
///
/// The status comes back with the target because it is the status of *this*
/// hop: a row's stored status can be the 200 at the end of a chain spider
/// followed, and a hop should record the code it actually served.
///
/// `Location` is very often relative (`/se/bett`), which RFC 7231 allows and
/// which every browser resolves against the request URL, so we do too.
async fn fetch_redirect_target(client: &reqwest::Client, url: &str) -> Option<(u16, String)> {
    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(e) => {
            tracing::debug!(url = %url, error = %e, "redirect resolution request failed");
            return None;
        }
    };
    if !response.status().is_redirection() {
        return None;
    }
    let status = response.status().as_u16();
    let location = response
        .headers()
        .get(reqwest::header::LOCATION)?
        .to_str()
        .ok()?
        .trim()
        .to_string();
    let base = url::Url::parse(url).ok()?;
    let resolved = base.join(&location).ok()?;
    (resolved.as_str() != url).then(|| (status, resolved.to_string()))
}

/// The URL the crawl is actually scoped to, following a redirect the start URL
/// itself serves onto the same site under a different spelling.
///
/// `https://www.example.com` answering `301 -> https://example.com/` is the
/// ordinary case, and the whole crawl hangs on it: the frontier, the
/// internal/external split and the depth graph are all keyed on the root's
/// host, so crawling under the spelling the site redirects away from leaves
/// every page that follows belonging to "another" site, unanalysed and with no
/// link graph.
///
/// Only a hop onto another spelling of the same host is adopted. A redirect
/// within the host (`/` -> `/sv/`) is left alone so it is still recorded as the
/// redirect it is, and a redirect somewhere genuinely else is left alone
/// because that is a fact about the URL worth reporting rather than a spelling
/// to correct.
async fn resolve_start_url(root_url: &str, config: &CrawlConfig) -> String {
    if config.list_mode {
        return root_url.to_string();
    }
    let Ok(client) = build_redirect_client(config) else {
        return root_url.to_string();
    };

    let mut url = root_url.to_string();
    for _ in 0..MAX_REDIRECT_HOPS {
        let Some((_, target)) = fetch_redirect_target(&client, &url).await else {
            break;
        };
        if !is_another_spelling_of_the_same_host(&url, &target) {
            break;
        }
        url = target;
    }

    if url != root_url {
        tracing::info!(from = %root_url, to = %url, "start URL redirects; crawling the target");
    }
    url
}

/// Fetches and analyzes one URL as a page of its own, outside the crawl.
///
/// Used by the post-crawl passes that know an address the crawler never
/// requested: a redirect's target and a declared canonical. It reads the one
/// URL it is given and does nothing with the links on it.
async fn fetch_uncrawled_page(
    client: &reqwest::Client,
    url: &str,
    config: &CrawlConfig,
    root_url: &str,
) -> Option<PageRecord> {
    let started = std::time::Instant::now();
    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(e) => {
            tracing::debug!(url = %url, error = %e, "redirect target fetch failed");
            return None;
        }
    };
    let status = response.status().as_u16();
    let headers: Vec<(String, String)> = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or(value).trim().to_string());
    let body = response.text().await.ok()?;

    let mut record = PageRecord {
        url: url.to_string(),
        status: Some(status),
        size_bytes: body.len() as u64,
        response_time: started.elapsed(),
        content_type,
        headers,
        is_page: true,
        is_internal: is_same_domain(root_url, url),
        ..Default::default()
    };
    if (200..300).contains(&status) && !body.is_empty() {
        crate::crawl::analyzers::analyze_html(&mut record, &body, &config.content_selector);
    }
    record.compute_indexability();
    Some(record)
}

/// A page URL to the sitemap that lists it, the `<lastmod>` it claims and any
/// `xhtml:link` alternates on the entry.
type SitemapLookup =
    std::collections::HashMap<String, (String, Option<String>, Vec<(String, String)>)>;

pub fn channel() -> (Sender<CrawlEvent>, Receiver<CrawlEvent>) {
    flume::unbounded()
}

async fn run_near_duplicate_analysis(
    pool: &SqlitePool,
    crawl_id: i64,
    threshold: u8,
) -> Result<(), sqlx::Error> {
    let pages = storage::load_simhashes_for_crawl(pool, crawl_id).await?;

    if pages.len() < 2 {
        return Ok(());
    }

    let results = crate::crawl::similarity::find_near_duplicates(&pages, threshold);

    storage::update_near_duplicates(
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
}

/// Assigns each crawled URL its click depth from the start page: a breadth-first
/// walk of the link graph, with a redirect counted as a hop so a page reached
/// only through a 301 sits one level deeper than the URL that redirected to it.
/// URLs the walk can't reach (sitemap-only orphans, robots.txt-blocked URLs) are
/// left untouched rather than reported as depth 0, which would put them level
/// with the start page.
async fn run_crawl_depth_analysis(
    pool: &SqlitePool,
    crawl_id: i64,
    root_url: &str,
) -> Result<(), sqlx::Error> {
    let link_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT src_url, dst_url FROM links WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let page_rows = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT url, redirect_url FROM pages WHERE crawl_id = ?",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    // Key the graph on normalised URLs so a link written as
    // `https://site.com` reaches the page stored as `https://site.com/`.
    let mut edges: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let normalize =
        |url: &str| crate::crawl::url_norm::normalize_url(url).unwrap_or_else(|| url.to_string());

    for (src, dst) in &link_rows {
        edges
            .entry(normalize(src))
            .or_default()
            .push(normalize(dst));
    }
    for (url, redirect_url) in &page_rows {
        if let Some(target) = redirect_url.as_deref().filter(|t| !t.is_empty()) {
            edges
                .entry(normalize(url))
                .or_default()
                .push(normalize(target));
        }
    }

    // Several stored URLs can normalise to the same key (e.g. with and without a
    // fragment), so a key maps back to every page row it stands for.
    let mut urls_by_key: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (url, _) in &page_rows {
        urls_by_key
            .entry(normalize(url))
            .or_default()
            .push(url.clone());
    }

    let root_key = normalize(root_url);
    if !urls_by_key.contains_key(&root_key) {
        tracing::warn!(
            root_url,
            "start page not found among crawled pages; skipping depth"
        );
        return Ok(());
    }

    let mut depth_by_key: std::collections::HashMap<String, u32> = std::collections::HashMap::new();
    depth_by_key.insert(root_key.clone(), 0);
    let mut queue = std::collections::VecDeque::from([root_key]);

    while let Some(key) = queue.pop_front() {
        let depth = depth_by_key[&key];
        let Some(targets) = edges.get(&key) else {
            continue;
        };
        for target in targets {
            if let std::collections::hash_map::Entry::Vacant(slot) =
                depth_by_key.entry(target.clone())
            {
                slot.insert(depth + 1);
                queue.push_back(target.clone());
            }
        }
    }

    let depths: Vec<(String, u32)> = depth_by_key
        .iter()
        .filter_map(|(key, depth)| urls_by_key.get(key).map(|urls| (urls, *depth)))
        .flat_map(|(urls, depth)| urls.iter().map(move |url| (url.clone(), depth)))
        .collect();

    storage::update_crawl_depths(pool, crawl_id, &depths).await
}

/// How many times a rate-limited or failing fetch is retried before its
/// status is recorded.
const RATE_LIMIT_RETRIES: u8 = 2;

async fn run_pagerank_analysis(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    let link_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT src_url, dst_url FROM links WHERE crawl_id = ? AND kind = 'internal'",
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    if link_rows.is_empty() {
        return Ok(());
    }

    let results = compute_pagerank(&link_rows);

    storage::update_link_scores(pool, crawl_id, &results).await
}

/// Normalised PageRank (0..100) over internal links. Each page's score is
/// divided among the pages it links *to*; dangling pages spread theirs over
/// the whole graph.
pub(crate) fn compute_pagerank(link_rows: &[(String, String)]) -> Vec<(String, f32)> {
    let mut all_urls: Vec<String> = link_rows
        .iter()
        .flat_map(|(src, dst)| [src.clone(), dst.clone()])
        .collect();
    all_urls.sort();
    all_urls.dedup();

    let url_count = all_urls.len();
    if url_count == 0 {
        return Vec::new();
    }
    let url_index: std::collections::HashMap<&str, usize> = all_urls
        .iter()
        .enumerate()
        .map(|(i, url)| (url.as_str(), i))
        .collect();

    let mut outlinks: Vec<Vec<usize>> = vec![Vec::new(); url_count];
    for (src, dst) in link_rows {
        let (Some(&src_idx), Some(&dst_idx)) =
            (url_index.get(src.as_str()), url_index.get(dst.as_str()))
        else {
            continue;
        };
        outlinks[src_idx].push(dst_idx);
    }

    let damping = 0.85f32;
    let iterations = 30;
    let mut scores = vec![1.0f32 / url_count as f32; url_count];

    for _ in 0..iterations {
        let mut new_scores = vec![(1.0 - damping) / url_count as f32; url_count];
        for (node, targets) in outlinks.iter().enumerate() {
            if targets.is_empty() {
                let share = scores[node] * damping / url_count as f32;
                for new_score in &mut new_scores {
                    *new_score += share;
                }
            } else {
                let share = scores[node] * damping / targets.len() as f32;
                for &target in targets {
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

    all_urls.into_iter().zip(scores).collect()
}

/// What the crawl knows about one page, for hreflang validation.
pub(crate) struct HreflangPage {
    pub hreflang_tags: Vec<(String, String)>,
    pub canonical: Option<String>,
}

/// The hreflang defects of a single page, given every page the crawl reached
/// keyed by normalised URL.
///
/// `crawled` holds *every* page, not just the ones carrying hreflang tags,
/// because the difference between "the target has no return tag" and "we never
/// looked at the target" is the whole point: a target outside the crawl tells us
/// nothing, and asserting against it flagged 104 of 125 pages on a site whose
/// alternates live in language trees the crawl doesn't cover.
pub(crate) fn hreflang_issues_for_page(
    page_url: &str,
    info: &HreflangPage,
    crawled: &std::collections::HashMap<String, HreflangPage>,
) -> Vec<crate::crawl::event::HreflangIssue> {
    use crate::crawl::event::HreflangIssue;
    use crate::crawl::url_norm::{normalize_url, resolve_url};

    let lookup =
        |url: &str| crawled.get(&normalize_url(url).unwrap_or_else(|| url.trim().to_string()));

    let mut issues: Vec<HreflangIssue> = Vec::new();

    let has_x_default = info
        .hreflang_tags
        .iter()
        .any(|(lang, _)| lang == "x-default");
    if !has_x_default {
        issues.push(HreflangIssue::MissingXDefault);
    }

    // Every page in an hreflang cluster must list itself. A set that omits its
    // own page describes a group the page isn't part of, and search engines may
    // discard the whole cluster rather than guess.
    let has_self_reference = info
        .hreflang_tags
        .iter()
        .any(|(lang, url)| lang != "x-default" && urls_equivalent(url, page_url));
    if !has_self_reference {
        issues.push(HreflangIssue::MissingSelfReference);
    }

    for (lang, target_url) in &info.hreflang_tags {
        if lang != "x-default" && !is_valid_bcp47(lang) {
            issues.push(HreflangIssue::InvalidLanguageCode { code: lang.clone() });
        }

        // Anything below needs the target's own markup, so a target the crawl
        // never reached is skipped rather than reported against.
        let Some(target) = lookup(target_url) else {
            continue;
        };

        // The return tag only has to point back at this page. Its language is
        // the target's opinion of *this* page, not `lang` (which is this page's
        // opinion of the target), so requiring the two to match reported a
        // missing return tag for every correctly configured cluster.
        let return_tag_exists = target
            .hreflang_tags
            .iter()
            .any(|(_, return_url)| urls_equivalent(return_url, page_url));
        if !return_tag_exists {
            issues.push(HreflangIssue::MissingReturnTag {
                lang: lang.clone(),
                target_url: target_url.clone(),
            });
        }

        // Resolve and normalise before comparing: a target whose canonical is
        // relative, or written without the trailing slash, is pointing at
        // itself, not somewhere else.
        if let Some(canonical) = target.canonical.as_deref().filter(|c| !c.trim().is_empty())
            && !urls_equivalent(
                &resolve_url(target_url, canonical).unwrap_or_else(|| canonical.to_string()),
                target_url,
            )
        {
            issues.push(HreflangIssue::NonCanonicalUrl {
                hreflang_url: target_url.clone(),
            });
        }
    }

    issues
}

async fn run_hreflang_validation(pool: &SqlitePool, crawl_id: i64) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        r#"SELECT url, hreflang_tags_json, canonical FROM pages WHERE crawl_id = ?"#,
    )
    .bind(crawl_id)
    .fetch_all(pool)
    .await?;

    let mut crawled: std::collections::HashMap<String, HreflangPage> =
        std::collections::HashMap::with_capacity(rows.len());
    let mut tagged: Vec<(String, HreflangPage)> = Vec::new();
    for (url, hreflang_json, canonical) in &rows {
        let tags: Vec<(String, String)> = hreflang_json
            .as_deref()
            .and_then(|j| serde_json::from_str(j).ok())
            .unwrap_or_default();
        if !tags.is_empty() {
            tagged.push((
                url.clone(),
                HreflangPage {
                    hreflang_tags: tags.clone(),
                    canonical: canonical.clone(),
                },
            ));
        }
        let key = crate::crawl::url_norm::normalize_url(url).unwrap_or_else(|| url.clone());
        crawled.insert(
            key,
            HreflangPage {
                hreflang_tags: tags,
                canonical: canonical.clone(),
            },
        );
    }

    if tagged.is_empty() {
        return Ok(());
    }

    let all_issues: Vec<(String, Vec<crate::crawl::event::HreflangIssue>)> = tagged
        .iter()
        .filter_map(|(page_url, info)| {
            let issues = hreflang_issues_for_page(page_url, info, &crawled);
            (!issues.is_empty()).then(|| (page_url.clone(), issues))
        })
        .collect();

    storage::update_hreflang_issues(pool, crawl_id, &all_issues).await
}

/// ISO 639-1 two-letter language codes.
const ISO_639_1: &[&str] = &[
    "aa", "ab", "ae", "af", "ak", "am", "an", "ar", "as", "av", "ay", "az", "ba", "be", "bg", "bh",
    "bi", "bm", "bn", "bo", "br", "bs", "ca", "ce", "ch", "co", "cr", "cs", "cu", "cv", "cy", "da",
    "de", "dv", "dz", "ee", "el", "en", "eo", "es", "et", "eu", "fa", "ff", "fi", "fj", "fo", "fr",
    "fy", "ga", "gd", "gl", "gn", "gu", "gv", "ha", "he", "hi", "ho", "hr", "ht", "hu", "hy", "hz",
    "ia", "id", "ie", "ig", "ii", "ik", "io", "is", "it", "iu", "ja", "jv", "ka", "kg", "ki", "kj",
    "kk", "kl", "km", "kn", "ko", "kr", "ks", "ku", "kv", "kw", "ky", "la", "lb", "lg", "li", "ln",
    "lo", "lt", "lu", "lv", "mg", "mh", "mi", "mk", "ml", "mn", "mr", "ms", "mt", "my", "na", "nb",
    "nd", "ne", "ng", "nl", "nn", "no", "nr", "nv", "ny", "oc", "oj", "om", "or", "os", "pa", "pi",
    "pl", "ps", "pt", "qu", "rm", "rn", "ro", "ru", "rw", "sa", "sc", "sd", "se", "sg", "si", "sk",
    "sl", "sm", "sn", "so", "sq", "sr", "ss", "st", "su", "sv", "sw", "ta", "te", "tg", "th", "ti",
    "tk", "tl", "tn", "to", "tr", "ts", "tt", "tw", "ty", "ug", "uk", "ur", "uz", "ve", "vi", "vo",
    "wa", "wo", "xh", "yi", "yo", "za", "zh", "zu",
];

/// Three-letter codes Google documents or that appear in the wild for
/// languages without a two-letter code (ISO 639-2/3).
const ISO_639_3_COMMON: &[&str] = &[
    "ast", "ceb", "chr", "fil", "gsw", "haw", "hmn", "ilo", "kok", "lus", "mai", "nso", "pap",
    "sco", "syr", "tlh", "yue", "zxx",
];

/// ISO 3166-1 alpha-2 region codes, plus the user-assigned `UK` and `EU` that
/// Google explicitly accepts.
const ISO_3166_1: &[&str] = &[
    "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX", "AZ",
    "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ", "BR", "BS",
    "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK", "CL", "CM", "CN",
    "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM", "DO", "DZ", "EC", "EE",
    "EG", "EH", "ER", "ES", "ET", "EU", "FI", "FJ", "FK", "FM", "FO", "FR", "GA", "GB", "GD", "GE",
    "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS", "GT", "GU", "GW", "GY", "HK",
    "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN", "IO", "IQ", "IR", "IS", "IT", "JE",
    "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN", "KP", "KR", "KW", "KY", "KZ", "LA", "LB",
    "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV", "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH",
    "MK", "ML", "MM", "MN", "MO", "MP", "MQ", "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ",
    "NA", "NC", "NE", "NF", "NG", "NI", "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF",
    "PG", "PH", "PK", "PL", "PM", "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU",
    "RW", "SA", "SB", "SC", "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR",
    "SS", "ST", "SV", "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN",
    "TO", "TR", "TT", "TV", "TW", "TZ", "UA", "UG", "UK", "UM", "US", "UY", "UZ", "VA", "VC", "VE",
    "VG", "VI", "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
];

/// Is `code` an hreflang value Google will act on: `x-default`, or an ISO 639
/// language optionally followed by an ISO 15924 script and/or an ISO 3166-1
/// region (or UN M.49 numeric area). Matching is case-insensitive, as the
/// attribute is.
///
/// The language table matters more than the syntax: `dk`, `uk` and `be` are
/// all well-formed, and all three are countries used as languages on real
/// shops, which Search Console reports as "unknown language code".
fn is_valid_bcp47(code: &str) -> bool {
    let code = code.trim();
    if code.eq_ignore_ascii_case("x-default") {
        return true;
    }
    let mut parts = code.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    let primary = primary.to_ascii_lowercase();
    let known_language = match primary.len() {
        2 => ISO_639_1.contains(&primary.as_str()),
        3 => ISO_639_3_COMMON.contains(&primary.as_str()),
        _ => false,
    };
    if !known_language {
        return false;
    }

    let mut seen_script = false;
    let mut seen_region = false;
    for part in parts {
        let is_script = part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic());
        let is_region = (part.len() == 2
            && ISO_3166_1.contains(&part.to_ascii_uppercase().as_str()))
            || (part.len() == 3 && part.chars().all(|c| c.is_ascii_digit()));
        match (is_script, is_region) {
            (true, _) if !seen_script && !seen_region => seen_script = true,
            (_, true) if !seen_region => seen_region = true,
            _ => return false,
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
        var lcpEntries = window.__sr_lcp_entries || [];
        if (lcpEntries.length) {
            lcp = Math.round(lcpEntries[lcpEntries.length - 1].startTime);
        }
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
    match (root_parsed.host_str(), url_parsed.host_str()) {
        (Some(root_host), Some(host)) => without_www(root_host) == without_www(host),
        (root_host, host) => root_host == host,
    }
}

/// `www.example.com` and `example.com` are one site: one of the two is the
/// canonical host and redirects to the other, and a site linking to itself
/// under both spellings is linking to itself. Treating them as different hosts
/// marks every page of a crawl started at the non-canonical spelling external,
/// which skips the analysis that makes the row worth having.
///
/// Only `www` is folded away. Any other subdomain is a separate site unless the
/// crawl asked for subdomains.
fn without_www(host: &str) -> &str {
    host.strip_prefix("www.").unwrap_or(host)
}

/// True when two URLs name the same site under different hosts: `www` against
/// the bare domain, either direction. A redirect between them is the site
/// picking its canonical spelling, which is the crawl's cue to follow rather
/// than to record and stop.
fn is_another_spelling_of_the_same_host(from: &str, to: &str) -> bool {
    let (Ok(from), Ok(to)) = (url::Url::parse(from), url::Url::parse(to)) else {
        return false;
    };
    let (Some(from_host), Some(to_host)) = (from.host_str(), to.host_str()) else {
        return false;
    };
    from_host != to_host && without_www(from_host) == without_www(to_host)
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

/// The server's HTML for the page Chrome is showing, taken from Chrome rather
/// than requested again.
///
/// `Page.getResourceContent` is what the DevTools Sources panel reads: the
/// document's response body as it arrived, before any script ran, out of the
/// frame's resource tree. That is exactly what the SSR diff wants, and it costs
/// no request, so the fetch is only the fallback for when Chrome no longer has
/// the body (evicted, or a document it declines to hand back as text).
///
/// Must be called while the tab is still open — `close_page()` takes the frame
/// with it.
async fn raw_html_from_chrome(page: &spider::page::Page, url: &str) -> Option<String> {
    let chrome_page = page.get_chrome_page()?;
    let frame_id = match chrome_page.mainframe().await {
        Ok(Some(frame_id)) => frame_id,
        Ok(None) => {
            tracing::debug!(url = %url, "no main frame; falling back to the SSR fetch");
            return None;
        }
        Err(e) => {
            tracing::debug!(url = %url, error=%e, "reading the main frame failed");
            return None;
        }
    };

    let params = spider::chromiumoxide::cdp::browser_protocol::page::GetResourceContentParams::new(
        frame_id, url,
    );
    match chrome_page.execute(params).await {
        Ok(response) => {
            // Base64 is how Chrome hands back what it does not consider text.
            // Decoding it would only feed the HTML analyzers bytes they cannot
            // read, so let the fetch answer for those documents instead.
            if response.result.base64_encoded {
                tracing::debug!(url = %url, "document returned base64-encoded; using the SSR fetch");
                return None;
            }
            if response.result.content.is_empty() {
                return None;
            }
            Some(response.result.content)
        }
        Err(e) => {
            tracing::debug!(url = %url, error=%e, "getResourceContent failed; using the SSR fetch");
            None
        }
    }
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
    // This request asked for the document itself, so these are the document's
    // headers. A row reaches here with none when the ones Chrome reported were
    // another request's and were dropped, which is the only case worth filling
    // in: a successful response only, so a fetch the server refuses (a bot
    // block answering 403) does not describe the page instead.
    if record.headers.is_empty() && response.status().is_success() {
        record.headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|value| (name.to_string(), value.to_string()))
            })
            .collect();
    }
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
            // No fallback to FCP: largest-contentful-paint is observer-only
            // (getEntriesByType never returns it) and a page that was hidden
            // during load has none. A missing LCP must read as missing.
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

/// axe-core 4.10.2 (MPL-2.0), byte-identical to the npm release. Bundled
/// rather than fetched per crawl: it is injected into every page the crawl
/// renders, and code run that way must not depend on a CDN being honest and
/// reachable at crawl time.
const AXE_JS: &str = include_str!("../../assets/js/axe.min.js");

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

#[cfg(test)]
mod pagerank_tests {
    use super::compute_pagerank;

    fn link(src: &str, dst: &str) -> (String, String) {
        (src.to_string(), dst.to_string())
    }

    fn score_of(results: &[(String, f32)], url: &str) -> f32 {
        results
            .iter()
            .find(|(u, _)| u == url)
            .map(|(_, s)| *s)
            .unwrap_or(f32::NAN)
    }

    /// Score must flow along links, not against them: the page everyone links
    /// to is the hub, the page nobody links to is the weakest.
    #[test]
    fn hub_outranks_leaf_pages() {
        let rows = vec![
            link("/a", "/home"),
            link("/b", "/home"),
            link("/c", "/home"),
            link("/home", "/a"),
            link("/orphan", "/home"),
        ];
        let results = compute_pagerank(&rows);
        assert_eq!(score_of(&results, "/home"), 100.0);
        assert!(score_of(&results, "/a") > score_of(&results, "/b"));
        assert!((score_of(&results, "/b") - score_of(&results, "/orphan")).abs() < 1e-3);
        assert!(score_of(&results, "/b") > 0.0);
    }

    #[test]
    fn empty_input_has_no_scores() {
        assert!(compute_pagerank(&[]).is_empty());
    }
}

#[cfg(test)]
mod host_spelling_tests {
    use super::*;

    /// The case from a real crawl: `https://www.mindgear.se` answers 301 to
    /// `https://mindgear.se/`, and under exact host comparison every page found
    /// afterwards belonged to "another site", so nothing was analysed.
    #[test]
    fn www_and_the_bare_domain_are_one_site() {
        assert!(is_same_domain(
            "https://www.mindgear.se",
            "https://mindgear.se/kontakt/"
        ));
        assert!(is_same_domain(
            "https://mindgear.se",
            "https://www.mindgear.se/kontakt/"
        ));
    }

    #[test]
    fn another_subdomain_is_another_site() {
        assert!(!is_same_domain(
            "https://mindgear.se",
            "https://shop.mindgear.se/"
        ));
        assert!(!is_same_domain(
            "https://www.mindgear.se",
            "https://shop.mindgear.se/"
        ));
        // A domain that merely ends the same is not the same site.
        assert!(!is_same_domain(
            "https://mindgear.se",
            "https://notmindgear.se/"
        ));
    }

    #[test]
    fn a_hop_between_spellings_of_the_host_is_followed() {
        assert!(is_another_spelling_of_the_same_host(
            "https://www.mindgear.se",
            "https://mindgear.se/"
        ));
        assert!(is_another_spelling_of_the_same_host(
            "http://mindgear.se",
            "https://www.mindgear.se/"
        ));
    }

    /// A redirect inside the host is the crawl's to record, not to be scoped
    /// by: adopting it would drop the row that says `/` redirects.
    #[test]
    fn a_hop_within_the_host_is_left_alone() {
        assert!(!is_another_spelling_of_the_same_host(
            "https://mindgear.se/",
            "https://mindgear.se/sv/"
        ));
        assert!(!is_another_spelling_of_the_same_host(
            "http://mindgear.se/",
            "https://mindgear.se/"
        ));
    }

    #[test]
    fn a_hop_to_another_site_is_left_alone() {
        assert!(!is_another_spelling_of_the_same_host(
            "https://mindgear.se/",
            "https://example.com/"
        ));
        assert!(!is_another_spelling_of_the_same_host(
            "https://mindgear.se/",
            "https://shop.mindgear.se/"
        ));
    }
}

#[cfg(test)]
mod hreflang_tests {
    use super::*;
    use crate::crawl::event::HreflangIssue;
    use std::collections::HashMap;

    fn page(tags: &[(&str, &str)]) -> HreflangPage {
        HreflangPage {
            hreflang_tags: tags
                .iter()
                .map(|(lang, url)| ((*lang).to_string(), (*url).to_string()))
                .collect(),
            canonical: None,
        }
    }

    fn crawl(pages: &[(&str, HreflangPage)]) -> HashMap<String, HreflangPage> {
        pages
            .iter()
            .map(|(url, info)| {
                (
                    crate::crawl::url_norm::normalize_url(url).unwrap_or_else(|| url.to_string()),
                    HreflangPage {
                        hreflang_tags: info.hreflang_tags.clone(),
                        canonical: info.canonical.clone(),
                    },
                )
            })
            .collect()
    }

    fn missing_return_tags(issues: &[HreflangIssue]) -> Vec<&str> {
        issues
            .iter()
            .filter_map(|issue| match issue {
                HreflangIssue::MissingReturnTag { target_url, .. } => Some(target_url.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_target_outside_the_crawl_is_unknown_not_missing() {
        // The live false positive: a Swedish crawl whose alternates point into
        // /de/ and /fr/ trees it never visits flagged 104 of 125 pages.
        let a = page(&[
            ("sv", "https://a.test/se/x"),
            ("de", "https://a.test/de/x"),
            ("fr", "https://a.test/fr/x"),
        ]);
        let crawled = crawl(&[(
            "https://a.test/se/x",
            page(&[("sv", "https://a.test/se/x")]),
        )]);
        let issues = hreflang_issues_for_page("https://a.test/se/x", &a, &crawled);
        assert!(
            missing_return_tags(&issues).is_empty(),
            "got {:?}",
            missing_return_tags(&issues)
        );
    }

    #[test]
    fn a_crawled_target_that_does_not_link_back_is_still_reported() {
        let a = page(&[("sv", "https://a.test/se/x"), ("de", "https://a.test/de/x")]);
        let crawled = crawl(&[
            (
                "https://a.test/se/x",
                page(&[("sv", "https://a.test/se/x")]),
            ),
            // The German page lists only itself, never the Swedish one.
            (
                "https://a.test/de/x",
                page(&[("de", "https://a.test/de/x")]),
            ),
        ]);
        let issues = hreflang_issues_for_page("https://a.test/se/x", &a, &crawled);
        assert_eq!(missing_return_tags(&issues), vec!["https://a.test/de/x"]);
    }

    #[test]
    fn a_crawled_target_with_no_hreflang_at_all_is_reported() {
        // Reached, parsed, and carries no alternates: that is a real missing
        // return tag rather than an unknown.
        let a = page(&[("sv", "https://a.test/se/x"), ("de", "https://a.test/de/x")]);
        let crawled = crawl(&[
            (
                "https://a.test/se/x",
                page(&[("sv", "https://a.test/se/x")]),
            ),
            ("https://a.test/de/x", page(&[])),
        ]);
        let issues = hreflang_issues_for_page("https://a.test/se/x", &a, &crawled);
        assert_eq!(missing_return_tags(&issues), vec!["https://a.test/de/x"]);
    }

    #[test]
    fn the_return_tag_comparison_normalises_urls() {
        // The target links back without the trailing slash.
        let a = page(&[("sv", "https://a.test/"), ("de", "https://a.test/de/")]);
        let crawled = crawl(&[
            ("https://a.test/", page(&[("sv", "https://a.test/")])),
            (
                "https://a.test/de/",
                page(&[("sv", "https://a.test"), ("de", "https://a.test/de/")]),
            ),
        ]);
        let issues = hreflang_issues_for_page("https://a.test/", &a, &crawled);
        assert!(
            missing_return_tags(&issues).is_empty(),
            "got {:?}",
            missing_return_tags(&issues)
        );
    }

    #[test]
    fn a_regional_tag_is_satisfied_by_its_base_language() {
        let a = page(&[("sv-SE", "https://a.test/"), ("de-DE", "https://a.test/de")]);
        let crawled = crawl(&[
            ("https://a.test/", page(&[("sv-SE", "https://a.test/")])),
            ("https://a.test/de", page(&[("de", "https://a.test/")])),
        ]);
        let issues = hreflang_issues_for_page("https://a.test/", &a, &crawled);
        assert!(
            missing_return_tags(&issues).is_empty(),
            "got {:?}",
            missing_return_tags(&issues)
        );
    }

    #[test]
    fn hreflang_codes_are_checked_against_real_languages_and_regions() {
        for valid in [
            "en",
            "EN",
            "en-GB",
            "en-gb",
            "EN-US",
            "sv-SE",
            "zh-Hant",
            "zh-Hant-TW",
            "es-419",
            "fil",
            "x-default",
            "X-Default",
            "en-UK",
            "de-EU",
        ] {
            assert!(is_valid_bcp47(valid), "{valid} should be valid");
        }
        for invalid in [
            "dk",
            "be-BE-BE",
            "se-SE-Latn",
            "uk-ua-extra",
            "",
            "en-",
            "xx",
            "de-XX",
            "eng-US",
            "english",
            "en_US",
            "123",
        ] {
            assert!(!is_valid_bcp47(invalid), "{invalid} should be invalid");
        }
    }

    #[test]
    fn the_other_three_rules_still_fire() {
        let mut a = page(&[
            ("sv", "https://a.test/se/x"),
            ("invalid", "https://a.test/de/x"),
        ]);
        a.canonical = None;
        let mut target = page(&[("de", "https://a.test/de/x")]);
        target.canonical = Some("https://a.test/de/other".into());
        let crawled = crawl(&[("https://a.test/de/x", target)]);
        let issues = hreflang_issues_for_page("https://a.test/other", &a, &crawled);

        assert!(issues.contains(&HreflangIssue::MissingXDefault));
        assert!(issues.contains(&HreflangIssue::MissingSelfReference));
        assert!(issues.contains(&HreflangIssue::InvalidLanguageCode {
            code: "invalid".into()
        }));
        assert!(issues.contains(&HreflangIssue::NonCanonicalUrl {
            hreflang_url: "https://a.test/de/x".into()
        }));
    }

    #[test]
    fn a_self_referencing_target_canonical_is_not_a_defect() {
        let a = page(&[("sv", "https://a.test/"), ("de", "https://a.test/de")]);
        let mut target = page(&[("sv", "https://a.test/"), ("de", "https://a.test/de")]);
        // Written relative, and resolving to the target itself.
        target.canonical = Some("/de".into());
        let crawled = crawl(&[("https://a.test/de", target)]);
        let issues = hreflang_issues_for_page("https://a.test/", &a, &crawled);
        assert!(
            !issues
                .iter()
                .any(|i| matches!(i, HreflangIssue::NonCanonicalUrl { .. })),
            "got {issues:?}"
        );
    }
}

#[cfg(test)]
mod out_of_band_fetch_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// A server that answers `/moved` with a 301 whose `Location` is relative,
    /// which is what most servers send, and serves the target as a real page.
    /// The crawler's own fixtures cannot exercise this: spider refuses to
    /// follow a redirect to loopback, so it never reports one there.
    fn spawn_redirect_server() -> (u16, std::sync::Arc<AtomicBool>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        listener.set_nonblocking(true).expect("nonblocking");
        let port = listener.local_addr().expect("addr").port();
        let stop = std::sync::Arc::new(AtomicBool::new(false));

        std::thread::spawn({
            let stop = stop.clone();
            move || {
                while !stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let mut buf = [0u8; 2048];
                            let read = stream.read(&mut buf).unwrap_or(0);
                            let request = String::from_utf8_lossy(&buf[..read]);
                            let path = request
                                .lines()
                                .next()
                                .and_then(|line| line.split_whitespace().nth(1))
                                .unwrap_or("/")
                                .to_string();
                            let response = if path == "/chain1" {
                                "HTTP/1.1 301 Moved Permanently\r\nLocation: /chain2\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                            } else if path == "/chain2" {
                                "HTTP/1.1 301 Moved Permanently\r\nLocation: /target\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                            } else if path == "/moved" {
                                "HTTP/1.1 301 Moved Permanently\r\nLocation: /target?a=1&b=2\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                            } else if path.starts_with("/target") {
                                let body = "<!doctype html><html><head><title>Target Page Title</title></head>\
                                            <body><h1>Target Heading</h1><p>one two three four five</p></body></html>";
                                format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                                    body.len()
                                )
                            } else {
                                "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                            };
                            let _ = stream.write_all(response.as_bytes());
                            let _ = stream.flush();
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => break,
                    }
                }
            }
        });
        (port, stop)
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime")
    }

    fn config() -> CrawlConfig {
        CrawlConfig {
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
        }
    }

    #[test]
    fn a_relative_location_resolves_against_the_redirects_own_url() {
        let (port, stop) = spawn_redirect_server();
        let base = format!("http://127.0.0.1:{port}");
        let rt = runtime();

        let target = rt.block_on(async {
            let client = build_redirect_client(&config()).expect("client");
            fetch_redirect_target(&client, &format!("{base}/moved")).await
        });

        stop.store(true, Ordering::Relaxed);
        assert_eq!(
            target,
            Some((301, format!("{base}/target?a=1&b=2"))),
            "a relative Location, query string and all, resolves against the request URL"
        );
    }

    #[test]
    fn a_redirect_target_is_fetched_and_analyzed_as_a_page() {
        let (port, stop) = spawn_redirect_server();
        let base = format!("http://127.0.0.1:{port}");
        let rt = runtime();

        let record = rt.block_on(async {
            let client = build_redirect_client(&config()).expect("client");
            fetch_uncrawled_page(&client, &format!("{base}/target"), &config(), &base).await
        });

        stop.store(true, Ordering::Relaxed);
        let record = record.expect("the target should be fetched");
        assert_eq!(record.status, Some(200));
        assert!(record.is_page, "the target is a document of its own");
        assert!(record.is_internal);
        assert_eq!(record.title.as_deref(), Some("Target Page Title"));
        assert_eq!(record.h1.as_deref(), Some("Target Heading"));
        // The heading counts too: the word count is over the whole body.
        assert_eq!(record.word_count, Some(7));
        assert_eq!(record.indexability.as_deref(), Some("Indexable"));
    }

    #[test]
    fn a_url_that_does_not_redirect_yields_no_target() {
        let (port, stop) = spawn_redirect_server();
        let base = format!("http://127.0.0.1:{port}");
        let rt = runtime();

        let target = rt.block_on(async {
            let client = build_redirect_client(&config()).expect("client");
            fetch_redirect_target(&client, &format!("{base}/target")).await
        });

        stop.store(true, Ordering::Relaxed);
        assert_eq!(target, None);
    }

    #[test]
    fn the_pass_fetches_a_canonical_target_and_records_it_as_a_page() {
        let (port, stop) = spawn_redirect_server();
        let base = format!("http://127.0.0.1:{port}");
        let rt = runtime();

        let pages = rt.block_on(async {
            let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").expect("pool");
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
            )
            .execute(&pool)
            .await
            .expect("migrations table");
            storage::run_migrations(&pool).await.expect("migrations");
            let crawl_id = storage::create_crawl(&pool, &base, "http", &config())
                .await
                .expect("crawl");

            // The shape this pass exists for: a crawled document that declares
            // a canonical nothing links to.
            let source = PageRecord {
                url: format!("{base}/source"),
                status: Some(200),
                canonical: Some("/target".to_string()),
                is_page: true,
                is_internal: true,
                ..Default::default()
            };
            storage::insert_page(&pool, crawl_id, &source)
                .await
                .expect("insert source");

            let (tx, rx) = channel();
            fetch_declared_canonical_targets(
                &pool,
                crawl_id,
                &tx,
                &config(),
                &base,
                &Arc::new(AtomicBool::new(false)),
            )
            .await;
            drop(tx);

            let emitted: Vec<String> = rx
                .drain()
                .filter_map(|event| match event {
                    CrawlEvent::Page(record) => Some(record.url.clone()),
                    _ => None,
                })
                .collect();
            let stored = storage::load_pages_for_crawl(&pool, crawl_id, &base)
                .await
                .expect("load");
            (emitted, stored)
        });

        stop.store(true, Ordering::Relaxed);
        let (emitted, stored) = pages;
        assert_eq!(
            emitted,
            vec![format!("{base}/target")],
            "the target reaches the live UI as well as the database"
        );
        let target = stored
            .iter()
            .find(|record| record.url == format!("{base}/target"))
            .expect("the canonical target should be recorded");
        assert_eq!(target.status, Some(200));
        assert_eq!(target.title.as_deref(), Some("Target Page Title"));
        assert!(
            target.is_page,
            "the authoritative URL is audited like any other document"
        );
    }

    /// spider reports the *final* URL of a chain it followed, so the hops in
    /// between are missing entirely and the first row claims to point two hops
    /// away. Each row should point at the hop it serves, and each hop should be
    /// a row.
    #[test]
    fn every_hop_of_a_redirect_chain_becomes_a_row() {
        let (port, stop) = spawn_redirect_server();
        let base = format!("http://127.0.0.1:{port}");
        let rt = runtime();

        let stored = rt.block_on(async {
            let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").expect("pool");
            sqlx::query(
                "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
            )
            .execute(&pool)
            .await
            .expect("migrations table");
            storage::run_migrations(&pool).await.expect("migrations");
            let crawl_id = storage::create_crawl(&pool, &base, "http", &config())
                .await
                .expect("crawl");

            // What the crawler leaves behind: the head of the chain, already
            // carrying the destination two hops away.
            // The shape that hid the middle hop: spider followed the chain,
            // so the row carries the 200 from the end of it and a destination
            // two hops away.
            let head = PageRecord {
                url: format!("{base}/chain1"),
                status: Some(200),
                redirect_url: Some(format!("{base}/target")),
                redirect_status: Some(301),
                is_page: true,
                is_internal: true,
                ..Default::default()
            };
            storage::insert_page(&pool, crawl_id, &head)
                .await
                .expect("insert head");

            let (tx, _rx) = channel();
            resolve_redirect_chains(
                &pool,
                crawl_id,
                &tx,
                &config(),
                &base,
                &Arc::new(AtomicBool::new(false)),
            )
            .await;

            storage::load_pages_for_crawl(&pool, crawl_id, &base)
                .await
                .expect("load")
        });

        stop.store(true, Ordering::Relaxed);
        let find = |url: String| {
            stored
                .iter()
                .find(|record| record.url == url)
                .unwrap_or_else(|| panic!("{url} is missing"))
        };

        let first = find(format!("{base}/chain1"));
        assert_eq!(
            first.redirect_url.as_deref(),
            Some(format!("{base}/chain2").as_str()),
            "a row points at the hop it serves, not at the end of the chain"
        );
        assert_eq!(
            first.redirect_status,
            Some(301),
            "the code recorded is the one this hop served"
        );
        let second = find(format!("{base}/chain2"));
        assert_eq!(second.status, Some(301));
        assert_eq!(
            second.redirect_url.as_deref(),
            Some(format!("{base}/target").as_str())
        );
        let end = find(format!("{base}/target"));
        assert_eq!(end.status, Some(200));
        assert_eq!(end.title.as_deref(), Some("Target Page Title"));
    }

    fn recorded(urls: &[&str]) -> std::collections::HashSet<String> {
        urls.iter().map(|url| url.to_string()).collect()
    }

    #[test]
    fn a_canonical_pointing_at_an_uncrawled_url_is_planned() {
        let declared = vec![(
            "https://example.com/Policy".to_string(),
            "https://example.com/policy".to_string(),
        )];
        assert_eq!(
            plan_canonical_fetches(
                &declared,
                &recorded(&["https://example.com/Policy"]),
                "https://example.com"
            ),
            vec!["https://example.com/policy".to_string()]
        );
    }

    #[test]
    fn a_self_referencing_canonical_is_never_fetched() {
        // Both spellings of self-reference: the identical URL, and the one
        // written without the trailing slash that made the home page look
        // canonicalised elsewhere.
        let declared = vec![
            (
                "https://example.com/a".to_string(),
                "https://example.com/a".to_string(),
            ),
            (
                "https://example.com/".to_string(),
                "https://example.com".to_string(),
            ),
            ("https://example.com/b".to_string(), "/b".to_string()),
        ];
        assert!(
            plan_canonical_fetches(&declared, &recorded(&[]), "https://example.com").is_empty()
        );
    }

    #[test]
    fn a_canonical_already_recorded_is_not_fetched_again() {
        let declared = vec![(
            "https://example.com/a".to_string(),
            "https://example.com/b".to_string(),
        )];
        // Recorded under a spelling that differs only where the URL standard
        // says it may: the default port and the fragment.
        assert!(
            plan_canonical_fetches(
                &declared,
                &recorded(&["https://example.com:443/b#top"]),
                "https://example.com"
            )
            .is_empty()
        );
    }

    #[test]
    fn a_canonical_off_this_site_is_left_to_the_resource_pass() {
        let declared = vec![(
            "https://example.com/a".to_string(),
            "https://other.invalid/a".to_string(),
        )];
        assert!(
            plan_canonical_fetches(&declared, &recorded(&[]), "https://example.com").is_empty()
        );
    }

    #[test]
    fn seven_pages_canonicalising_to_one_url_fetch_it_once() {
        let declared: Vec<(String, String)> = (0..7)
            .map(|i| {
                (
                    format!("https://example.com/p{i}"),
                    "https://example.com/one".to_string(),
                )
            })
            .collect();
        assert_eq!(
            plan_canonical_fetches(&declared, &recorded(&[]), "https://example.com").len(),
            1
        );
    }
}
