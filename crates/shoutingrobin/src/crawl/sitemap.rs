use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

#[derive(Debug, Clone)]
pub struct SitemapUrl {
    pub sitemap_url: String,
    pub page_url: String,
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

    let body = match fetch_url(url).await {
        Ok(b) => b,
        Err(_) => return,
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

    if let Some(page_urls) = parse_urlset(&body) {
        for page_url in page_urls {
            results.push(SitemapUrl {
                sitemap_url: root_sitemap.to_string(),
                page_url,
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
    client.get(url).send().await?.text().await
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

fn parse_urlset(xml: &str) -> Option<Vec<String>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut in_loc = false;
    let mut urls = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let local = e.local_name();
                let name: &[u8] = local.as_ref();
                in_loc = name == b"loc";
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

    if urls.is_empty() { None } else { Some(urls) }
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
        assert_eq!(urls[0], "https://example.com/");
        assert_eq!(urls[2], "https://example.com/contact");
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
}
