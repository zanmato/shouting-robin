use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SitemapUrl {
    pub sitemap_url: String,
    pub page_url: String,
    /// The entry's `<lastmod>`, verbatim, when it has one.
    pub lastmod: Option<String>,
    /// The `xhtml:link rel="alternate"` annotations on the entry, as
    /// (hreflang, URL). A sitemap is one of the three places Google accepts
    /// hreflang, and the only one that needs no change to the page itself.
    pub hreflang: Vec<(String, String)>,
}

pub async fn discover_sitemaps(root_url: &str) -> Vec<String> {
    let mut sitemap_urls = Vec::new();

    if let Ok(robots_sitemaps) = fetch_robots_sitemaps(root_url).await {
        sitemap_urls.extend(robots_sitemaps);
    }

    let base = root_url.trim_end_matches('/');
    for probe in &["/sitemap.xml", "/sitemap_index.xml"] {
        let url = format!("{base}{probe}");
        if !sitemap_urls.contains(&url) {
            sitemap_urls.push(url);
        }
    }

    sitemap_urls
}

pub async fn fetch_sitemap_urls(sitemap_urls: &[String], max_depth: usize) -> Vec<SitemapUrl> {
    let mut results = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    for sitemap_url in sitemap_urls {
        expand_sitemap(
            sitemap_url,
            sitemap_url,
            max_depth,
            &mut visited,
            &mut results,
        )
        .await;
    }

    results
}

async fn expand_sitemap(
    root_sitemap: &str,
    url: &str,
    depth_left: usize,
    visited: &mut HashSet<String>,
    results: &mut Vec<SitemapUrl>,
) {
    if depth_left == 0 || visited.contains(url) {
        return;
    }
    visited.insert(url.to_string());

    let Ok(body) = fetch_url(url).await else {
        return;
    };

    if let Some(child_sitemaps) = parse_sitemap_index(&body) {
        for child in child_sitemaps {
            Box::pin(expand_sitemap(
                root_sitemap,
                &child,
                depth_left.saturating_sub(1),
                visited,
                results,
            ))
            .await;
        }
        return;
    }

    if let Some(entries) = parse_urlset(&body) {
        for entry in entries {
            // Stored in the same spelling the crawler records pages under,
            // so a `<loc>` with a fragment, an uppercase host or an explicit
            // default port still matches its page instead of becoming an
            // orphan.
            let page_url = crate::crawl::url_norm::normalize_url(&entry.loc)
                .unwrap_or_else(|| entry.loc.trim().to_string());
            results.push(SitemapUrl {
                sitemap_url: root_sitemap.to_string(),
                page_url,
                lastmod: entry.lastmod,
                hreflang: entry.hreflang,
            });
        }
    }
}

async fn fetch_robots_sitemaps(root_url: &str) -> Result<Vec<String>, reqwest::Error> {
    let base = root_url.trim_end_matches('/');
    let robots_url = format!("{base}/robots.txt");
    let body = fetch_url(&robots_url).await?;

    let mut sitemaps = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if let Some(url) = line.strip_prefix("Sitemap:") {
            let url = url.trim().to_string();
            if !url.is_empty() {
                sitemaps.push(url);
            }
        } else if let Some(url) = line.strip_prefix("sitemap:") {
            let url = url.trim().to_string();
            if !url.is_empty() {
                sitemaps.push(url);
            }
        }
    }
    Ok(sitemaps)
}

async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let bytes = client.get(url).send().await?.bytes().await?;
    Ok(decode_sitemap_body(&bytes))
}

/// Sitemaps are commonly published gzipped, as `sitemap.xml.gz` served with
/// `Content-Type: application/gzip`. That is a compressed *payload*, not a
/// transfer encoding, so no HTTP client inflates it for us and reading the
/// response as text yields binary that parses as zero URLs. Detect the gzip
/// magic number and inflate before decoding.
fn decode_sitemap_body(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(bytes);
        let mut decoded = String::new();
        match decoder.read_to_string(&mut decoded) {
            Ok(_) => return decoded,
            Err(e) => {
                tracing::warn!(error=%e, "failed to inflate gzipped sitemap");
                return String::new();
            }
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

fn parse_sitemap_index(xml: &str) -> Option<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut in_loc = false;
    let mut is_index = false;
    let mut urls = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let local = e.local_name();
                let name: &[u8] = local.as_ref();
                if name == b"sitemapindex" {
                    is_index = true;
                } else if name == b"urlset" {
                    return None;
                } else if name == b"loc" {
                    in_loc = true;
                }
            }
            Ok(Event::End(_)) => {
                in_loc = false;
            }
            Ok(Event::Text(e)) if in_loc => {
                if let Ok(text) = e.unescape() {
                    let url = text.trim().to_string();
                    if !url.is_empty() {
                        urls.push(url);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    if is_index && !urls.is_empty() {
        Some(urls)
    } else {
        None
    }
}

/// One `<url>` entry of a sitemap: its `<loc>`, its `<lastmod>` and any
/// `xhtml:link rel="alternate"` annotations on it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UrlsetEntry {
    pub loc: String,
    pub lastmod: Option<String>,
    pub hreflang: Vec<(String, String)>,
}

/// Parses a `<urlset>`. `None` when the document is not one.
fn parse_urlset(xml: &str) -> Option<Vec<UrlsetEntry>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut entries: Vec<UrlsetEntry> = Vec::new();
    let mut field: Option<Field> = None;
    // The entry being read. A urlset is a flat list of <url> elements, so a
    // <lastmod> or an alternate belongs to the <loc> most recently seen.
    let mut current: Option<UrlsetEntry> = None;
    let mut buf = Vec::new();

    enum Field {
        Loc,
        LastMod,
    }

    loop {
        let event = reader.read_event_into(&mut buf);
        match event {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local = e.local_name();
                match local.as_ref() {
                    b"loc" => field = Some(Field::Loc),
                    b"lastmod" => field = Some(Field::LastMod),
                    b"link" => {
                        field = None;
                        if let Some(entry) = current.as_mut()
                            && let Some((lang, href)) = link_alternate(e)
                        {
                            entry.hreflang.push((lang, href));
                        }
                    }
                    b"url" => {
                        field = None;
                        current = Some(UrlsetEntry::default());
                    }
                    _ => field = None,
                }
            }
            Ok(Event::End(ref e)) => {
                // Closing a <url> flushes the entry it described. An entry with
                // no <loc> describes nothing and is dropped.
                if e.local_name().as_ref() == b"url"
                    && let Some(entry) = current.take()
                    && !entry.loc.is_empty()
                {
                    entries.push(entry);
                }
                field = None;
            }
            Ok(Event::Text(ref e)) => {
                let Ok(text) = e.unescape() else {
                    buf.clear();
                    continue;
                };
                let text = text.trim().to_string();
                if text.is_empty() {
                    buf.clear();
                    continue;
                }
                if let Some(entry) = current.as_mut() {
                    match field {
                        Some(Field::Loc) => entry.loc = text,
                        Some(Field::LastMod) => entry.lastmod = Some(text),
                        None => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    if let Some(entry) = current.take()
        && !entry.loc.is_empty()
    {
        entries.push(entry);
    }

    if entries.is_empty() {
        None
    } else {
        Some(entries)
    }
}

/// The (hreflang, href) of an `xhtml:link rel="alternate"` element, if it is
/// one. Sitemaps also use `<xhtml:link>` for other rel values.
fn link_alternate(e: &quick_xml::events::BytesStart<'_>) -> Option<(String, String)> {
    let mut rel = None;
    let mut lang = None;
    let mut href = None;
    for attribute in e.attributes().flatten() {
        let value = String::from_utf8_lossy(&attribute.value).trim().to_string();
        match attribute.key.local_name().as_ref() {
            b"rel" => rel = Some(value),
            b"hreflang" => lang = Some(value),
            b"href" => href = Some(value),
            _ => {}
        }
    }
    let (rel, lang, href) = (rel?, lang?, href?);
    if !rel.eq_ignore_ascii_case("alternate") || lang.is_empty() || href.is_empty() {
        return None;
    }
    Some((lang, href))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_urlset() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url><loc>https://example.com/</loc></url>
            <url><loc>https://example.com/about</loc></url>
            <url><loc>https://example.com/contact</loc></url>
        </urlset>"#;
        let urls = parse_urlset(xml).unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(urls[0].loc, "https://example.com/");
        assert_eq!(urls[2].loc, "https://example.com/contact");
    }

    #[test]
    fn parses_lastmod_alongside_each_loc() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url><loc>https://example.com/</loc><lastmod>2026-08-01</lastmod></url>
            <url><loc>https://example.com/about</loc></url>
            <url>
                <lastmod>2026-07-15T09:30:00+02:00</lastmod>
                <loc>https://example.com/contact</loc>
            </url>
        </urlset>"#;
        let urls = parse_urlset(xml).unwrap();
        let pairs: Vec<(&str, Option<&str>)> = urls
            .iter()
            .map(|entry| (entry.loc.as_str(), entry.lastmod.as_deref()))
            .collect();
        assert_eq!(
            pairs,
            vec![
                ("https://example.com/", Some("2026-08-01")),
                ("https://example.com/about", None),
                // Order within a <url> does not matter: the entry is flushed
                // when it closes, not when its loc is read.
                (
                    "https://example.com/contact",
                    Some("2026-07-15T09:30:00+02:00")
                ),
            ]
        );
    }

    #[test]
    fn parses_sitemap_hreflang_alternates() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
                xmlns:xhtml="http://www.w3.org/1999/xhtml">
            <url>
                <loc>https://example.com/se/</loc>
                <xhtml:link rel="alternate" hreflang="sv" href="https://example.com/se/"/>
                <xhtml:link rel="alternate" hreflang="de" href="https://example.com/de/"/>
                <xhtml:link rel="next" href="https://example.com/se/2"/>
            </url>
            <url><loc>https://example.com/de/</loc></url>
        </urlset>"#;
        let urls = parse_urlset(xml).unwrap();
        assert_eq!(
            urls[0].hreflang,
            vec![
                ("sv".to_string(), "https://example.com/se/".to_string()),
                ("de".to_string(), "https://example.com/de/".to_string()),
            ],
            "only rel=alternate links with an hreflang are hreflang tags"
        );
        assert!(urls[1].hreflang.is_empty());
    }

    #[test]
    fn parses_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap><loc>https://example.com/sitemap-posts.xml</loc></sitemap>
            <sitemap><loc>https://example.com/sitemap-pages.xml</loc></sitemap>
        </sitemapindex>"#;
        let urls = parse_sitemap_index(xml).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com/sitemap-posts.xml");
    }

    #[test]
    fn returns_none_for_non_sitemap_xml() {
        let html = "<html><body>Hello</body></html>";
        assert!(parse_urlset(html).is_none());
        assert!(parse_sitemap_index(html).is_none());
    }

    #[test]
    fn extracts_robots_sitemaps() {
        let robots = "User-agent: *\nDisallow: /admin/\n\nSitemap: https://example.com/sitemap.xml\nSitemap: https://example.com/sitemap-2.xml\n";
        let mut sitemaps = Vec::new();
        for line in robots.lines() {
            let line = line.trim();
            if let Some(url) = line.strip_prefix("Sitemap:") {
                let url = url.trim().to_string();
                if !url.is_empty() {
                    sitemaps.push(url);
                }
            }
        }
        assert_eq!(sitemaps.len(), 2);
        assert_eq!(sitemaps[0], "https://example.com/sitemap.xml");
    }

    #[test]
    fn handles_empty_urlset() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        </urlset>"#;
        assert!(parse_urlset(xml).is_none());
    }

    #[test]
    fn inflates_a_gzipped_sitemap() {
        use std::io::Write;
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url><loc>https://example.com/a</loc></url>
            <url><loc>https://example.com/b</loc></url>
        </urlset>"#;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(xml.as_bytes()).expect("gzip the sitemap");
        let gzipped = encoder.finish().expect("finish gzip");

        assert_ne!(
            String::from_utf8_lossy(&gzipped),
            xml,
            "the fixture should really be compressed"
        );

        let body = decode_sitemap_body(&gzipped);
        let urls = parse_urlset(&body).expect("gzipped sitemap should parse");
        let locs: Vec<&str> = urls.iter().map(|entry| entry.loc.as_str()).collect();
        assert_eq!(locs, vec!["https://example.com/a", "https://example.com/b"]);
    }

    #[test]
    fn plain_xml_is_left_alone() {
        let xml = "<urlset><url><loc>https://example.com/a</loc></url></urlset>";
        assert_eq!(decode_sitemap_body(xml.as_bytes()), xml);
    }
}
