mod common;

use common::*;
use std::time::Duration;

#[test]
fn test_http_crawl() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site(&root_url);

    server.kill();

    assert!(!pages.is_empty(), "crawl should discover pages");

    eprintln!(
        "HTTP crawl found {} pages: {:?}",
        pages.len(),
        page_paths(&pages)
    );

    // Home
    let home = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home page should be crawled");
    assert_eq!(home.status, Some(200));
    assert_eq!(home.indexability.as_deref(), Some("Indexable"));
    assert_eq!(home.title.as_deref(), Some("Test Site Home"));
    assert_eq!(home.h1.as_deref(), Some("Test Site Home"));
    assert!(
        home.outlinks.len() > 5,
        "home should have multiple outlinks, got {}",
        home.outlinks.len()
    );
    assert_eq!(home.in_sitemap, Some(true));

    // Noindex: blocked by robots.txt
    let noindex = find_page(&pages, "/noindex.html");
    assert!(
        noindex.is_some(),
        "noindex.html should appear as a robots.txt-blocked record"
    );
    let noindex = noindex.unwrap();
    assert_eq!(
        noindex.blocked_by_robots,
        Some(true),
        "noindex.html should be marked as blocked by robots.txt"
    );
    assert_eq!(noindex.status, None, "blocked page should have no status");

    // Missing meta
    let missing = find_page(&pages, "/missing-meta.html").expect("missing-meta should be crawled");
    assert!(
        missing.h1.as_deref().is_none_or(|h| h.is_empty()),
        "missing-meta should have no h1, got {:?}",
        missing.h1
    );
    assert!(
        missing.meta_description.is_none()
            || missing
                .meta_description
                .as_deref()
                .is_some_and(|d| d.is_empty()),
        "missing-meta should have no meta description"
    );

    // Duplicate title
    let dup =
        find_page(&pages, "/duplicate-title.html").expect("duplicate-title should be crawled");
    assert_eq!(
        dup.title.as_deref(),
        Some("Test Site Home"),
        "duplicate-title should share the home page title"
    );

    // About: structured data + h2
    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    assert_eq!(about.status, Some(200));
    assert!(
        about.h2.as_deref().is_some_and(|h| !h.is_empty()),
        "about should have h2"
    );
    assert!(
        about.sd_items.len() >= 2,
        "about should have at least 2 structured data items (Organization + Product), got {}",
        about.sd_items.len()
    );
    assert!(
        about.sd_jsonld_count >= 2,
        "about should have at least 2 JSON-LD blocks"
    );
    assert_eq!(about.in_sitemap, Some(true));

    // Images
    let images = find_page(&pages, "/images.html").expect("images should be crawled");
    assert_eq!(images.images.len(), 3, "images page should have 3 images");

    // SPA: HTTP mode sees empty shell (no Chrome rendering)
    let spa = find_page(&pages, "/spa.html").expect("spa should be crawled");
    assert!(
        spa.h1.as_deref().is_none_or(|h| h.is_empty()),
        "SPA h1 should be empty in HTTP mode, got {:?}",
        spa.h1
    );
    assert!(
        spa.word_count.unwrap_or(0) < 20,
        "SPA word count should be low in HTTP mode, got {:?}",
        spa.word_count
    );
    assert_eq!(
        spa.ssr_word_count, None,
        "SSR diff should not run in HTTP mode"
    );
    assert_eq!(spa.ssr_content_missing, None);

    // Redirect
    let redirect = find_page(&pages, "/redirect.html").expect("redirect should be crawled");
    assert_eq!(redirect.title.as_deref(), Some("Redirect"));

    // External resources
    let external = find_page(&pages, "/external.html").expect("external should be crawled");
    assert!(
        external
            .outlinks
            .iter()
            .any(|l| l.dst_url.contains("example.com")),
        "external page should link to example.com"
    );
    assert!(
        external
            .images
            .iter()
            .any(|i| i.src.contains("example.com")),
        "external page should reference external image"
    );

    // Performance / a11y / SSR: all None or zero in HTTP mode
    for page in &pages {
        assert_eq!(
            page.ttfb_ms, None,
            "TTFB should be None in HTTP mode for {}",
            page.url
        );
        assert_eq!(
            page.lcp_ms, None,
            "LCP should be None in HTTP mode for {}",
            page.url
        );
        assert_eq!(
            page.cls, None,
            "CLS should be None in HTTP mode for {}",
            page.url
        );
        assert_eq!(
            page.fcp_ms, None,
            "FCP should be None in HTTP mode for {}",
            page.url
        );
        assert_eq!(
            page.ssr_word_count, None,
            "SSR word count should be None in HTTP mode"
        );
        assert_eq!(
            page.ssr_content_missing, None,
            "SSR content missing should be None in HTTP mode"
        );
    }

    // Sitemap orphan
    let orphan = find_page(&pages, "/orphan-page.html");
    assert!(
        orphan.is_some(),
        "orphan page from sitemap should be present"
    );
    let orphan = orphan.unwrap();
    assert_eq!(orphan.status, Some(404), "orphan page should return 404");
    assert_eq!(orphan.in_sitemap, Some(true));

    // Sitemap membership powers the "In sitemap" / "Not in sitemap" filters on
    // the Sitemaps tab. A crawled page absent from the sitemap must be recorded
    // as Some(false) (not None); a None here means sitemap matching never ran
    // and every sitemap filter would silently select nothing.
    assert_eq!(
        images.in_sitemap,
        Some(false),
        "images.html is crawled but not listed in the sitemap, so it should be marked Not in sitemap"
    );
    // Response headers must be captured (requires spider's `headers` feature).
    // Without them content_type detection and the entire Security tab (HSTS,
    // CSP, X-Frame-Options, ...) silently treat every page as header-less.
    assert!(
        home.headers
            .iter()
            .any(|(name, _)| name.eq_ignore_ascii_case("content-type")),
        "home page should capture response headers, got {:?}",
        home.headers
    );
    // Resources (CSS/JS/images) are discovered from Chrome's Resource Timing in
    // Chrome mode only, so the HTTP crawl should not produce resource rows.
    assert!(
        find_page(&pages, "/style.css").is_none(),
        "HTTP mode should not crawl subresources"
    );

    let in_sitemap_count = pages.iter().filter(|p| p.in_sitemap == Some(true)).count();
    let not_in_sitemap_count = pages.iter().filter(|p| p.in_sitemap == Some(false)).count();
    assert!(
        in_sitemap_count > 0,
        "at least one page should populate the In-sitemap filter"
    );
    assert!(
        not_in_sitemap_count > 0,
        "at least one page should populate the Not-in-sitemap filter"
    );
}

#[test]
fn test_chrome_crawl() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=debug".into()),
        )
        .with_test_writer()
        .try_init();

    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(120),
    );

    server.kill();

    assert!(!pages.is_empty(), "crawl should discover pages");

    eprintln!(
        "Chrome crawl found {} pages: {:?}",
        pages.len(),
        page_paths(&pages)
    );

    // -- Rendered content --

    // SPA: JavaScript-injected content visible in Chrome
    let spa = find_page(&pages, "/spa.html").expect("spa should be crawled");
    assert_eq!(
        spa.h1.as_deref(),
        Some("Client Rendered Heading"),
        "Chrome should render the JS-injected h1"
    );
    assert!(
        spa.word_count.unwrap_or(0) >= 50,
        "Chrome-rendered SPA should have substantial content, got {:?} words",
        spa.word_count
    );

    // -- SSR diff --

    assert!(
        spa.ssr_word_count.unwrap_or(u32::MAX) < 10,
        "raw server HTML should be near-empty, got {:?} SSR words",
        spa.ssr_word_count
    );
    assert_eq!(
        spa.ssr_content_missing,
        Some(true),
        "SPA should be flagged as SSR content missing"
    );
    let spa_pct = ssr_diff_pct(spa);
    assert!(
        spa_pct >= 90,
        "SPA SSR diff should be >= 90%, got {spa_pct}%"
    );

    // About: server-rendered content matches
    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    assert_eq!(
        about.ssr_content_missing,
        Some(false),
        "about.html is static HTML, SSR should not be flagged"
    );
    let about_pct = ssr_diff_pct(about);
    assert!(
        about_pct <= 35,
        "about.html SSR diff should be low, got {about_pct}%"
    );

    // Home: also static
    let home = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home should be crawled");
    assert_eq!(home.ssr_content_missing, Some(false));

    // -- Performance metrics --

    assert!(
        home.ttfb_ms.is_some(),
        "Chrome mode should populate TTFB for home"
    );
    assert!(
        home.lcp_ms.is_some(),
        "Chrome mode should populate LCP for home"
    );
    assert!(
        home.fcp_ms.is_some(),
        "Chrome mode should populate FCP for home"
    );
    assert!(
        about.cls.is_some(),
        "Chrome mode should populate CLS for about"
    );
    assert!(
        pages.iter().any(|p| p.ttfb_ms.is_some_and(|v| v > 0)),
        "at least one page should have TTFB > 0"
    );

    // -- Accessibility --

    let a11y = find_page(&pages, "/a11y.html").expect("a11y page should be crawled");
    assert!(
        a11y.a11y_errors > 0,
        "a11y page should have errors (image-alt, button-name, link-name), got {}",
        a11y.a11y_errors
    );
    assert!(
        a11y.a11y_issues
            .iter()
            .any(|i| i.rule.contains("image-alt") || i.rule.contains("img-alt")),
        "a11y issues should include image-alt rule"
    );
    assert_eq!(home.a11y_errors, 0, "home page should have no a11y errors");

    // -- Structured data --

    assert!(
        about
            .sd_items
            .iter()
            .any(|sd| sd.type_name == "Organization"),
        "about should have Organization structured data"
    );
    assert!(
        about.sd_items.iter().any(|sd| sd.type_name == "Product"),
        "about should have Product structured data"
    );

    // -- Images --

    let images = find_page(&pages, "/images.html").expect("images should be crawled");
    assert_eq!(images.images.len(), 3, "images page should have 3 images");

    let external = find_page(&pages, "/external.html").expect("external should be crawled");
    assert!(
        external
            .images
            .iter()
            .any(|i| i.src.contains("example.com")),
        "external page should have an external image"
    );

    // -- Resources (harvested from Chrome's Resource Timing) --
    //
    // The resources Chrome loads to render a page (CSS/JS/images) are recorded
    // as their own rows without re-fetching, which is what populates the
    // Internal tab's CSS / JavaScript / Images filters.
    let css = find_page(&pages, "/style.css").expect("stylesheet should be a resource row");
    assert!(
        css.content_type
            .as_deref()
            .is_some_and(|ct| ct.contains("css")),
        "style.css resource should report a CSS content type, got {:?}",
        css.content_type
    );
    let js = find_page(&pages, "/app.js").expect("script should be a resource row");
    assert!(
        js.content_type
            .as_deref()
            .is_some_and(|ct| ct.contains("javascript")),
        "app.js resource should report a JavaScript content type, got {:?}",
        js.content_type
    );
    let logo = find_page(&pages, "/img/logo.png").expect("image should be a resource row");
    assert!(
        logo.content_type
            .as_deref()
            .is_some_and(|ct| ct.starts_with("image/")),
        "logo.png resource should report an image content type, got {:?}",
        logo.content_type
    );

    // -- Links --

    assert!(
        external
            .outlinks
            .iter()
            .any(|l| l.dst_url.contains("example.com")),
        "external page should link to example.com"
    );
    assert!(
        home.outlinks.len() > 5,
        "home should have multiple outlinks"
    );

    // -- Robots.txt blocked --

    let noindex = find_page(&pages, "/noindex.html");
    assert!(
        noindex.is_some(),
        "noindex.html should appear as robots.txt-blocked"
    );
    let noindex = noindex.unwrap();
    assert_eq!(noindex.blocked_by_robots, Some(true));
    assert_eq!(noindex.status, None);

    // -- Sitemap --

    assert_eq!(home.in_sitemap, Some(true));
    assert_eq!(about.in_sitemap, Some(true));

    let orphan = find_page(&pages, "/orphan-page.html");
    assert!(orphan.is_some(), "sitemap orphan should be present");
    let orphan = orphan.unwrap();
    assert_eq!(orphan.status, Some(404));
    assert_eq!(orphan.in_sitemap, Some(true));
}
