//! End-to-end coverage for every results-grid filter.
//!
//! A purpose-built site is crawled (in both HTTP and Chrome render modes) and
//! the resulting `PageRecord`s are loaded back exactly the way the grid loads a
//! selected crawl (`storage::load_pages_for_crawl`, so post-crawl fields like
//! hreflang issues, near-duplicate counts and link scores are present). Then,
//! for every `ResultTab` and every `IssueFilter` that tab exposes, we assert
//! that `matching_urls` selects the pages we engineered to trigger it.
//!
//! The expected-matches table (`expectation`) is an exhaustive `match` over
//! `IssueFilter` with no wildcard arm: adding a new filter variant fails to
//! compile until its expected behavior is declared here.
//!
//! A handful of conditions cannot be produced by a normalized live crawl
//! (resource content-types are not crawled as pages, URLs never contain literal
//! spaces/non-ascii, axe rule firing is environment-dependent, sitemap orphans
//! are not persisted, and redirects to loopback/private hosts are refused by the
//! crawler's SSRF guard so a 127.0.0.1 test server cannot exercise a followed
//! redirect). Those are covered by injected synthetic `PageRecord`s
//! (`synthetic_pages`), which exercise the pure filter logic deterministically.
//!
//! Route map (served by `spawn_site`): see `route` below.

use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::crawl::CrawlConfig;
use crate::crawl::event::{A11yIssue, CrawlEvent, ImageRef, Outlink, PageRecord};
use crate::crawl::render_mode::RenderMode;
use crate::views::ResultTab;
use crate::views::results_grid::{
    IssueFilter, filters_for_tab, matching_urls, tab_filter_counts_for_test, tab_is_flattened,
};

const LONG_PATH: &str = "/this-is-a-very-long-url-path-segment-that-keeps-going-and-going-and-going-well-past-one-hundred-and-fifteen-characters-total-x";

/// Internal HTML routes linked from the home page so the spider discovers them.
fn linked_paths() -> Vec<String> {
    let mut paths: Vec<String> = [
        "/missing-all",
        "/dup-a",
        "/dup-b",
        "/long-all",
        "/short-all",
        "/multiple-all",
        "/title-eq-h1",
        "/exact-dup-a",
        "/exact-dup-b",
        "/near-dup-a",
        "/near-dup-b",
        "/low-content",
        "/no-body",
        "/large",
        "/images",
        "/canonical-self",
        "/canonical-other",
        "/hreflang-a",
        "/hreflang-b",
        "/hreflang-c",
        "/hreflang-d",
        "/sd-article",
        "/sd-faq",
        "/sd-howto",
        "/sd-recipe",
        "/sd-video",
        "/sd-breadcrumb",
        "/sd-organization",
        "/sd-microdata",
        "/sd-errors",
        "/sd-product",
        "/product-bare",
        "/slow-perf",
        "/not-found",
        "/server-error",
        "/secure",
        "/MixedCase",
        "/under_score",
        "/multi//slash",
        "/withparam?x=1",
        "/robots-meta",
        "/directive-none",
        "/x-robots",
        "/external",
        "/links",
        "/a11y",
        "/spa",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    paths.push(LONG_PATH.to_string());
    paths
}

// ---------------------------------------------------------------------------
// Test server
// ---------------------------------------------------------------------------

#[allow(clippy::zombie_processes)]
fn spawn_site() -> (std::thread::JoinHandle<()>, u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();

    let handle = std::thread::spawn(move || {
        let base = format!("http://127.0.0.1:{port}");
        while !stop_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 8192];
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let path = req
                        .lines()
                        .next()
                        .and_then(|l| l.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .to_string();

                    let (status_line, extra_headers, body) = route(&path, &base);
                    let response = format!(
                        "{status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {len}\r\nServer: coverage-test\r\n{extra}Connection: close\r\n\r\n{body}",
                        len = body.len(),
                        extra = extra_headers,
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });

    (handle, port, stop)
}

fn doc(title: &str, head_extra: &str, body_inner: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <title>{title}</title>{head_extra}</head><body>{body_inner}</body></html>"
    )
}

fn lorem(words: usize) -> String {
    let mut out = String::new();
    for i in 0..words {
        out.push_str("lorem");
        out.push_str(&i.to_string());
        out.push(' ');
    }
    out
}

/// Returns (status_line, extra_header_block, body). The header block, if
/// non-empty, must end in "\r\n".
fn route(path: &str, base: &str) -> (&'static str, String, String) {
    let ok = "HTTP/1.1 200 OK";
    let no_headers = String::new();
    let plain = |title: &str, h1: &str, meta: &str, h2: &str, extra_body: &str| -> String {
        doc(
            title,
            &format!("<meta name=\"description\" content=\"{meta}\">"),
            &format!("<h1>{h1}</h1><h2>{h2}</h2><p>{extra_body}</p>"),
        )
    };

    // Strip query string for routing while preserving exact-path matches.
    let route_path = path.split('?').next().unwrap_or(path);

    match route_path {
        "/" => {
            let mut links = String::new();
            for p in linked_paths() {
                links.push_str(&format!("<a href=\"{p}\">link {p}</a> "));
            }
            let body = format!(
                "<h1>Coverage Test Home</h1><h2>Overview Section</h2><p>{}</p>{links}",
                lorem(120)
            );
            (
                ok,
                no_headers,
                doc(
                    "Shouting Robin Coverage Test Home Landing Page",
                    "<meta name=\"description\" content=\"A representative home page that serves as the crawl entry point for the filter coverage test suite.\">",
                    &body,
                ),
            )
        }
        // Markup with no `<body>` element, which a parser invents and the
        // source therefore has to be read for. Written out rather than built
        // with `doc`, which always writes one.
        "/no-body" => (
            ok,
            no_headers,
            format!(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
                 <title>A page whose markup has no body element at all</title>\
                 <meta name=\"description\" content=\"Markup that goes from the head \
                 straight into content, the way a JavaScript shell often does.\"></head>\
                 <h1>No body tag</h1><h2>Recovered by the parser</h2><p>{}</p></html>",
                lorem(120)
            ),
        ),
        "/robots.txt" => (
            ok,
            no_headers,
            format!("User-agent: *\nAllow: /\nSitemap: {base}/sitemap.xml\n"),
        ),
        "/sitemap.xml" => {
            let xml = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                 <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\
                 <url><loc>{base}/</loc><lastmod>2026-08-01</lastmod></url>\
                 <url><loc>{base}/robots-meta</loc></url>\
                 <url><loc>{base}/not-found</loc></url>\
                 </urlset>"
            );
            (ok, no_headers, xml)
        }

        "/missing-all" => (
            ok,
            no_headers,
            // No title, no meta description, no h1, no h2.
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"></head><body><p>page with no title heading or meta description at all here</p></body></html>".to_string(),
        ),
        "/dup-a" => (
            ok,
            no_headers,
            plain(
                "Shared Duplicate Title Across Two Distinct Pages",
                "Shared Duplicate Heading One",
                "Shared duplicate meta description that is intentionally repeated on two pages.",
                "Shared Duplicate Heading Two",
                "unique body alpha for the first duplicate page so content hashes differ",
            ),
        ),
        "/dup-b" => (
            ok,
            no_headers,
            plain(
                "Shared Duplicate Title Across Two Distinct Pages",
                "Shared Duplicate Heading One",
                "Shared duplicate meta description that is intentionally repeated on two pages.",
                "Shared Duplicate Heading Two",
                "unique body bravo for the second duplicate page so content hashes differ",
            ),
        ),
        "/long-all" => {
            let long_title = "Very long page title that intentionally exceeds the sixty character recommendation by a wide margin indeed";
            let long_meta = "Very long meta description that intentionally exceeds the one hundred and sixty character recommendation for meta descriptions by repeating filler words filler words filler words filler words.";
            let long_h = "Very long heading that intentionally exceeds the seventy character heading length recommendation by a margin";
            (
                ok,
                no_headers,
                plain(long_title, long_h, long_meta, long_h, "body for the over-length page"),
            )
        }
        "/short-all" => (
            ok,
            no_headers,
            plain("Hi", "Short", "Tiny meta.", "Short Two", "body for the under-length page"),
        ),
        "/multiple-all" => (
            ok,
            no_headers,
            doc(
                "First Title Element For The Multiple Tags Page",
                "<title>Second Title Element For The Multiple Tags Page</title>\
                 <meta name=\"description\" content=\"First meta description for the multiple tags page here.\">\
                 <meta name=\"description\" content=\"Second meta description for the multiple tags page here.\">",
                "<h1>First H1 Multiple</h1><h1>Second H1 Multiple</h1>\
                 <h2>First H2 Multiple</h2><h2>Second H2 Multiple</h2><p>multiple tags body</p>",
            ),
        ),
        "/title-eq-h1" => (
            ok,
            no_headers,
            plain(
                "Identical Title And Heading String Value",
                "Identical Title And Heading String Value",
                "Meta description for the page whose title equals its h1 heading exactly.",
                "Distinct H2 For Title Equals H1 Page",
                "body for title equals h1 page",
            ),
        ),
        "/exact-dup-a" | "/exact-dup-b" => (
            ok,
            no_headers,
            plain(
                "Exact Duplicate Content Pair Page Title Value",
                "Exact Duplicate Content Heading",
                "Meta for the exact duplicate content pair which is byte identical across both.",
                "Exact Duplicate Subheading",
                "the exact same body paragraph text appears verbatim on both pages of this pair",
            ),
        ),
        // Near-duplicate pair: identical title/headings/meta and a large
        // identical body differing by a single trailing token, so the simhashes
        // land within the 90% threshold (<=6 differing bits) while the content
        // hashes differ (so they are not classed as exact duplicates).
        "/near-dup-a" => (
            ok,
            no_headers,
            plain(
                "Near Duplicate Content Pair Shared Page Title",
                "Near Duplicate Content Shared Heading",
                "Shared meta for the near duplicate content pair under test here.",
                "Near Duplicate Shared Subheading",
                &format!("{}alphaonlytoken", lorem(400)),
            ),
        ),
        "/near-dup-b" => (
            ok,
            no_headers,
            plain(
                "Near Duplicate Content Pair Shared Page Title",
                "Near Duplicate Content Shared Heading",
                "Shared meta for the near duplicate content pair under test here.",
                "Near Duplicate Shared Subheading",
                &format!("{}bravoonlytoken", lorem(400)),
            ),
        ),
        "/low-content" => (
            ok,
            no_headers,
            plain(
                "Low Content Page With Very Few Body Words",
                "Low Content Heading",
                "Meta description for the deliberately low word count page under test.",
                "Low Content Subheading",
                "only a handful of words here",
            ),
        ),
        "/large" => (
            ok,
            no_headers,
            plain(
                "Large Body Page With Plenty Of Words Present",
                "Large Body Heading",
                "Meta description for the large body page that has well over one hundred words.",
                "Large Body Subheading",
                &lorem(300),
            ),
        ),
        "/images" => (
            ok,
            no_headers,
            doc(
                "Images Page Covering Every Image Issue Filter",
                "<meta name=\"description\" content=\"Images page exercising every image related filter at once.\">",
                &format!(
                    "<h1>Images Heading</h1><h2>Images Subheading</h2>\
                     <img src=\"/no-alt.png\" width=\"10\" height=\"10\">\
                     <img src=\"/empty-alt.png\" alt=\"\" width=\"10\" height=\"10\">\
                     <img src=\"/long-alt.png\" alt=\"{}\" width=\"10\" height=\"10\">\
                     <img src=\"/no-size.png\" alt=\"present\">\
                     <p>images body</p>",
                    "a".repeat(150)
                ),
            ),
        ),
        "/canonical-self" => (
            ok,
            no_headers,
            doc(
                "Self Referencing Canonical Page Title Value",
                &format!(
                    "<meta name=\"description\" content=\"Self referencing canonical page meta.\">\
                     <link rel=\"canonical\" href=\"{base}/canonical-self\">"
                ),
                "<h1>Self Canonical Heading</h1><h2>Sub</h2><p>self canonical body</p>",
            ),
        ),
        "/canonical-other" => (
            ok,
            no_headers,
            doc(
                "Canonicalised Page Pointing Elsewhere Title",
                &format!(
                    "<meta name=\"description\" content=\"Canonicalised page meta description here.\">\
                     <link rel=\"canonical\" href=\"{base}/\">"
                ),
                "<h1>Canonicalised Heading</h1><h2>Sub</h2><p>canonicalised body</p>",
            ),
        ),
        "/hreflang-a" => (
            ok,
            no_headers,
            doc(
                "Hreflang Page A Triggering Every Hreflang Issue",
                &format!(
                    "<meta name=\"description\" content=\"Hreflang page a meta description here for test.\">\
                     <link rel=\"alternate\" hreflang=\"en\" href=\"{base}/hreflang-a\">\
                     <link rel=\"alternate\" hreflang=\"de\" href=\"{base}/hreflang-b\">\
                     <link rel=\"alternate\" hreflang=\"invalid\" href=\"{base}/hreflang-a\">"
                ),
                "<h1>Hreflang A Heading</h1><h2>Sub</h2><p>hreflang a body</p>",
            ),
        ),
        "/hreflang-b" => (
            ok,
            no_headers,
            doc(
                "Hreflang Page B Target Of Page A Alternates",
                &format!(
                    "<meta name=\"description\" content=\"Hreflang page b meta description here for test.\">\
                     <link rel=\"canonical\" href=\"{base}/canonical-other\">\
                     <link rel=\"alternate\" hreflang=\"fr\" href=\"{base}/hreflang-b\">\
                     <link rel=\"alternate\" hreflang=\"en\" href=\"{base}/sd-article\">"
                ),
                "<h1>Hreflang B Heading</h1><h2>Sub</h2><p>hreflang b body</p>",
            ),
        ),
        // A complete cluster whose only other member lives outside the crawl.
        // Nothing can be asserted about that member, so this page must come back
        // clean rather than reporting a missing return tag.
        "/hreflang-d" => (
            ok,
            no_headers,
            doc(
                "Hreflang Page D Pointing Outside The Crawl",
                &format!(
                    "<meta name=\"description\" content=\"Hreflang page d meta description here for test.\">\
                     <link rel=\"alternate\" hreflang=\"x-default\" href=\"{base}/hreflang-d\">\
                     <link rel=\"alternate\" hreflang=\"sv\" href=\"{base}/hreflang-d\">\
                     <link rel=\"alternate\" hreflang=\"de\" href=\"https://never-crawled.invalid/de/x\">"
                ),
                "<h1>Hreflang D Heading</h1><h2>Sub</h2><p>hreflang d body</p>",
            ),
        ),
        // Points at the other two but never at itself, which is the one
        // hreflang defect the a/b pair doesn't exhibit.
        "/hreflang-c" => (
            ok,
            no_headers,
            doc(
                "Hreflang Page C Omitting Its Own Self Reference",
                &format!(
                    "<meta name=\"description\" content=\"Hreflang page c meta description here for test.\">\
                     <link rel=\"alternate\" hreflang=\"x-default\" href=\"{base}/hreflang-a\">\
                     <link rel=\"alternate\" hreflang=\"en\" href=\"{base}/hreflang-a\">\
                     <link rel=\"alternate\" hreflang=\"de\" href=\"{base}/hreflang-b\">"
                ),
                "<h1>Hreflang C Heading</h1><h2>Sub</h2><p>hreflang c body</p>",
            ),
        ),
        "/sd-article" => (
            ok,
            no_headers,
            doc(
                "Structured Data Article Page Title Value Here",
                "<meta name=\"description\" content=\"JSON-LD Article structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Article\",\"headline\":\"Sample Article Headline\",\"author\":\"Jane Doe\",\"datePublished\":\"2024-01-01\",\"image\":\"https://example.com/a.jpg\"}</script>",
                "<h1>Article Heading</h1><h2>Sub</h2><p>article body</p>",
            ),
        ),
        "/sd-faq" => (
            ok,
            no_headers,
            doc(
                "Structured Data FAQ Page Title Value Here",
                "<meta name=\"description\" content=\"JSON-LD FAQPage structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"FAQPage\",\"mainEntity\":[{\"@type\":\"Question\",\"name\":\"Q?\",\"acceptedAnswer\":{\"@type\":\"Answer\",\"text\":\"A\"}}]}</script>",
                "<h1>FAQ Heading</h1><h2>Sub</h2><p>faq body</p>",
            ),
        ),
        "/sd-howto" => (
            ok,
            no_headers,
            doc(
                "Structured Data HowTo Page Title Value Here",
                "<meta name=\"description\" content=\"JSON-LD HowTo structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"HowTo\",\"name\":\"How To Test\",\"step\":[{\"@type\":\"HowToStep\",\"text\":\"Step one\"}]}</script>",
                "<h1>HowTo Heading</h1><h2>Sub</h2><p>howto body</p>",
            ),
        ),
        "/sd-recipe" => (
            ok,
            no_headers,
            doc(
                "Structured Data Recipe Page Title Value Here",
                "<meta name=\"description\" content=\"JSON-LD Recipe structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Recipe\",\"name\":\"Test Recipe\"}</script>",
                "<h1>Recipe Heading</h1><h2>Sub</h2><p>recipe body</p>",
            ),
        ),
        "/sd-video" => (
            ok,
            no_headers,
            doc(
                "Structured Data Video Page Title Value Here",
                "<meta name=\"description\" content=\"JSON-LD VideoObject structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"VideoObject\",\"name\":\"Test Video\",\"description\":\"desc\",\"thumbnailUrl\":\"https://example.com/t.jpg\",\"uploadDate\":\"2024-01-01\"}</script>",
                "<h1>Video Heading</h1><h2>Sub</h2><p>video body</p>",
            ),
        ),
        "/sd-breadcrumb" => (
            ok,
            no_headers,
            doc(
                "Structured Data Breadcrumb Page Title Value",
                "<meta name=\"description\" content=\"JSON-LD BreadcrumbList structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"BreadcrumbList\",\"itemListElement\":[{\"@type\":\"ListItem\",\"position\":1,\"name\":\"Home\",\"item\":\"https://example.com/\"}]}</script>",
                "<h1>Breadcrumb Heading</h1><h2>Sub</h2><p>breadcrumb body</p>",
            ),
        ),
        "/sd-organization" => (
            ok,
            no_headers,
            doc(
                "Structured Data Organization Page Title Value",
                "<meta name=\"description\" content=\"JSON-LD Organization structured data page meta.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Organization\",\"name\":\"Test Org\",\"url\":\"https://example.com/\"}</script>",
                "<h1>Organization Heading</h1><h2>Sub</h2><p>organization body</p>",
            ),
        ),
        "/sd-microdata" => (
            ok,
            no_headers,
            doc(
                "Structured Data Microdata Page Title Value",
                "<meta name=\"description\" content=\"Microdata structured data page meta description.\">",
                "<div itemscope itemtype=\"https://schema.org/Person\">\
                 <span itemprop=\"name\">Microdata Person</span></div>\
                 <h1>Microdata Heading</h1><h2>Sub</h2><p>microdata body</p>",
            ),
        ),
        "/sd-errors" => (
            ok,
            no_headers,
            doc(
                "Structured Data Parse Error Page Title Value",
                "<meta name=\"description\" content=\"Structured data with invalid JSON to force a parse error.\">\
                 <script type=\"application/ld+json\">{ this is not valid json at all }</script>",
                "<h1>SD Errors Heading</h1><h2>Sub</h2><p>sd errors body</p>",
            ),
        ),
        "/sd-product" => (
            ok,
            no_headers,
            doc(
                "Structured Data Full Product Page Title Value",
                "<meta name=\"description\" content=\"JSON-LD Product with every ecommerce field present.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Product\",\"name\":\"Full Product\",\"sku\":\"SKU-123\",\"gtin13\":\"0123456789012\",\"brand\":{\"@type\":\"Brand\",\"name\":\"Acme\"},\"image\":\"https://example.com/p.jpg\",\"description\":\"A product\",\"aggregateRating\":{\"@type\":\"AggregateRating\",\"ratingValue\":\"4.5\",\"reviewCount\":\"10\"},\"offers\":{\"@type\":\"Offer\",\"price\":\"19.99\",\"priceCurrency\":\"USD\",\"availability\":\"https://schema.org/InStock\"}}</script>",
                "<h1>Full Product Heading</h1><h2>Sub</h2><p>full product body</p>",
            ),
        ),
        "/product-bare" => (
            ok,
            no_headers,
            doc(
                "Structured Data Bare Product Page Title Value",
                "<meta name=\"description\" content=\"JSON-LD Product missing every optional ecommerce field.\">\
                 <script type=\"application/ld+json\">{\"@context\":\"https://schema.org\",\"@type\":\"Product\",\"name\":\"Bare Product\"}</script>",
                "<h1>Bare Product Heading</h1><h2>Sub</h2><p>bare product body</p>",
            ),
        ),
        "/slow-perf" => (
            ok,
            no_headers,
            doc(
                "Slow Performance Metrics Page Title Value",
                "<meta name=\"description\" content=\"Page embedding slow core web vitals metrics for the perf filters.\">\
                 <script id=\"__sr_metrics\">{\"ttfb\":2000,\"lcp\":5000,\"cls\":0.3,\"fcp\":3500}</script>",
                "<h1>Slow Perf Heading</h1><h2>Sub</h2><p>slow perf body</p>",
            ),
        ),
        "/not-found" => (
            "HTTP/1.1 404 Not Found",
            no_headers,
            plain(
                "Not Found Page Title Value Goes Here",
                "Not Found",
                "Meta for the 404 not found page used by the response code filters.",
                "Sub",
                "the unique four oh four not found body text",
            ),
        ),
        "/server-error" => (
            "HTTP/1.1 500 Internal Server Error",
            no_headers,
            plain(
                "Server Error Page Title Value Goes Here",
                "Server Error",
                "Meta for the 500 server error page used by the response code filters.",
                "Sub",
                "the unique five hundred server error body text",
            ),
        ),
        "/secure" => (
            ok,
            "strict-transport-security: max-age=31536000\r\ncontent-security-policy: default-src 'self'\r\nx-frame-options: DENY\r\nx-content-type-options: nosniff\r\n".to_string(),
            plain(
                "Secure Headers Present Page Title Value",
                "Secure",
                "Meta for the page that sends every checked security response header.",
                "Sub",
                "the unique secure headers present body text",
            ),
        ),
        "/MixedCase" => (
            ok,
            no_headers,
            plain(
                "Mixed Case URL Path Page Title Value Here",
                "Mixed Case",
                "Meta for the mixed case url path page used by the url filters.",
                "Sub",
                "the unique mixed case url body text",
            ),
        ),
        "/under_score" => (
            ok,
            no_headers,
            plain(
                "Underscore URL Path Page Title Value Here",
                "Underscore",
                "Meta for the underscore url path page used by the url filters.",
                "Sub",
                "the unique underscore url body text",
            ),
        ),
        "/multi//slash" => (
            ok,
            no_headers,
            plain(
                "Multiple Slash URL Path Page Title Value",
                "Multiple Slash",
                "Meta for the multiple slash url path page used by the url filters.",
                "Sub",
                "the unique multiple slash url body text",
            ),
        ),
        "/withparam" => (
            ok,
            no_headers,
            plain(
                "Query Parameter URL Path Page Title Value",
                "Query Parameter",
                "Meta for the query parameter url path page used by the url filters.",
                "Sub",
                "the unique query parameter url body text",
            ),
        ),
        "/robots-meta" => (
            ok,
            no_headers,
            doc(
                "Robots Meta Directives Page Title Value Here",
                "<meta name=\"description\" content=\"Page with a robots meta tag carrying multiple directives.\">\
                 <meta name=\"robots\" content=\"noindex, nofollow, noarchive, nosnippet\">",
                "<h1>Robots Meta Heading</h1><h2>Sub</h2><p>robots meta body</p>",
            ),
        ),
        "/directive-none" => (
            ok,
            no_headers,
            doc(
                "Robots None Directive Page Title Value Here",
                "<meta name=\"description\" content=\"Page with a robots meta tag set to none.\">\
                 <meta name=\"robots\" content=\"none\">",
                "<h1>Directive None Heading</h1><h2>Sub</h2><p>directive none body</p>",
            ),
        ),
        "/x-robots" => (
            ok,
            "x-robots-tag: noindex\r\n".to_string(),
            plain(
                "X Robots Tag Header Page Title Value Here",
                "X Robots",
                "Meta for the page that sends an x-robots-tag noindex response header.",
                "Sub",
                "the unique x robots tag header body text",
            ),
        ),
        "/external" => (
            ok,
            no_headers,
            doc(
                "External Outlink Page Title Value Goes Here",
                "<meta name=\"description\" content=\"Page linking to an external domain for the external filter.\">",
                "<h1>External Heading</h1><h2>Sub</h2>\
                 <p>see <a href=\"https://example.com/\">example dot com</a></p>",
            ),
        ),
        "/links" => (
            ok,
            no_headers,
            doc(
                "Internal Links Page Title Value Goes Here",
                "<meta name=\"description\" content=\"Page with assorted internal outlinks for the links filters.\">",
                "<h1>Links Heading</h1><h2>Sub</h2>\
                 <p><a href=\"/not-found\">broken</a> \
                 <a href=\"/canonical-other\">redirected</a> \
                 <a href=\"/large\" rel=\"nofollow\">nofollow</a></p>",
            ),
        ),
        "/a11y" => (
            ok,
            no_headers,
            // Live axe violations (best-effort); the A11y* filters are asserted
            // against an injected synthetic page so they stay deterministic.
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>Accessibility Violations Page Title</title><meta name=\"description\" content=\"Page with deliberate accessibility violations.\"></head><body><h1>A11y</h1><img src=\"/x.png\"><input type=\"text\"><a href=\"/y\"></a><button></button><p style=\"color:#cccccc;background:#dddddd\">low contrast</p><h4>skipped heading level</h4></body></html>".to_string(),
        ),
        "/spa" => (
            ok,
            no_headers,
            // Empty shell server-side; client JS injects substantial content so
            // the Chrome-mode SSR diff flags ssr_content_missing.
            format!(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Single Page App Shell Title Value</title><meta name=\"description\" content=\"SPA shell whose content is injected by client side javascript.\"></head><body><div id=\"app\"></div><script>document.getElementById('app').innerHTML='<h1>Rendered Heading</h1><p>{}</p>';</script></body></html>",
                lorem(80)
            ),
        ),
        p if p == LONG_PATH => (
            ok,
            no_headers,
            plain(
                "Over Length URL Path Page Title Value Here",
                "Over Length URL",
                "Meta for the over length url path page used by the url filters.",
                "Sub",
                "the unique over length url body text",
            ),
        ),
        _ => (
            "HTTP/1.1 404 Not Found",
            no_headers,
            "<html><body>unknown</body></html>".to_string(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Crawl + load helper
// ---------------------------------------------------------------------------

fn chrome_available() -> bool {
    [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ]
    .iter()
    .any(|bin| {
        std::process::Command::new(bin)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

fn chrome_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static CHROME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = CHROME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    for name in ["SingletonSocket", "SingletonCookie", "SingletonLock"] {
        let path = std::path::Path::new("/tmp/chromiumoxide-runner").join(name);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    guard
}

fn crawl_and_load(root_url: &str, render_mode: RenderMode, timeout: Duration) -> Vec<PageRecord> {
    let _chrome_guard = matches!(render_mode, RenderMode::Chrome).then(chrome_test_guard);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .unwrap();

    let pool = rt.block_on(async {
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        crate::storage::run_migrations(&pool).await.unwrap();
        pool
    });

    let (tx, rx) = crate::crawl::engine::channel();
    let (cancel, fut) = {
        let mut engine = crate::crawl::engine::CrawlEngine::new();
        engine.start(
            root_url.to_string(),
            tx,
            pool.clone(),
            render_mode,
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
                // The fixture links to other origins on purpose; a filter test
                // must not send requests to them.
                check_resources: false,
            },
        )
    };

    rt.spawn(async move {
        fut.await;
    });

    // Drain the channel until the crawl finishes; the grid reloads a selected
    // crawl from the DB, so we assert against the persisted-and-reloaded records
    // (`load_pages_for_crawl`) rather than the live channel events. This also
    // guards that the loader reconstructs every field the filters read.
    let start = std::time::Instant::now();
    loop {
        let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
            cancel.store(true, Ordering::Relaxed);
            break;
        };
        match rx.recv_timeout(remaining) {
            Ok(CrawlEvent::Finished { .. }) => break,
            Ok(CrawlEvent::Error { url, message }) => eprintln!("crawl error {url}: {message}"),
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

/// Records for conditions a normalized live crawl cannot deterministically
/// produce. Injected into the loaded page set so the pure filter logic is
/// still exercised. See module docs.
fn synthetic_pages(base: &str) -> Vec<PageRecord> {
    let internal = |url: String| PageRecord {
        url,
        status: Some(200),
        is_internal: true,
        is_page: true,
        ..Default::default()
    };
    // Subresources harvested from resource timings: their content type is the
    // basis for the Internal tab's resource-type filters, but they are not
    // navigated documents, so they must not surface in document tabs.
    let with_ct = |path: &str, ct: &str| PageRecord {
        content_type: Some(ct.to_string()),
        is_page: false,
        is_resource: true,
        ..internal(format!("{base}{path}"))
    };
    let with_headers = |path: &str, headers: &[(&str, &str)]| PageRecord {
        headers: headers
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
        ..internal(format!("{base}{path}"))
    };

    let a11y_rules = [
        "image-alt",
        "label",
        "link-name",
        "button-name",
        "color-contrast",
        "html-has-lang",
        "heading-order",
    ];

    vec![
        with_ct("/syn-css", "text/css"),
        with_ct("/syn-js", "application/javascript"),
        with_ct("/syn-img", "image/png"),
        with_ct("/syn-pdf", "application/pdf"),
        with_ct("/syn-other", "application/json"),
        // A Fetch/XHR request harvested from resource timings: no usable
        // content type, identified solely by its initiator.
        PageRecord {
            is_page: false,
            is_resource: true,
            resource_initiator: Some("fetch".to_string()),
            ..internal(format!("{base}/syn-xhr"))
        },
        // A redirect the crawler did not follow: a 3xx with no final URL, which
        // is how two of three redirects arrived on the crawled site. It is a
        // redirect for every purpose, including being kept out of the
        // content-issue rules, despite carrying no `redirect_url`.
        PageRecord {
            status: Some(301),
            ..internal(format!("{base}/syn-3xx"))
        },
        // A followed (non-loop) redirect. The crawler's SSRF guard refuses to
        // follow redirects to loopback/private hosts, so a 127.0.0.1 test server
        // cannot produce a live followed redirect; cover the Redirects filter
        // synthetically. Target is a normal page that does not redirect, so it
        // must NOT be flagged as a redirect loop.
        PageRecord {
            redirect_url: Some(format!("{base}/large")),
            ..internal(format!("{base}/syn-redirect"))
        },
        PageRecord {
            redirect_url: Some(format!("{base}/syn-loop-b")),
            ..internal(format!("{base}/syn-loop-a"))
        },
        PageRecord {
            redirect_url: Some(format!("{base}/syn-loop-a")),
            ..internal(format!("{base}/syn-loop-b"))
        },
        internal(format!("{base}/café")),
        internal(format!("{base}/has space")),
        PageRecord {
            a11y_issues: a11y_rules
                .iter()
                .map(|rule| A11yIssue {
                    rule: rule.to_string(),
                    impact: "serious".to_string(),
                    target: None,
                    html: None,
                })
                .collect(),
            ..internal(format!("{base}/syn-a11y"))
        },
        PageRecord {
            status: None,
            in_sitemap: Some(true),
            sitemap_url: Some(format!("{base}/sitemap.xml")),
            ..internal(format!("{base}/sitemap-orphan"))
        },
        // Response headers are not captured in HTTP render mode, so the
        // header-based filters are exercised against synthetic records.
        with_headers(
            "/syn-secure",
            &[
                ("strict-transport-security", "max-age=31536000"),
                ("content-security-policy", "default-src 'self'"),
                ("x-frame-options", "DENY"),
                ("x-content-type-options", "nosniff"),
                ("referrer-policy", "strict-origin-when-cross-origin"),
            ],
        ),
        with_headers("/syn-xrobots", &[("x-robots-tag", "noindex")]),
        // Mixed content (HTTPS page loading an HTTP subresource) cannot arise on
        // the plain-HTTP coverage origin, so cover the Security/MixedContent
        // filter synthetically.
        PageRecord {
            has_mixed_content: true,
            // Headers, because the Security tab only judges a row whose
            // headers it has: a real page answering 200 always has some, and a
            // fixture without them is not the thing it stands in for.
            headers: vec![("content-type".to_string(), "text/html".to_string())],
            ..internal(format!("{base}/syn-mixed"))
        },
        // External resources/URLs (other origin) are recorded with
        // is_internal == false and surface on the External tab. A normalized
        // crawl of a single 127.0.0.1 origin produces none, so cover it here.
        PageRecord {
            url: "https://cdn.example.net/asset.png".to_string(),
            status: Some(200),
            content_type: Some("image/png".to_string()),
            is_internal: false,
            ..Default::default()
        },
        // An oversized and a broken image, as the post-crawl resource pass
        // records them, with a page referencing both. The pass is off for the
        // coverage crawl (the fixture links off-site on purpose), and a
        // loopback fixture cannot be made to serve a 100 kB image cheaply.
        PageRecord {
            size_bytes: 250_000,
            content_type: Some("image/png".to_string()),
            is_page: false,
            is_resource: true,
            resource_initiator: Some("img".to_string()),
            ..internal(format!("{base}/syn-heavy.png"))
        },
        PageRecord {
            status: Some(404),
            is_page: false,
            is_resource: true,
            resource_initiator: Some("img".to_string()),
            ..internal(format!("{base}/syn-gone.png"))
        },
        PageRecord {
            images: vec![
                ImageRef {
                    src: format!("{base}/syn-heavy.png"),
                    alt: Some("Heavy".to_string()),
                    width: Some(10),
                    height: Some(10),
                    has_alt_attr: true,
                },
                ImageRef {
                    src: format!("{base}/syn-gone.png"),
                    alt: Some("Gone".to_string()),
                    width: Some(10),
                    height: Some(10),
                    has_alt_attr: true,
                },
            ],
            ..internal(format!("{base}/syn-images"))
        },
        // One page carrying a link of every kind the Links sub-filters select.
        // The live `/links` fixture covers broken and nofollow, but a followed
        // redirect cannot be produced against loopback (see `/syn-redirect`),
        // so the whole set is covered here where the destinations are known.
        PageRecord {
            outlinks: vec![
                link(&format!("{base}/not-found"), Some("broken"), None),
                link(&format!("{base}/syn-3xx"), Some("redirected"), None),
                link(&format!("{base}/large"), Some("nofollow"), Some("nofollow")),
                link(&format!("{base}/dup-a"), None, None),
                link("https://example.com/", Some("external"), None),
            ],
            ..internal(format!("{base}/syn-links"))
        },
    ]
}

fn link(dst_url: &str, anchor: Option<&str>, rel: Option<&str>) -> Outlink {
    Outlink {
        dst_url: dst_url.to_string(),
        anchor: anchor.map(|a| a.to_string()),
        rel: rel.map(|r| r.to_string()),
        csr_only: false,
    }
}

// ---------------------------------------------------------------------------
// Expectations
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Mode {
    Both,
    HttpOnly,
    ChromeOnly,
}

struct Expect {
    mode: Mode,
    same_as_all: bool,
    must_match: &'static [&'static str],
    must_not_match: &'static [&'static str],
}

fn both(must_match: &'static [&'static str], must_not_match: &'static [&'static str]) -> Expect {
    Expect {
        mode: Mode::Both,
        same_as_all: false,
        must_match,
        must_not_match,
    }
}
fn http(must_match: &'static [&'static str], must_not_match: &'static [&'static str]) -> Expect {
    Expect {
        mode: Mode::HttpOnly,
        same_as_all: false,
        must_match,
        must_not_match,
    }
}
fn chrome(must_match: &'static [&'static str], must_not_match: &'static [&'static str]) -> Expect {
    Expect {
        mode: Mode::ChromeOnly,
        same_as_all: false,
        must_match,
        must_not_match,
    }
}
fn same_as_all() -> Expect {
    Expect {
        mode: Mode::Both,
        same_as_all: true,
        must_match: &[],
        must_not_match: &[],
    }
}

/// Exhaustive over `IssueFilter` (no wildcard arm): a new variant will not
/// compile until its expected behavior is declared. Shared heading filters
/// (Missing/Duplicate/OverLength/...) are uniform across the title/meta/h1/h2
/// tabs because the fixtures duplicate every heading tag in lockstep, so no
/// per-tab branching is needed beyond `All`.
fn expectation(tab: ResultTab, filter: IssueFilter) -> Expect {
    use IssueFilter as F;
    match filter {
        F::All => match tab {
            // External lists external URLs/resources (is_internal == false);
            // no internal page (e.g. "/") should appear here.
            ResultTab::External => both(&["/asset.png"], &["/"]),
            ResultTab::Images => both(&["/images"], &["/"]),
            _ => both(&["/"], &[]),
        },

        // Internal content-type / indexability
        F::NonIndexable => both(&["/robots-meta", "/not-found", "/server-error"], &["/"]),
        F::Html => both(&["/"], &["/syn-css"]),
        F::Images => both(&["/syn-img"], &["/"]),
        F::Css => both(&["/syn-css"], &["/"]),
        F::JavaScript => both(&["/syn-js"], &["/"]),
        F::Pdf => both(&["/syn-pdf"], &["/"]),
        F::OtherResource => both(&["/syn-other"], &["/", "/syn-xhr"]),
        F::FetchXhr => both(&["/syn-xhr"], &["/", "/syn-other"]),

        // Response codes
        F::Status2xx => both(&["/"], &["/not-found"]),
        F::Status3xx => both(&["/syn-3xx"], &[]),
        F::Status4xx => both(&["/not-found"], &["/"]),
        F::Status5xx => both(&["/server-error"], &["/"]),
        F::Redirects => both(&["/syn-redirect", "/syn-3xx"], &["/"]),
        F::RedirectLoop => both(&["/syn-loop-a"], &["/syn-redirect"]),

        // Titles / meta / headings (uniform across the four heading tabs)
        F::Missing => both(&["/missing-all"], &["/", "/syn-3xx"]),
        F::Duplicate => both(&["/dup-a", "/dup-b"], &["/"]),
        F::OverLength => both(&["/long-all"], &["/"]),
        F::UnderLength => both(&["/short-all"], &["/"]),
        F::OverPixelWidth => both(&["/long-all"], &["/short-all"]),
        F::UnderPixelWidth => both(&["/short-all"], &["/long-all"]),
        F::Multiple => both(&["/multiple-all"], &["/"]),
        F::SameAsH1 => both(&["/title-eq-h1"], &["/"]),

        // Content
        F::ExactDuplicates => both(&["/exact-dup-a", "/exact-dup-b"], &["/"]),
        F::NearDuplicates => both(&["/near-dup-a", "/near-dup-b"], &["/"]),
        F::LowContent => both(&["/low-content"], &["/large"]),
        F::SsrContentMissing => chrome(&["/spa"], &[]),
        F::BlockedByRobots => both(&[], &[]),

        // Images
        F::MissingAltText => both(&["/images"], &[]),
        F::MissingAltAttribute => both(&["/images"], &[]),
        F::AltOver100 => both(&["/images"], &[]),
        F::MissingSizeAttributes => both(&["/images"], &[]),

        // Canonicals
        F::ContainsCanonical => both(&["/canonical-self", "/canonical-other"], &["/missing-all"]),
        F::SelfReferencing => both(&["/canonical-self"], &["/canonical-other"]),
        F::Canonicalised => both(&["/canonical-other"], &["/canonical-self"]),
        F::MissingCanonical => both(&["/missing-all"], &["/canonical-self"]),

        // Hreflang
        F::ContainsHreflang => both(&["/hreflang-a", "/hreflang-b"], &["/"]),
        F::MissingHreflang => both(&["/"], &["/hreflang-a"]),
        F::HreflangMissingReturnTag => both(&["/hreflang-a"], &["/hreflang-d"]),
        F::HreflangInvalidLang => both(&["/hreflang-a"], &[]),
        F::HreflangMissingXDefault => both(&["/hreflang-a"], &["/hreflang-c", "/hreflang-d"]),
        F::HreflangMissingSelfReference => both(&["/hreflang-c"], &["/hreflang-a", "/hreflang-d"]),
        F::HreflangNonCanonical => both(&["/hreflang-a"], &[]),

        // Structured data
        F::HasStructuredData => both(&["/sd-article"], &["/"]),
        F::MissingStructuredData => both(&["/"], &["/sd-article"]),
        F::SdErrors => both(&["/sd-errors"], &["/sd-article"]),
        F::SdWarnings => both(&["/sd-article"], &[]),
        F::JsonLdUrls => both(&["/sd-article"], &["/sd-microdata"]),
        F::MicrodataUrls => both(&["/sd-microdata"], &["/sd-article"]),
        F::ParseErrors => both(&["/sd-errors"], &[]),
        F::SdTypeArticle => both(&["/sd-article"], &["/sd-faq"]),
        F::SdTypeProduct => both(&["/sd-product", "/product-bare"], &["/sd-article"]),
        F::SdTypeFaq => both(&["/sd-faq"], &[]),
        F::SdTypeHowTo => both(&["/sd-howto"], &[]),
        F::SdTypeRecipe => both(&["/sd-recipe"], &[]),
        F::SdTypeVideo => both(&["/sd-video"], &[]),
        F::SdTypeBreadcrumb => both(&["/sd-breadcrumb"], &[]),
        F::SdTypeOrganization => both(&["/sd-organization"], &[]),

        // Accessibility (synthetic page, deterministic in both modes)
        F::A11yImageAlt => both(&["/syn-a11y"], &[]),
        F::A11yLabel => both(&["/syn-a11y"], &[]),
        F::A11yLinkName => both(&["/syn-a11y"], &[]),
        F::A11yButtonName => both(&["/syn-a11y"], &[]),
        F::A11yColorContrast => both(&["/syn-a11y"], &[]),
        F::A11yHtmlHasLang => both(&["/syn-a11y"], &[]),
        F::A11yHeadingOrder => both(&["/syn-a11y"], &[]),

        // Performance (faked metrics; Chrome overwrites them, so HTTP only)
        F::SlowLcp => http(&["/slow-perf"], &[]),
        F::SlowCls => http(&["/slow-perf"], &[]),
        F::SlowFcp => http(&["/slow-perf"], &[]),
        F::SlowTtfb => http(&["/slow-perf"], &[]),

        // Ecommerce
        F::IsProductPage => both(&["/sd-product", "/product-bare"], &["/"]),
        F::MissingPrice => both(&["/product-bare"], &["/sd-product"]),
        F::MissingAvailability => both(&["/product-bare"], &["/sd-product"]),
        F::MissingSku => both(&["/product-bare"], &["/sd-product"]),
        F::MissingGtin => both(&["/product-bare"], &["/sd-product"]),
        F::MissingBrand => both(&["/product-bare"], &["/sd-product"]),
        F::MissingReviewRating => both(&["/product-bare"], &["/sd-product"]),
        F::MissingProductImage => both(&["/product-bare"], &["/sd-product"]),

        // Sitemaps
        F::UrlsInSitemap => both(&["/"], &["/images"]),
        F::UrlsNotInSitemap => both(&["/images"], &["/"]),
        F::SitemapOrphans => both(&["/sitemap-orphan"], &[]),
        F::NonIndexableInSitemap => both(&["/robots-meta"], &[]),
        // The sitemap advertises a URL that 404s, which is the point of the
        // rule: a sitemap should list URLs that answer 200.
        F::SitemapNon200 => both(&["/not-found"], &["/"]),

        // Security
        F::MissingHttps => both(&["/"], &[]),
        F::MissingHsts => both(&["/"], &["/syn-secure"]),
        F::MissingCsp => both(&["/"], &["/syn-secure"]),
        F::MissingFrameGuard => both(&["/"], &["/syn-secure"]),
        F::MissingContentTypeOptions => both(&["/"], &["/syn-secure"]),
        F::MissingReferrerPolicy => both(&["/"], &["/syn-secure"]),
        F::MixedContent => both(&["/syn-mixed"], &["/"]),

        // Markup validity: read off the source, since a parser invents the
        // element when the server omits it.
        F::MissingBodyTag => both(&["/no-body"], &["/"]),

        // URL hygiene
        F::UrlNonAscii => both(&["/café"], &["/"]),
        F::UrlUppercase => both(&["/MixedCase"], &["/"]),
        F::UrlUnderscores => both(&["/under_score"], &["/"]),
        F::UrlMultipleSlashes => both(&["/multi//slash"], &["/"]),
        F::UrlParameters => both(&["/withparam?x=1"], &["/"]),
        F::UrlOverLength => both(&[LONG_PATH], &["/"]),
        F::UrlSpaces => both(&["/has space"], &["/"]),

        // Directives
        F::DirectiveNoindex => both(&["/robots-meta", "/syn-xrobots"], &["/"]),
        F::DirectiveNofollow => both(&["/robots-meta"], &["/"]),
        F::DirectiveNoarchive => both(&["/robots-meta"], &["/"]),
        F::DirectiveNosnippet => both(&["/robots-meta"], &["/"]),
        F::DirectiveNone => both(&["/directive-none"], &["/robots-meta"]),

        // Images the resource pass fetched: their size and status live on the
        // resource row, not on the referencing page.
        F::ImageOver100Kb => both(&["/syn-images"], &["/images"]),
        // No negative: in Chrome mode the fixture's own images are fetched by
        // the browser and recorded through Resource Timing, and they 404, so
        // "/images" really does carry broken images there. The discriminating
        // case is unit-tested in `filter::counting_tests`.
        F::ImageBroken => both(&["/syn-images"], &[]),

        // Links: the tab lists one row per URL, so each sub-filter selects the
        // pages carrying at least one such link. `/external` carries a
        // single external link with anchor text, which is the useful negative
        // for the other four.
        F::LinkBroken => both(&["/links", "/syn-links"], &["/external"]),
        F::LinkRedirected => both(&["/syn-links"], &["/external"]),
        F::LinkNofollow => both(&["/links", "/syn-links"], &["/external"]),
        F::LinkNoAnchorText => both(&["/syn-links"], &["/external"]),
        F::LinkExternal => both(&["/external", "/syn-links"], &["/links"]),

        // Overview issue/priority + depth: page-level no-ops that return the
        // tab's full base set.
        F::IssueTypeError
        | F::IssueTypeOpportunity
        | F::IssueTypeWarning
        | F::PriorityHigh
        | F::PriorityMedium
        | F::PriorityLow
        | F::DepthShallow
        | F::DepthMedium
        | F::DepthDeep
        | F::ChangeAdded
        | F::ChangeRemoved
        | F::ChangeChanged => same_as_all(),
    }
}

// ---------------------------------------------------------------------------
// Driver
// ---------------------------------------------------------------------------

fn path_of(url: &str) -> String {
    let after = url.split_once("://").map(|x| x.1).unwrap_or(url);
    match after.find('/') {
        Some(i) => after[i..].to_string(),
        None => "/".to_string(),
    }
}

fn run_coverage(render_mode: RenderMode) {
    // The HTTP and Chrome passes each drive a full crawl; running them
    // concurrently (cargo's default) starves the Chrome pass and makes it
    // flaky, so serialize the two against each other.
    static COVERAGE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _coverage_guard = COVERAGE_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let (handle, port, stop) = spawn_site();
    let base = format!("http://127.0.0.1:{port}");
    let root_url = format!("{base}/");
    let root_origin = url::Url::parse(&root_url)
        .ok()
        .map(|u| u.origin().ascii_serialization());

    let timeout = match render_mode {
        RenderMode::Http => Duration::from_secs(60),
        RenderMode::Chrome => Duration::from_secs(180),
    };
    let mut pages = crawl_and_load(&root_url, render_mode, timeout);
    pages.extend(synthetic_pages(&base));

    stop.store(true, Ordering::Relaxed);
    let _ = handle.join();

    assert!(
        pages.len() > 30,
        "expected the full coverage site to be crawled, only got {} pages",
        pages.len()
    );

    let is_http = matches!(render_mode, RenderMode::Http);
    let mut failures: Vec<String> = Vec::new();

    for &tab in ResultTab::ALL {
        let all_paths: HashSet<String> = matching_urls(tab, IssueFilter::All, &pages)
            .iter()
            .map(|u| path_of(u))
            .collect();

        for &filter in filters_for_tab(tab) {
            let expect = expectation(tab, filter);

            let skip = match expect.mode {
                Mode::Both => false,
                Mode::HttpOnly => !is_http,
                Mode::ChromeOnly => is_http,
            };
            if skip {
                continue;
            }

            let matched: HashSet<String> = matching_urls(tab, filter, &pages)
                .iter()
                .map(|u| path_of(u))
                .collect();

            if expect.same_as_all {
                if matched != all_paths {
                    failures.push(format!(
                        "{tab:?}/{filter:?}: expected same set as All ({} vs {})",
                        all_paths.len(),
                        matched.len()
                    ));
                }
                continue;
            }

            for want in expect.must_match {
                if !matched.contains(*want) {
                    failures.push(format!(
                        "{tab:?}/{filter:?}: expected to match {want:?}, matched {matched:?}"
                    ));
                }
            }
            for forbid in expect.must_not_match {
                if matched.contains(*forbid) {
                    failures.push(format!(
                        "{tab:?}/{filter:?}: expected NOT to match {forbid:?}, but it did"
                    ));
                }
            }
        }

        // The badge must be a pure function of the same
        // per-filter data the sub-filter buttons render, so it can never claim
        // more issues than the tab holds or disagree with the `All` count.
        let counts = tab_filter_counts_for_test(tab, &pages, root_origin.as_deref());
        let all_count = counts
            .filter_counts
            .iter()
            .find(|(f, _)| *f == IssueFilter::All)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        if all_count != counts.badge.total {
            failures.push(format!(
                "{tab:?}: All count {all_count} != badge.total {}",
                counts.badge.total
            ));
        }
        if counts.badge.errors + counts.badge.warnings > counts.badge.total {
            failures.push(format!(
                "{tab:?}: errors {} + warnings {} exceed total {}",
                counts.badge.errors, counts.badge.warnings, counts.badge.total
            ));
        }
        // On page-listing tabs the per-filter count is exactly the number of
        // pages the filter selects, so it must equal what `matching_urls`
        // returns. Flattened tabs (and Overview/Changes) count rows or entries
        // rather than pages, so that equality does not apply to them.
        if !tab_is_flattened(tab) {
            for &(filter, count) in &counts.filter_counts {
                let expected = matching_urls(tab, filter, &pages).len();
                if count != expected {
                    failures.push(format!(
                        "{tab:?}/{filter:?}: filter count {count} != matching_urls {expected}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "filter coverage failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_filters_http() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=warn".into()),
        )
        .with_test_writer()
        .try_init();
    run_coverage(RenderMode::Http);
}

#[test]
fn all_filters_chrome() {
    if !chrome_available() {
        eprintln!("skipping all_filters_chrome: no chrome binary on PATH");
        return;
    }
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=warn".into()),
        )
        .with_test_writer()
        .try_init();
    run_coverage(RenderMode::Chrome);
}
