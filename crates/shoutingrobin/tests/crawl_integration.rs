mod common;

use common::*;
use std::collections::{HashMap, HashSet};
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

    // Markup validity: read off what the server sent, since a parser invents a
    // body element when the server omits one.
    let no_body = find_page(&pages, "/no-body.html").expect("the no-body page should be crawled");
    assert_eq!(no_body.has_body_tag, Some(false));
    let home_body = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home page should be crawled");
    assert_eq!(home_body.has_body_tag, Some(true));

    // Heading order: the page whose first heading is an H2 is out of order, and
    // the home page, which opens with its H1, is not.
    let heading_order =
        find_page(&pages, "/heading-order.html").expect("the heading-order page should be crawled");
    assert_eq!(heading_order.h2_non_sequential, Some(true));
    assert_eq!(home_body.h2_non_sequential, Some(false));

    // Home
    let home = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home page should be crawled");
    assert_eq!(home.status, Some(200));
    // The home page's canonical is the relative `/index.html`, i.e. it points at
    // itself. It must resolve against the page URL before being compared, or a
    // self-referencing canonical reads as canonicalised elsewhere.
    assert_eq!(home.canonical.as_deref(), Some("/index.html"));
    assert_eq!(home.indexability.as_deref(), Some("Indexable"));
    assert_eq!(home.indexability_status(), "Indexable");
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
            .any(|l| l.dst_url.contains("example.invalid")),
        "external page should link to example.invalid"
    );
    assert!(
        external
            .images
            .iter()
            .any(|i| i.src.contains("example.invalid")),
        "external page should reference external image"
    );

    // External bail: external-link.html links off-domain to books.invalid.
    // We capture it as an outlink but must never crawl or analyze the external
    // host as a page of its own. Asserting on this also keeps the test offline.
    let external_link =
        find_page(&pages, "/external-link.html").expect("external-link page should be crawled");
    assert!(
        external_link
            .outlinks
            .iter()
            .any(|l| l.dst_url.contains("books.invalid")),
        "external-link page should record the off-domain link as an outlink"
    );
    assert!(
        !pages.iter().any(|p| p.url.contains("books.invalid")),
        "off-domain host must not be crawled as its own page, found: {:?}",
        pages
            .iter()
            .map(|p| p.url.as_str())
            .filter(|u| u.contains("books.toscrape.com"))
            .collect::<Vec<_>>()
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
fn test_slashless_start_url_is_not_a_redirect() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    // No trailing slash: Chrome normalizes this to `...:<port>/` when it
    // navigates, and the pump must not mistake that normalization for a
    // redirect. A false redirect would skip analysis of the start page
    // entirely, leaving its outlinks/title/H1 empty.
    let root_url = format!("http://127.0.0.1:{port}");

    let pages = crawl_test_site(&root_url);

    server.kill();

    let root = pages
        .iter()
        .find(|p| p.url == root_url || path_of(&p.url) == "/")
        .unwrap_or_else(|| {
            panic!(
                "start page should be crawled, got pages: {:?}",
                page_paths(&pages)
            )
        });

    assert!(
        root.redirect_url.is_none(),
        "start page must not be treated as a redirect (redirect_url = {:?})",
        root.redirect_url
    );
    assert!(
        !root.outlinks.is_empty(),
        "start page should have outlinks after analysis, got {}",
        root.outlinks.len()
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

    // -- Links only JavaScript produces --
    //
    // The SPA serves one link and its script writes three more, two of them to
    // the same page and one off-site. The counts beside each link total say how
    // much of the graph a crawler without JavaScript would never see.
    let csr_out = spa.outlinks.iter().filter(|link| link.csr_only).count();
    let unique_csr_out: std::collections::HashSet<&str> = spa
        .outlinks
        .iter()
        .filter(|link| link.csr_only)
        .map(|link| link.dst_url.as_str())
        .collect();
    let external_csr_out: Vec<&str> = spa
        .outlinks
        .iter()
        .filter(|link| link.csr_only && !link.dst_url.contains(&format!("127.0.0.1:{port}")))
        .map(|link| link.dst_url.as_str())
        .collect();
    assert_eq!(csr_out, 3, "three links are written by the script");
    assert_eq!(
        unique_csr_out.len(),
        2,
        "two of the three point at the same page, got {unique_csr_out:?}"
    );
    assert_eq!(
        external_csr_out,
        vec!["https://rendered.invalid/page"],
        "one of them leaves the site"
    );
    assert!(
        spa.outlinks
            .iter()
            .any(|link| !link.csr_only && link.dst_url.ends_with("/index.html")),
        "the link in the served markup is not a rendered-only one"
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
            .any(|i| i.src.contains("example.invalid")),
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
            .any(|l| l.dst_url.contains("example.invalid")),
        "external page should link to example.invalid"
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

/// The records streamed during a crawl predate the post-crawl passes, which
/// write straight to the database. The app reloads from the database once the
/// crawl finishes; this asserts the reloaded records actually carry those
/// results, so the live session matches what reopening the crawl would show.
#[test]
fn test_reload_after_finish_carries_post_crawl_analysis() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let streamed = crawl_test_site(&root_url);
    let reloaded = crawl_test_site_reloaded(&root_url);

    server.kill();

    assert!(
        !reloaded.is_empty(),
        "reload should return the crawled pages"
    );

    assert!(
        streamed.iter().all(|p| p.link_score.is_none()),
        "streamed records predate the PageRank pass, so they carry no link score"
    );
    assert!(
        reloaded.iter().any(|p| p.link_score.is_some()),
        "reloaded records should carry the link scores the PageRank pass persisted"
    );
}

/// The CSR half of the link counts: how much of a page's link graph exists
/// only once JavaScript has run.
///
/// Needs both a rendered crawl and the post-crawl reload — a rendered crawl to
/// see the links at all, and the reload because an inlink count is a property
/// of the whole graph rather than of the page that streamed through.
#[test]
fn test_csr_link_counts_cover_what_only_rendering_produces() {
    // No chrome guard here: the crawl helper takes it, and the mutex behind it
    // is not reentrant, so taking it twice on one thread deadlocks the test.
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_reloaded_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(120),
    );
    server.kill();

    // The SPA's script writes two links to this page and one off-site; the
    // served markup links only home.
    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    assert!(
        about.csr_inlinks_count >= 2,
        "the script writes two links here, got {}",
        about.csr_inlinks_count
    );
    assert_eq!(
        about.unique_csr_inlinks_count, 1,
        "both come from one page, got {}",
        about.unique_csr_inlinks_count
    );
    assert!(
        about.unique_csr_inlinks_count < about.csr_inlinks_count,
        "two links from one page is one unique inlink, got {} of {}",
        about.unique_csr_inlinks_count,
        about.csr_inlinks_count
    );

    // A page nothing links to after rendering has none of either, rather than
    // inheriting its plain inlink count.
    let home = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home should be crawled");
    assert!(
        home.inlinks_count > 0,
        "the fixture links home from several pages"
    );
    assert_eq!(home.csr_inlinks_count, 0);
    assert_eq!(home.unique_csr_inlinks_count, 0);
}

/// Inlink counts must reflect the whole link graph, not just the pages that
/// happened to be crawled before a given page streamed through.
#[test]
fn test_inlink_counts_reflect_the_whole_link_graph() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_reloaded(&root_url);

    server.kill();

    let mut total: HashMap<&str, u32> = HashMap::new();
    let mut unique: HashMap<&str, HashSet<&str>> = HashMap::new();
    for page in &pages {
        for link in &page.outlinks {
            *total.entry(link.dst_url.as_str()).or_default() += 1;
            unique
                .entry(link.dst_url.as_str())
                .or_default()
                .insert(page.url.as_str());
        }
    }

    let home = find_page(&pages, "/index.html")
        .or_else(|| find_page(&pages, &format!(":{port}/")))
        .expect("home page should be crawled");
    assert!(
        home.inlinks_count > 1,
        "home should be linked from more than one page, got {}",
        home.inlinks_count
    );

    for page in &pages {
        let expected_total = total.get(page.url.as_str()).copied().unwrap_or(0);
        let expected_unique = unique
            .get(page.url.as_str())
            .map(|sources| sources.len() as u32)
            .unwrap_or(0);
        assert_eq!(
            page.inlinks_count, expected_total,
            "inlinks for {} should match the link graph",
            page.url
        );
        assert_eq!(
            page.unique_inlinks_count, expected_unique,
            "unique inlinks for {} should count distinct sources",
            page.url
        );
    }
}

/// Crawl depth is the number of clicks from the start page. It must come from a
/// walk of the link graph, not default to zero for every URL.
#[test]
fn test_crawl_depth_is_computed_from_the_link_graph() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_reloaded(&root_url);

    server.kill();

    for page in &pages {
        eprintln!("depth {:?} {}", page.depth, path_of(&page.url));
    }

    let home = find_page(&pages, &format!(":{port}/"))
        .filter(|p| path_of(&p.url) == "/")
        .expect("home page should be crawled");
    assert_eq!(home.depth, Some(0), "the start page is zero clicks deep");

    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    assert_eq!(
        about.depth,
        Some(1),
        "about.html is linked from the start page"
    );

    assert!(
        pages.iter().any(|p| p.depth.is_some_and(|d| d > 0)),
        "pages beyond the start page should have a non-zero depth"
    );

    // A sitemap-only orphan is never reached by following links, so its depth is
    // unknown rather than zero, which would put it level with the start page.
    let orphan = find_page(&pages, "/orphan-page.html").expect("orphan should be present");
    assert_eq!(orphan.depth, None, "an unlinked URL has no crawl depth");
}

/// HTML requires `&` to be escaped in attribute values, so a href of
/// `?ref=nav&amp;page=2` addresses `?ref=nav&page=2`. Queueing the raw text
/// instead fetches a URL nothing linked to and misses the real one.
#[test]
fn test_escaped_ampersands_in_hrefs_are_decoded_before_crawling() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site(&root_url);

    server.kill();

    assert!(
        pages.iter().all(|p| !p.url.contains("amp;")),
        "no crawled URL should carry an undecoded entity, got {:?}",
        pages
            .iter()
            .map(|p| p.url.as_str())
            .filter(|u| u.contains("amp;"))
            .collect::<Vec<_>>()
    );
    assert!(
        pages
            .iter()
            .any(|p| p.url.contains("about.html?ref=nav&page=2")),
        "the decoded URL should have been crawled, got {:?}",
        page_paths(&pages)
    );
}

/// Everything the pages point at is requested once after the crawl: images,
/// stylesheets, scripts and links to other sites. Without this pass a broken
/// image is invisible and no image has a known size.
#[test]
fn test_discovered_resources_are_status_checked() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_checking_resources(&root_url);

    server.kill();

    let resource = |path: &str| {
        pages
            .iter()
            .find(|page| page.is_resource && page.url.ends_with(path))
    };

    let stylesheet = resource("/style.css").expect("the stylesheet should be a row of its own");
    assert_eq!(stylesheet.status, Some(200));
    assert_eq!(stylesheet.content_type.as_deref(), Some("text/css"));
    assert!(
        stylesheet.size_bytes > 0,
        "a 200 stylesheet should have a size"
    );

    let script = resource("/app.js").expect("the script should be a row of its own");
    assert_eq!(script.status, Some(200));

    let logo = resource("/img/logo.png").expect("the logo should be a row of its own");
    assert_eq!(logo.status, Some(200));
    assert_eq!(logo.content_type.as_deref(), Some("image/png"));
    assert!(logo.size_bytes > 0, "a 200 image should have a size");
    assert_eq!(logo.resource_initiator.as_deref(), Some("img"));

    // images.html references two files that do not exist. Being able to say so
    // is the point of the pass.
    let missing = resource("/img/banner.png").expect("a missing image is still a row");
    assert_eq!(missing.status, Some(404));

    // A page the crawler reached itself must not be re-listed as a resource.
    assert!(
        !pages
            .iter()
            .any(|page| page.is_resource && page.url.ends_with("/about.html")),
        "a crawled page should not also appear as a resource row"
    );
}

/// A sitemap's `<lastmod>` is captured for the URLs it advertises, both for
/// pages the crawl reached and for the ones it never linked to.
#[test]
fn test_sitemap_lastmod_is_recorded() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");
    write_sitemap(port);

    let pages = crawl_test_site(&root_url);

    server.kill();

    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    assert_eq!(about.in_sitemap, Some(true));
    assert_eq!(about.sitemap_lastmod.as_deref(), Some("2026-08-01"));

    // The sitemap lists a page nothing links to, so it arrives as an orphan
    // row after the crawl, carrying its lastmod like any other entry.
    let orphan = pages
        .iter()
        .find(|page| page.url.ends_with("/orphan-page.html"))
        .expect("the sitemap orphan should be reported");
    assert_eq!(
        orphan.sitemap_lastmod.as_deref(),
        Some("2026-07-15T09:30:00+02:00")
    );

    // An entry with no lastmod stays empty rather than borrowing a neighbour's.
    let home = find_page(&pages, "/index.html").expect("home should be crawled");
    assert_eq!(home.sitemap_lastmod, None);
}

/// A sitemap is one of the three places hreflang may live. A page annotated
/// only there still carries its alternates.
#[test]
fn test_sitemap_hreflang_reaches_the_page() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");
    write_sitemap(port);

    let pages = crawl_test_site(&root_url);

    server.kill();

    let about = find_page(&pages, "/about.html").expect("about should be crawled");
    let langs: Vec<&str> = about
        .hreflang_tags
        .iter()
        .map(|(lang, _)| lang.as_str())
        .collect();
    assert!(
        langs.contains(&"en") && langs.contains(&"sv"),
        "the sitemap's alternates should reach the page, got {:?}",
        about.hreflang_tags
    );
    assert_eq!(
        about.hreflang_sources,
        vec![shoutingrobin::crawl::event::HreflangSource::Sitemap],
        "the page's own HTML has no hreflang, so the sitemap is the only source"
    );
}

/// The crawler's SSRF guard refuses to follow a redirect to a loopback host,
/// which is what a 127.0.0.1 fixture produces. That refusal wins: the redirect
/// is recorded as a URL we could not fetch, and no target is invented for it.
/// Resolving a redirect we *can* read is covered by the engine's unit tests,
/// which a loopback server cannot exercise.
#[test]
fn test_a_refused_redirect_is_not_resolved_behind_the_guard() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_reloaded(&root_url);

    server.kill();

    let moved = pages
        .iter()
        .find(|page| page.url.ends_with("/moved"))
        .expect("the redirect itself should be recorded");
    assert_eq!(
        moved.status,
        Some(599),
        "spider reports its own refusal, not the 301 the server sent"
    );
    assert_eq!(moved.redirect_url, None);
    assert!(
        find_page(&pages, "/redirect-target.html").is_none(),
        "nothing links to the target, and the guard stopped us reading where the redirect went"
    );
}

/// Sitemap orphans and robots-blocked URLs are reported by the crawl and must
/// survive it: both were live-only events, so reopening a crawl lost them.
#[test]
fn test_orphans_and_blocked_urls_are_persisted() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");
    write_sitemap(port);

    // Reloaded from the database, which is where they used to go missing.
    let pages = crawl_test_site_reloaded(&root_url);

    server.kill();

    // Listed in the sitemap, disallowed by robots.txt, linked from nowhere: a
    // URL the sitemap advertises and the crawl never reaches.
    let orphan = find_page(&pages, "/sitemap-only.html")
        .expect("a sitemap URL the crawl never reached should survive it");
    assert_eq!(orphan.in_sitemap, Some(true));
    assert_eq!(orphan.sitemap_lastmod.as_deref(), Some("2026-05-02"));
    assert_eq!(orphan.status, None, "it was never fetched");
    assert!(
        !orphan.is_page,
        "an uncrawled URL is not a document, so it stays off the content tabs"
    );

    let blocked =
        find_page(&pages, "/noindex.html").expect("a robots-blocked URL should survive the crawl");
    assert_eq!(blocked.blocked_by_robots, Some(true));
    assert_eq!(blocked.status, None);

    // One row each, not one per source that reported them.
    for path in ["/sitemap-only.html", "/noindex.html"] {
        let count = pages.iter().filter(|page| page.url.ends_with(path)).count();
        assert_eq!(count, 1, "{path} should be a single row");
    }
}
