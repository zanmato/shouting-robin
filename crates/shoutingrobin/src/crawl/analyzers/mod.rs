mod rich_results;
mod schema_org;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use scraper::{Html, Selector};

use crate::crawl::event::{
    EcommerceAudit, ImageRef, Outlink, PageRecord, SdFormat, SdIssue, SdItem, SdSeverity,
};
use crate::crawl::font_metrics::{
    META_DESCRIPTION_FONT_SIZE_PX, TITLE_FONT_SIZE_PX, text_pixel_width,
};

pub fn analyze_html(record: &mut PageRecord, html: &str, content_selector: &str) {
    let doc = Html::parse_document(html);

    record.title = select_text(&doc, "title");
    record.meta_description = select_attr(&doc, r#"meta[name="description"]"#, "content");
    record.h1 = select_text(&doc, "h1");
    record.h2 = select_text(&doc, "h2");
    record.title_2 = select_nth_text(&doc, "title", 1);
    record.meta_description_2 = select_nth_attr(&doc, r#"meta[name="description"]"#, "content", 1);
    record.h1_2 = select_nth_text(&doc, "h1", 1);
    record.h2_2 = select_nth_text(&doc, "h2", 1);
    record.canonical = select_attr(&doc, r#"link[rel="canonical"]"#, "href");
    // The robots meta name and its directives are case-insensitive per the
    // HTML spec (e.g. `<meta name="ROBOTS" content="NOINDEX">`), so match the
    // name with the CSS case-insensitive flag rather than the literal "robots".
    record.robots = select_attr(&doc, r#"meta[name="robots" i]"#, "content");
    record.content_type = record
        .content_type
        .clone()
        .or_else(|| Some("text/html".into()));
    let content_text = if content_selector.is_empty() {
        extract_body_text(&doc)
    } else {
        extract_selector_text(&doc, content_selector).unwrap_or_else(|| extract_body_text(&doc))
    };
    record.word_count = Some(
        content_text
            .split_whitespace()
            .filter(|w| w.chars().any(|c| c.is_alphabetic()))
            .count() as u32,
    );
    record.content_hash = Some(compute_content_hash(&content_text));
    record.simhash = Some(compute_simhash(&content_text));
    record.title_count = count_elements(&doc, "title");
    record.h1_count = count_elements(&doc, "h1");
    record.h2_count = count_elements(&doc, "h2");

    record.title_pixel_width = record
        .title
        .as_ref()
        .map(|t| text_pixel_width(t, TITLE_FONT_SIZE_PX));
    record.meta_description_pixel_width = record
        .meta_description
        .as_ref()
        .map(|d| text_pixel_width(d, META_DESCRIPTION_FONT_SIZE_PX));

    extract_perf_metrics(&doc, record);
    extract_hreflang(&doc, record);
    extract_structured_data(&doc, record);
    extract_microdata(&doc, record);
    extract_images(&doc, record);
    extract_anchors(&doc, record);
    extract_og_type(&doc, record);
    extract_mixed_content(&doc, record);
    compute_ecommerce_audit(record);
}

/// Flags mixed content: an HTTPS page that pulls in a subresource over plain
/// HTTP (script, stylesheet, image, iframe, media, …). Browsers block active
/// mixed content and warn on passive, so it undermines the page's security.
/// Protocol-relative (`//host/...`) URLs inherit HTTPS and are not flagged.
fn extract_mixed_content(doc: &Html, record: &mut PageRecord) {
    if !record.url.starts_with("https://") {
        return;
    }
    const SUBRESOURCES: &[(&str, &str)] = &[
        ("script[src]", "src"),
        ("img[src]", "src"),
        ("iframe[src]", "src"),
        ("audio[src]", "src"),
        ("video[src]", "src"),
        ("source[src]", "src"),
        ("track[src]", "src"),
        ("embed[src]", "src"),
        ("object[data]", "data"),
        (r#"link[rel="stylesheet" i][href]"#, "href"),
    ];
    for (selector, attr) in SUBRESOURCES {
        let Ok(sel) = Selector::parse(selector) else {
            continue;
        };
        for el in doc.select(&sel) {
            if let Some(value) = el.value().attr(attr)
                && value
                    .trim_start()
                    .to_ascii_lowercase()
                    .starts_with("http://")
            {
                record.has_mixed_content = true;
                return;
            }
        }
    }
}

/// Compares the raw server-rendered HTML against the already-analyzed rendered
/// DOM (`record`) to detect pages whose content only appears after client-side
/// JavaScript. Call after `analyze_html` so the rendered fields are populated.
pub fn analyze_ssr(record: &mut PageRecord, raw_html: &str, content_selector: &str) {
    const MIN_RENDERED_WORDS: u32 = 50;
    const SSR_RATIO_THRESHOLD: f32 = 0.5;

    let doc = Html::parse_document(raw_html);

    if is_meta_refresh(&doc) {
        return;
    }

    let ssr_h1 = select_text(&doc, "h1");
    let content_text = if content_selector.is_empty() {
        extract_body_text(&doc)
    } else {
        extract_selector_text(&doc, content_selector).unwrap_or_else(|| extract_body_text(&doc))
    };
    let ssr_word_count = content_text
        .split_whitespace()
        .filter(|w| w.chars().any(|c| c.is_alphabetic()))
        .count() as u32;

    let rendered_words = record.word_count.unwrap_or(0);
    let low_content = rendered_words >= MIN_RENDERED_WORDS
        && (ssr_word_count as f32) < rendered_words as f32 * SSR_RATIO_THRESHOLD;
    let h1_only_after_render = record.h1.as_deref().is_some_and(|h1| !h1.is_empty())
        && ssr_h1.as_deref().unwrap_or("").is_empty();

    record.ssr_word_count = Some(ssr_word_count);
    record.ssr_h1 = ssr_h1;
    record.ssr_content_missing = Some(low_content || h1_only_after_render);

    let ssr_link_dsts: std::collections::HashSet<String> = {
        let Ok(sel) = Selector::parse("a[href]") else {
            return;
        };
        let Some(base) = url::Url::parse(&record.url).ok() else {
            return;
        };
        doc.select(&sel)
            .filter_map(|el| {
                let href = el.value().attr("href")?;
                resolve_href(&base, href)
            })
            .collect()
    };
    for link in &mut record.outlinks {
        if !ssr_link_dsts.contains(&link.dst_url) {
            link.csr_only = true;
        }
    }
}

fn is_meta_refresh(doc: &Html) -> bool {
    let Ok(sel) = Selector::parse(r#"meta[http-equiv]"#) else {
        return false;
    };
    doc.select(&sel).any(|el| {
        el.value()
            .attr("http-equiv")
            .is_some_and(|v| v.eq_ignore_ascii_case("refresh"))
    })
}

fn extract_perf_metrics(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse(r#"script#__sr_metrics"#) else {
        return;
    };
    let Some(el) = doc.select(&sel).next() else {
        return;
    };
    let text: String = el.text().collect();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) else {
        return;
    };
    if let Some(v) = value.get("ttfb").and_then(|v| v.as_u64()) {
        record.ttfb_ms = Some(v);
    }
    if let Some(v) = value.get("lcp").and_then(|v| v.as_u64()) {
        record.lcp_ms = Some(v);
    }
    if let Some(v) = value.get("cls").and_then(|v| v.as_f64()) {
        record.cls = Some(v);
    }
    if let Some(v) = value.get("fcp").and_then(|v| v.as_u64()) {
        record.fcp_ms = Some(v);
    }
}

fn extract_hreflang(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse(r#"link[rel="alternate"][hreflang]"#) else {
        return;
    };
    for el in doc.select(&sel) {
        let Some(lang) = el.value().attr("hreflang") else {
            continue;
        };
        let href = el.value().attr("href").unwrap_or("").trim();
        if !lang.is_empty() && !href.is_empty() {
            record
                .hreflang_tags
                .push((lang.trim().to_string(), href.to_string()));
        }
    }
}

fn extract_structured_data(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse(r#"script[type="application/ld+json"]"#) else {
        return;
    };
    for el in doc.select(&sel) {
        let text: String = el.text().collect();
        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }
        record.sd_jsonld_count = record.sd_jsonld_count.saturating_add(1);
        match serde_json::from_str::<serde_json::Value>(trimmed) {
            Ok(value) => {
                extract_schema_objects(&value, trimmed, SdFormat::JsonLd, record);
            }
            Err(_) => {
                record.sd_errors = record.sd_errors.saturating_add(1);
            }
        }
    }
}

fn extract_schema_objects(
    value: &serde_json::Value,
    raw_json: &str,
    format: SdFormat,
    record: &mut PageRecord,
) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_schema_objects(item, raw_json, format, record);
            }
        }
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::String(t)) = map.get("@type") {
                let normalized = schema_org::normalize_type(t).to_string();
                if !record.sd_types.contains(&normalized) {
                    record.sd_types.push(normalized.clone());
                }

                if !schema_org::is_valid_type(t) {
                    record.sd_issues.push(SdIssue {
                        severity: SdSeverity::Error,
                        type_name: normalized.clone(),
                        code: "unknown-type".into(),
                        message: format!("Unknown schema.org type '{normalized}'"),
                    });
                    record.sd_errors = record.sd_errors.saturating_add(1);
                }

                let issues = rich_results::validate_type(t, map);
                for issue in &issues {
                    if issue.severity == SdSeverity::Error {
                        record.sd_errors = record.sd_errors.saturating_add(1);
                    } else {
                        record.sd_warnings = record.sd_warnings.saturating_add(1);
                    }
                }
                record.sd_issues.extend(issues);

                record.sd_items.push(SdItem {
                    format,
                    type_name: normalized,
                    raw_json: raw_json.to_string(),
                });
            }
            if let Some(graph) = map.get("@graph") {
                extract_schema_objects(graph, raw_json, format, record);
            }
        }
        _ => {}
    }
}

fn extract_microdata(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse("[itemscope][itemtype]") else {
        return;
    };
    for el in doc.select(&sel) {
        let Some(type_url) = el.value().attr("itemtype") else {
            continue;
        };
        let type_name = type_url
            .trim()
            .rsplit('/')
            .next()
            .unwrap_or(type_url)
            .to_string();
        if type_name.is_empty() {
            continue;
        }
        record.sd_microdata_count = record.sd_microdata_count.saturating_add(1);

        if !record.sd_types.contains(&type_name) {
            record.sd_types.push(type_name.clone());
        }

        if !schema_org::is_valid_type(&type_name) {
            record.sd_issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: type_name.clone(),
                code: "unknown-type".into(),
                message: format!("Unknown schema.org type '{type_name}'"),
            });
            record.sd_errors = record.sd_errors.saturating_add(1);
        }

        record.sd_items.push(SdItem {
            format: SdFormat::Microdata,
            type_name,
            raw_json: String::new(),
        });
    }
}

fn extract_images(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse("img") else {
        return;
    };
    for el in doc.select(&sel) {
        let Some(src) = el.value().attr("src") else {
            continue;
        };
        let src = src.trim().to_string();
        if src.is_empty() {
            continue;
        }
        let has_alt_attr = el.value().attr("alt").is_some();
        let alt = el.value().attr("alt").map(|a| a.to_string());
        let width = el.value().attr("width").and_then(|w| w.parse().ok());
        let height = el.value().attr("height").and_then(|h| h.parse().ok());

        record.images.push(ImageRef {
            src,
            alt,
            width,
            height,
            has_alt_attr,
        });
    }
}

fn resolve_href(base_url: &url::Url, href: &str) -> Option<String> {
    let resolved = base_url.join(href).ok()?;
    let dst = resolved.to_string();
    if dst.starts_with('#') || dst.starts_with("javascript:") || dst.starts_with("mailto:") {
        return None;
    }
    Some(dst)
}

fn extract_anchors(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse("a[href]") else {
        return;
    };
    let Some(base) = url::Url::parse(&record.url).ok() else {
        return;
    };
    for el in doc.select(&sel) {
        let Some(href) = el.value().attr("href") else {
            continue;
        };
        let Some(dst) = resolve_href(&base, href) else {
            continue;
        };
        let anchor: String = el.text().collect::<Vec<_>>().join(" ");
        let anchor = if anchor.trim().is_empty() {
            None
        } else {
            Some(anchor.split_whitespace().collect::<Vec<_>>().join(" "))
        };
        let rel = el.value().attr("rel").map(|r| r.to_string());
        record.outlinks.push(Outlink {
            dst_url: dst,
            anchor,
            rel,
            csr_only: false,
        });
    }
}

fn extract_og_type(doc: &Html, record: &mut PageRecord) {
    record.og_type = select_attr(doc, r#"meta[property="og:type"]"#, "content");
}

fn compute_ecommerce_audit(record: &mut PageRecord) {
    let is_product = record.sd_types.iter().any(|t| t == "Product")
        || record
            .og_type
            .as_deref()
            .is_some_and(|og| og == "product" || og == "product.item");

    if !is_product {
        return;
    }

    let mut audit = EcommerceAudit::default();

    for item in &record.sd_items {
        if item.type_name != "Product" || item.raw_json.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&item.raw_json) else {
            continue;
        };
        let Some(map) = value.as_object() else {
            continue;
        };

        if map.get("image").is_some_and(|v| !is_empty_value(v)) {
            audit.has_image = true;
        }
        if map.get("description").is_some_and(|v| !is_empty_value(v)) {
            audit.has_description = true;
        }
        if map.get("review").is_some_and(|v| !is_empty_value(v))
            || map
                .get("aggregateRating")
                .is_some_and(|v| !is_empty_value(v))
        {
            audit.has_review_or_rating = true;
        }
        if let Some(brand) = map.get("brand") {
            audit.brand = extract_brand_name(brand);
        }
        if let Some(sku) = map.get("sku").and_then(|v| v.as_str()) {
            audit.sku = Some(sku.to_string());
        }
        let gtin_key = ["gtin", "gtin13", "gtin12", "gtin8", "gtin14"]
            .iter()
            .find(|&&k| map.contains_key(k));
        if let Some(key) = gtin_key {
            audit.gtin = map
                .get(*key)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }

        if let Some(offers) = map.get("offers") {
            extract_offers(offers, &mut audit);
        }
    }

    record.ecommerce = Some(audit);
}

fn is_empty_value(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => true,
        serde_json::Value::String(s) => s.trim().is_empty(),
        serde_json::Value::Array(arr) => arr.is_empty(),
        serde_json::Value::Object(obj) => obj.is_empty(),
        _ => false,
    }
}

fn extract_brand_name(brand: &serde_json::Value) -> Option<String> {
    match brand {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => map
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn extract_offers(offers: &serde_json::Value, audit: &mut EcommerceAudit) {
    let offer_maps = match offers {
        serde_json::Value::Object(map) => vec![map],
        serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_object()).collect(),
        _ => return,
    };

    for map in &offer_maps {
        if audit.price.is_none() {
            audit.price = map
                .get("price")
                .and_then(|v| v.as_f64())
                .map(|p| p.to_string())
                .or_else(|| {
                    map.get("price")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                });
        }
        if audit.currency.is_none() {
            audit.currency = map
                .get("priceCurrency")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
        if audit.availability.is_none() {
            audit.availability = map
                .get("availability")
                .and_then(|v| v.as_str())
                .and_then(|s| s.rsplit('/').next().map(|part| part.to_ascii_lowercase()));
        }
        if audit.sku.is_none() {
            audit.sku = map
                .get("sku")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }
}

fn count_elements(doc: &Html, selector: &str) -> u32 {
    Selector::parse(selector)
        .ok()
        .map(|sel| doc.select(&sel).count() as u32)
        .unwrap_or(0)
}

fn select_text(doc: &Html, selector: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel).next().map(|el| {
        let text: String = el.text().collect::<Vec<_>>().join(" ");
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    })
}

fn select_nth_text(doc: &Html, selector: &str, index: usize) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel).nth(index).map(|el| {
        let text: String = el.text().collect::<Vec<_>>().join(" ");
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    })
}

fn select_attr(doc: &Html, selector: &str, attr: &str) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel)
        .next()
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_string())
}

fn select_nth_attr(doc: &Html, selector: &str, attr: &str, index: usize) -> Option<String> {
    let sel = Selector::parse(selector).ok()?;
    doc.select(&sel)
        .nth(index)
        .and_then(|el| el.value().attr(attr))
        .map(|s| s.trim().to_string())
}

fn extract_body_text(doc: &Html) -> String {
    let Ok(body_sel) = Selector::parse("body") else {
        return String::new();
    };
    let Some(body) = doc.select(&body_sel).next() else {
        return String::new();
    };
    let mut out = String::new();
    let mut skip_depth: usize = 0;
    for edge in body.traverse() {
        match edge {
            ego_tree::iter::Edge::Open(node) => {
                if let scraper::node::Node::Element(el) = node.value() {
                    let tag = el.name();
                    if (tag == "script" || tag == "style" || tag == "noscript") && skip_depth == 0 {
                        skip_depth = 1;
                        continue;
                    }
                    if skip_depth > 0 {
                        skip_depth += 1;
                        continue;
                    }
                }
                if skip_depth == 0
                    && let scraper::node::Node::Text(t) = node.value()
                    && !t.trim().is_empty()
                {
                    if !out.is_empty() {
                        out.push(' ');
                    }
                    out.push_str(t.trim());
                }
            }
            ego_tree::iter::Edge::Close(node) => {
                if skip_depth > 0 {
                    if let scraper::node::Node::Element(_) = node.value() {
                        skip_depth -= 1;
                    }
                } else if let scraper::node::Node::Element(el) = node.value()
                    && is_block_element(el.name())
                    && !out.is_empty()
                    && !out.trim_end().ends_with('.')
                    && !out.trim_end().ends_with('!')
                    && !out.trim_end().ends_with('?')
                {
                    out.push('.');
                    out.push(' ');
                }
            }
        }
    }
    out
}

fn is_block_element(tag: &str) -> bool {
    matches!(
        tag,
        "address"
            | "article"
            | "aside"
            | "blockquote"
            | "dd"
            | "details"
            | "dialog"
            | "div"
            | "dl"
            | "dt"
            | "figcaption"
            | "figure"
            | "footer"
            | "form"
            | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "header"
            | "hr"
            | "li"
            | "main"
            | "nav"
            | "ol"
            | "p"
            | "pre"
            | "section"
            | "summary"
            | "table"
            | "tbody"
            | "td"
            | "tfoot"
            | "th"
            | "thead"
            | "tr"
            | "ul"
    )
}

fn extract_selector_text(doc: &Html, selector_str: &str) -> Option<String> {
    let sel = Selector::parse(selector_str).ok()?;
    let elements: Vec<_> = doc.select(&sel).collect();
    if elements.is_empty() {
        return None;
    }
    Some(
        elements
            .iter()
            .flat_map(|el| el.text())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

fn compute_content_hash(text: &str) -> String {
    let digest = md5::compute(text);
    format!("{digest:x}")
}

pub fn compute_simhash(text: &str) -> u64 {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 0;
    }
    if words.len() < 3 {
        let shingle = words.join(" ");
        return hash_string(&shingle);
    }

    let mut counts = [0i32; 64];
    for window in words.windows(3) {
        let shingle = window.join(" ");
        let hash = hash_string(&shingle);
        for (bit, count) in counts.iter_mut().enumerate() {
            if (hash >> bit) & 1 == 1 {
                *count += 1;
            } else {
                *count -= 1;
            }
        }
    }

    let mut result: u64 = 0;
    for (bit, count) in counts.iter().enumerate() {
        if *count > 0 {
            result |= 1 << bit;
        }
    }
    result
}

fn hash_string(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(html: &str) -> PageRecord {
        let mut record = PageRecord::default();
        analyze_html(&mut record, html, "");
        record
    }

    fn analyze_with_selector(html: &str, selector: &str) -> PageRecord {
        let mut record = PageRecord::default();
        analyze_html(&mut record, html, selector);
        record
    }

    fn analyze_at(url: &str, html: &str) -> PageRecord {
        let mut record = PageRecord {
            url: url.to_string(),
            ..Default::default()
        };
        analyze_html(&mut record, html, "");
        record
    }

    #[test]
    fn mixed_content_flagged_for_http_subresource_on_https_page() {
        let r = analyze_at(
            "https://example.test/page",
            r#"<html><head><script src="http://cdn.example.test/a.js"></script></head>
            <body><h1>H</h1></body></html>"#,
        );
        assert!(r.has_mixed_content);
    }

    #[test]
    fn mixed_content_ignores_https_and_protocol_relative_subresources() {
        let r = analyze_at(
            "https://example.test/page",
            r#"<html><head>
            <link rel="stylesheet" href="https://cdn.example.test/a.css">
            <script src="//cdn.example.test/a.js"></script>
            </head><body><img src="/local.png"><h1>H</h1></body></html>"#,
        );
        assert!(!r.has_mixed_content);
    }

    #[test]
    fn mixed_content_not_flagged_on_http_page() {
        // An HTTP page loading HTTP resources is not "mixed" content.
        let r = analyze_at(
            "http://example.test/page",
            r#"<html><body><img src="http://cdn.example.test/a.png"><h1>H</h1></body></html>"#,
        );
        assert!(!r.has_mixed_content);
    }

    #[test]
    fn extracts_basic_fields() {
        let r = analyze(
            r#"<html><head><title>My Page</title>
            <meta name="description" content="A test page">
            <link rel="canonical" href="https://example.com/">
            <meta name="robots" content="noindex, nofollow">
            </head><body>
            <h1>Main Heading</h1>
            <h2>Sub Heading</h2>
            <p>Hello world</p>
            </body></html>"#,
        );
        assert_eq!(r.title.as_deref(), Some("My Page"));
        assert_eq!(r.meta_description.as_deref(), Some("A test page"));
        assert_eq!(r.h1.as_deref(), Some("Main Heading"));
        assert_eq!(r.h2.as_deref(), Some("Sub Heading"));
        assert_eq!(r.canonical.as_deref(), Some("https://example.com/"));
        assert_eq!(r.robots.as_deref(), Some("noindex, nofollow"));
        assert_eq!(r.word_count, Some(6));
        assert_eq!(r.title_count, 1);
        assert_eq!(r.h1_count, 1);
        assert_eq!(r.h2_count, 1);
    }

    #[test]
    fn robots_meta_is_case_insensitive() {
        let mut r = analyze(
            r#"<html><head><title>T</title>
            <meta name="ROBOTS" content="NOINDEX">
            </head><body><h1>H</h1></body></html>"#,
        );
        assert_eq!(r.robots.as_deref(), Some("NOINDEX"));
        r.status = Some(200);
        r.compute_indexability();
        assert_eq!(r.indexability.as_deref(), Some("Non-Indexable"));
    }

    #[test]
    fn extracts_title_with_pipe_from_realistic_html() {
        let r = analyze(
            r#"<!doctype html><html lang="sv"><head>
            <meta charset="utf-8">
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <title>Diva Gummi Fasta Ringar 145mm | ByLynga</title>
            <link rel="preconnect" href="https://assets.example.com" crossorigin>
            <link rel="stylesheet" crossorigin href="/assets/index.css">
            <meta name="description" content="Diva Gummi Fasta Ringar 145mm från ByLynga. Alltid snabba leveranser!">
            <link rel="canonical" href="https://www.example.com/se/produkt/diva-145mm">
            </head><body>
            <div id="app"></div>
            <script type="module" src="/assets/index.js"></script>
            </body></html>"#,
        );
        assert_eq!(
            r.title.as_deref(),
            Some("Diva Gummi Fasta Ringar 145mm | ByLynga"),
            "title should contain the full text including the pipe character"
        );
        assert_eq!(
            r.meta_description.as_deref(),
            Some("Diva Gummi Fasta Ringar 145mm från ByLynga. Alltid snabba leveranser!")
        );
        assert_eq!(
            r.canonical.as_deref(),
            Some("https://www.example.com/se/produkt/diva-145mm")
        );
        assert_eq!(r.title_count, 1);
    }

    #[test]
    fn title_with_multiple_pipes_and_special_chars() {
        let r = analyze(
            r#"<html><head><title>Product: "Special" | Category | Site Name</title></head><body></body></html>"#,
        );
        assert_eq!(
            r.title.as_deref(),
            Some("Product: \"Special\" | Category | Site Name")
        );
    }

    #[test]
    fn title_from_html_with_svg_title_in_body() {
        let r = analyze(
            r#"<html><head><title>Page Title</title></head><body>
            <svg><title>SVG Icon Title</title><circle/></svg>
            </body></html>"#,
        );
        assert_eq!(r.title.as_deref(), Some("Page Title"));
        assert_eq!(r.title_count, 2);
    }

    #[test]
    fn counts_multiple_headings() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <h1>First</h1><h1>Second</h1>
            <h2>A</h2><h2>B</h2><h2>C</h2>
            </body></html>"#,
        );
        assert_eq!(r.h1_count, 2);
        assert_eq!(r.h2_count, 3);
        assert_eq!(r.title_count, 1);
    }

    #[test]
    fn extracts_hreflang_tags() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <link rel="alternate" hreflang="en" href="https://example.com/">
            <link rel="alternate" hreflang="de" href="https://example.com/de/">
            <link rel="alternate" hreflang="x-default" href="https://example.com/">
            </head><body></body></html>"#,
        );
        assert_eq!(r.hreflang_tags.len(), 3);
        assert_eq!(
            r.hreflang_tags[0],
            ("en".into(), "https://example.com/".into())
        );
        assert_eq!(
            r.hreflang_tags[1],
            ("de".into(), "https://example.com/de/".into())
        );
        assert_eq!(
            r.hreflang_tags[2],
            ("x-default".into(), "https://example.com/".into())
        );
    }

    #[test]
    fn hreflang_ignores_empty_values() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <link rel="alternate" hreflang="" href="https://example.com/">
            <link rel="alternate" hreflang="en" href="">
            </head><body></body></html>"#,
        );
        assert!(r.hreflang_tags.is_empty());
    }

    #[test]
    fn no_hreflang_when_absent() {
        let r = analyze(r#"<html><head><title>T</title></head><body></body></html>"#);
        assert!(r.hreflang_tags.is_empty());
    }

    #[test]
    fn extracts_single_json_ld_type() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"Widget","image":"x","description":"d","brand":"b","review":"r","aggregateRating":"a","offers":"o"}</script>
            </head><body></body></html>"#,
        );
        assert_eq!(r.sd_types, vec!["Product"]);
        assert_eq!(r.sd_errors, 0);
    }

    #[test]
    fn extracts_multiple_types_from_graph() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{
                "@context": "https://schema.org",
                "@graph": [
                    {"@type": "Product", "name": "Widget"},
                    {"@type": "BreadcrumbList", "itemListElement": []}
                ]
            }</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
        assert!(r.sd_types.contains(&"BreadcrumbList".to_string()));
        assert_eq!(r.sd_types.len(), 2);
    }

    #[test]
    fn extracts_types_from_json_ld_array() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">[
                {"@type": "Article", "headline": "News"},
                {"@type": "Organization", "name": "Corp"}
            ]</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.contains(&"Article".to_string()));
        assert!(r.sd_types.contains(&"Organization".to_string()));
    }

    #[test]
    fn deduplicates_schema_types() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"A"}</script>
            <script type="application/ld+json">{"@type":"Product","name":"B"}</script>
            </head><body></body></html>"#,
        );
        assert_eq!(r.sd_types.len(), 1);
        assert_eq!(r.sd_types[0], "Product");
    }

    #[test]
    fn tracks_parse_errors() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{invalid json}</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.is_empty());
        assert_eq!(r.sd_errors, 1);
    }

    #[test]
    fn empty_script_ignored() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json"></script>
            <script type="application/ld+json">   </script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.is_empty());
        assert_eq!(r.sd_errors, 0);
    }

    #[test]
    fn missing_fields_are_none() {
        let r = analyze(r#"<html><head></head><body></body></html>"#);
        assert!(r.title.is_none());
        assert!(r.meta_description.is_none());
        assert!(r.h1.is_none());
        assert!(r.h2.is_none());
        assert!(r.canonical.is_none());
        assert!(r.robots.is_none());
    }

    #[test]
    fn extracts_microdata_itemtype() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">Widget</span>
                <span itemprop="price">$9.99</span>
            </div>
            </body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
    }

    #[test]
    fn extracts_multiple_microdata_types() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">Widget</span>
            </div>
            <div itemscope itemtype="https://schema.org/Organization">
                <span itemprop="name">Corp</span>
            </div>
            </body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
        assert!(r.sd_types.contains(&"Organization".to_string()));
        assert_eq!(r.sd_types.len(), 2);
    }

    #[test]
    fn microdata_deduplicates_json_ld_types() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"Widget"}</script>
            </head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">Widget</span>
            </div>
            </body></html>"#,
        );
        assert_eq!(r.sd_types.iter().filter(|t| *t == "Product").count(), 1);
    }

    #[test]
    fn microdata_with_nested_scopes() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">Widget</span>
                <div itemprop="brand" itemscope itemtype="https://schema.org/Brand">
                    <span itemprop="name">Acme</span>
                </div>
            </div>
            </body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
        assert!(r.sd_types.contains(&"Brand".to_string()));
    }

    #[test]
    fn itemscope_without_itemtype_ignored() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <div itemscope>
                <span itemprop="name">Widget</span>
            </div>
            </body></html>"#,
        );
        assert!(r.sd_types.is_empty());
    }

    #[test]
    fn product_missing_required_name_is_error() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product"}</script>
            </head><body></body></html>"#,
        );
        assert!(
            r.sd_issues
                .iter()
                .any(|i| i.code == "missing-required:name" && i.severity == SdSeverity::Error)
        );
    }

    #[test]
    fn product_missing_recommended_image_is_warning() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"Widget"}</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_issues.iter().any(|i|
            i.code == "missing-recommended:image" && i.severity == SdSeverity::Warning
        ));
        assert!(r.sd_warnings > 0);
    }

    #[test]
    fn unknown_type_generates_error() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"FakeType123","name":"x"}</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_issues.iter().any(|i| i.code == "unknown-type"));
        assert!(r.sd_errors > 0);
    }

    #[test]
    fn tracks_jsonld_and_microdata_format_counts() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"A"}</script>
            </head><body>
            <div itemscope itemtype="https://schema.org/Organization">
                <span itemprop="name">Corp</span>
            </div>
            </body></html>"#,
        );
        assert_eq!(r.sd_jsonld_count, 1);
        assert_eq!(r.sd_microdata_count, 1);
    }

    #[test]
    fn sd_items_populated_with_format() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"A"}</script>
            </head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <span itemprop="name">B</span>
            </div>
            </body></html>"#,
        );
        assert_eq!(r.sd_items.len(), 2);
        assert_eq!(r.sd_items[0].format, SdFormat::JsonLd);
        assert_eq!(r.sd_items[1].format, SdFormat::Microdata);
    }

    #[test]
    fn extracts_images_with_alt() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <img src="/logo.png" alt="Logo" width="200" height="50">
            </body></html>"#,
        );
        assert_eq!(r.images.len(), 1);
        assert_eq!(r.images[0].src, "/logo.png");
        assert_eq!(r.images[0].alt, Some("Logo".into()));
        assert_eq!(r.images[0].width, Some(200));
        assert_eq!(r.images[0].height, Some(50));
        assert!(r.images[0].has_alt_attr);
    }

    #[test]
    fn image_missing_alt_attr() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <img src="/photo.jpg">
            </body></html>"#,
        );
        assert_eq!(r.images.len(), 1);
        assert!(!r.images[0].has_alt_attr);
        assert_eq!(r.images[0].alt, None);
    }

    #[test]
    fn image_empty_alt_has_attr() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <img src="/spacer.gif" alt="">
            </body></html>"#,
        );
        assert_eq!(r.images.len(), 1);
        assert!(r.images[0].has_alt_attr);
        assert_eq!(r.images[0].alt, Some("".into()));
    }

    #[test]
    fn skips_img_without_src() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <img alt="no src">
            </body></html>"#,
        );
        assert!(r.images.is_empty());
    }

    #[test]
    fn extracts_multiple_images() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <img src="/a.jpg" alt="A">
            <img src="/b.png" width="100">
            <img src="/c.webp" alt="C" width="300" height="200">
            </body></html>"#,
        );
        assert_eq!(r.images.len(), 3);
        assert_eq!(r.images[0].src, "/a.jpg");
        assert!(!r.images[1].has_alt_attr);
        assert_eq!(r.images[2].width, Some(300));
        assert_eq!(r.images[2].height, Some(200));
    }

    #[test]
    fn content_hash_deterministic() {
        let r1 = analyze(r#"<html><head><title>T</title></head><body>Hello world</body></html>"#);
        let r2 = analyze(r#"<html><head><title>T</title></head><body>Hello world</body></html>"#);
        assert_eq!(r1.content_hash, r2.content_hash);
        assert!(r1.content_hash.is_some());
    }

    #[test]
    fn content_hash_differs_for_different_content() {
        let r1 = analyze(r#"<html><head><title>T</title></head><body>Alpha content</body></html>"#);
        let r2 = analyze(r#"<html><head><title>T</title></head><body>Beta content</body></html>"#);
        assert_ne!(r1.content_hash, r2.content_hash);
    }

    #[test]
    fn simhash_populated_for_body_content() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>Some page content here</body></html>"#,
        );
        assert!(r.simhash.is_some());
    }

    #[test]
    fn simhash_similar_for_near_duplicate_text() {
        let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
        let modified = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi OTHER";
        let r1 = analyze(&format!(
            r#"<html><head><title>T</title></head><body>{base}</body></html>"#
        ));
        let r2 = analyze(&format!(
            r#"<html><head><title>T</title></head><body>{modified}</body></html>"#
        ));
        let distance = (r1.simhash.unwrap() ^ r2.simhash.unwrap()).count_ones();
        assert!(
            distance <= 15,
            "hamming distance {distance} should be small for near-duplicates"
        );
    }

    #[test]
    fn simhash_distant_for_unrelated_text() {
        let r1 = analyze(
            r#"<html><head><title>T</title></head><body>Alpha beta gamma delta epsilon zeta eta theta iota kappa lambda</body></html>"#,
        );
        let r2 = analyze(
            r#"<html><head><title>T</title></head><body>Red green blue yellow orange purple pink brown black white gray</body></html>"#,
        );
        let distance = (r1.simhash.unwrap() ^ r2.simhash.unwrap()).count_ones();
        assert!(
            distance > 10,
            "hamming distance {distance} should be large for unrelated content"
        );
    }

    #[test]
    fn og_type_extracted() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <meta property="og:type" content="product">
            </head><body></body></html>"#,
        );
        assert_eq!(r.og_type.as_deref(), Some("product"));
    }

    #[test]
    fn og_type_none_when_absent() {
        let r = analyze(r#"<html><head><title>T</title></head><body></body></html>"#);
        assert!(r.og_type.is_none());
    }

    #[test]
    fn ecommerce_audit_from_product_jsonld() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{
                "@type": "Product",
                "name": "Widget",
                "image": "https://example.com/widget.jpg",
                "description": "A fine widget",
                "sku": "WDG-001",
                "gtin13": "1234567890123",
                "brand": {"@type": "Brand", "name": "Acme"},
                "review": {"@type": "Review", "reviewBody": "Great"},
                "offers": {
                    "@type": "Offer",
                    "price": "9.99",
                    "priceCurrency": "USD",
                    "availability": "https://schema.org/InStock"
                }
            }</script>
            </head><body></body></html>"#,
        );
        let audit = r.ecommerce.as_ref().unwrap();
        assert_eq!(audit.price.as_deref(), Some("9.99"));
        assert_eq!(audit.currency.as_deref(), Some("USD"));
        assert_eq!(audit.sku.as_deref(), Some("WDG-001"));
        assert_eq!(audit.gtin.as_deref(), Some("1234567890123"));
        assert_eq!(audit.brand.as_deref(), Some("Acme"));
        assert!(audit.has_image);
        assert!(audit.has_description);
        assert!(audit.has_review_or_rating);
        assert_eq!(audit.availability.as_deref(), Some("instock"));
    }

    #[test]
    fn ecommerce_audit_from_og_type() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <meta property="og:type" content="product">
            <script type="application/ld+json">{
                "@type": "Product",
                "name": "OG Product",
                "offers": {"@type": "Offer", "price": 19.99, "priceCurrency": "EUR"}
            }</script>
            </head><body></body></html>"#,
        );
        assert!(r.ecommerce.is_some());
        let audit = r.ecommerce.as_ref().unwrap();
        assert_eq!(audit.price.as_deref(), Some("19.99"));
        assert_eq!(audit.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn ecommerce_audit_none_for_non_product() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Article","headline":"News"}</script>
            </head><body></body></html>"#,
        );
        assert!(r.ecommerce.is_none());
    }

    #[test]
    fn ecommerce_audit_missing_fields() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{"@type":"Product","name":"Bare"}</script>
            </head><body></body></html>"#,
        );
        let audit = r.ecommerce.as_ref().unwrap();
        assert!(audit.price.is_none());
        assert!(audit.currency.is_none());
        assert!(audit.availability.is_none());
        assert!(audit.sku.is_none());
        assert!(audit.gtin.is_none());
        assert!(!audit.has_image);
        assert!(!audit.has_description);
        assert!(!audit.has_review_or_rating);
    }

    #[test]
    fn ecommerce_brand_as_string() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{
                "@type": "Product",
                "name": "X",
                "brand": "Acme Corp"
            }</script>
            </head><body></body></html>"#,
        );
        let audit = r.ecommerce.as_ref().unwrap();
        assert_eq!(audit.brand.as_deref(), Some("Acme Corp"));
    }

    #[test]
    fn content_selector_scopes_word_count() {
        let r = analyze_with_selector(
            r#"<html><head><title>T</title></head><body>
            <nav>Navigation text here</nav>
            <main><p>Main content only</p></main>
            </body></html>"#,
            "main",
        );
        assert_eq!(r.word_count, Some(3));
    }

    #[test]
    fn content_selector_falls_back_to_body_when_no_match() {
        let r = analyze_with_selector(
            r#"<html><head><title>T</title></head><body>
            <p>Hello world</p>
            </body></html>"#,
            "article",
        );
        assert_eq!(r.word_count, Some(2));
    }

    #[test]
    fn empty_selector_uses_full_body() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body><nav>Nav</nav><main><p>Content</p></main></body></html>"#,
        );
        assert_eq!(r.word_count, Some(2));
    }

    #[test]
    fn ssr_flags_content_missing_when_server_html_near_empty() {
        let mut r = PageRecord {
            word_count: Some(200),
            h1: Some("Rendered Heading".into()),
            ..Default::default()
        };
        let raw = r#"<html><head><title>T</title></head><body><div id="app"></div></body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert_eq!(r.ssr_word_count, Some(0));
        assert_eq!(r.ssr_h1.as_deref().unwrap_or(""), "");
        assert_eq!(r.ssr_content_missing, Some(true));
    }

    #[test]
    fn ssr_not_flagged_when_server_html_matches_render() {
        let mut r = PageRecord {
            word_count: Some(6),
            h1: Some("Main Heading".into()),
            ..Default::default()
        };
        let raw = r#"<html><head><title>T</title></head><body>
            <h1>Main Heading</h1><p>one two three four</p></body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert_eq!(r.ssr_h1.as_deref(), Some("Main Heading"));
        assert_eq!(r.ssr_content_missing, Some(false));
    }

    #[test]
    fn ssr_flags_when_h1_only_after_render() {
        let mut r = PageRecord {
            word_count: Some(10),
            h1: Some("Hydrated Heading".into()),
            ..Default::default()
        };
        // SSR has plenty of words but no h1 at all.
        let raw = r#"<html><head><title>T</title></head><body>
            <p>one two three four five six seven eight nine ten</p></body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert_eq!(r.ssr_content_missing, Some(true));
    }

    #[test]
    fn ssr_skips_meta_refresh_redirect_pages() {
        // Chrome follows the meta-refresh to a content-rich target; the raw stub
        // does not. The diff must be skipped, not flagged as content-missing.
        let mut r = PageRecord {
            word_count: Some(200),
            h1: Some("Target Page Heading".into()),
            ..Default::default()
        };
        let raw = r#"<html><head><meta http-equiv="refresh" content="0;url=/home">
            <title>Redirect</title></head><body><p>Redirecting...</p></body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert_eq!(r.ssr_content_missing, None);
        assert_eq!(r.ssr_word_count, None);
    }

    #[test]
    fn csr_only_set_for_links_absent_from_raw_html() {
        let mut r = PageRecord {
            url: "https://example.com/page".into(),
            word_count: Some(200),
            h1: Some("Heading".into()),
            outlinks: vec![
                Outlink {
                    dst_url: "https://example.com/".into(),
                    anchor: Some("Home".into()),
                    rel: None,
                    csr_only: false,
                },
                Outlink {
                    dst_url: "https://example.com/js-link".into(),
                    anchor: Some("JS Link".into()),
                    rel: None,
                    csr_only: false,
                },
            ],
            ..Default::default()
        };
        let raw = r#"<html><head><title>T</title></head><body>
            <h1>Heading</h1>
            <a href="https://example.com/">Home</a>
            <p>enough words to pass the threshold one two three four five six seven eight nine ten</p>
            </body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert!(
            !r.outlinks[0].csr_only,
            "link in raw HTML should not be csr_only"
        );
        assert!(
            r.outlinks[1].csr_only,
            "link absent from raw HTML should be csr_only"
        );
    }

    #[test]
    fn csr_only_not_set_when_all_links_present_in_raw() {
        let mut r = PageRecord {
            url: "https://example.com/page".into(),
            word_count: Some(50),
            h1: Some("Heading".into()),
            outlinks: vec![Outlink {
                dst_url: "https://example.com/about".into(),
                anchor: Some("About".into()),
                rel: None,
                csr_only: false,
            }],
            ..Default::default()
        };
        let raw = r#"<html><head><title>T</title></head><body>
            <h1>Heading</h1>
            <a href="https://example.com/about">About</a>
            <p>one two three four five six seven eight nine ten</p>
            </body></html>"#;
        analyze_ssr(&mut r, raw, "");
        assert!(!r.outlinks[0].csr_only);
    }

    #[test]
    fn csr_only_not_touched_in_http_mode() {
        let mut r = PageRecord {
            url: "https://example.com/page".into(),
            ..Default::default()
        };
        analyze_html(
            &mut r,
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/a">A</a>
            <a href="https://example.com/b">B</a>
            </body></html>"#,
            "",
        );
        assert_eq!(r.outlinks.len(), 2);
        assert!(!r.outlinks[0].csr_only);
        assert!(!r.outlinks[1].csr_only);
    }
}
