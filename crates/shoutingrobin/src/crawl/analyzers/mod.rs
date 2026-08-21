mod rich_results;
mod schema_org;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use scraper::{Html, Selector};

use crate::crawl::url_norm::decode_entities;

use crate::crawl::event::{
    EcommerceAudit, HreflangSource, ImageRef, Outlink, PageRecord, SdFormat, SdIssue, SdItem,
    SdSeverity, Subresource, SubresourceKind,
};
use crate::crawl::font_metrics::{meta_description_pixel_width, title_pixel_width};

pub fn analyze_html(record: &mut PageRecord, html: &str, content_selector: &str) {
    let doc = Html::parse_document(html);

    record.has_body_tag = Some(has_element(html, "body"));

    let titles = document_titles(&doc);
    record.title = titles.first().cloned();
    record.meta_description = select_attr(&doc, r#"meta[name="description"]"#, "content");
    record.h1 = select_text(&doc, "h1");
    record.h2 = select_text(&doc, "h2");
    record.title_2 = titles.get(1).cloned();
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
    record.title_count = titles.len() as u32;
    record.h1_count = count_elements(&doc, "h1");
    record.h2_count = count_elements(&doc, "h2");
    record.h2_non_sequential = Some(h2_precedes_h1(&doc));

    record.title_pixel_width = record.title.as_ref().map(|t| title_pixel_width(t));
    record.meta_description_pixel_width = record
        .meta_description
        .as_ref()
        .map(|d| meta_description_pixel_width(d));

    extract_perf_metrics(&doc, record);
    extract_hreflang(&doc, record);
    extract_structured_data(&doc, record);
    extract_microdata(&doc, record);
    extract_images(&doc, record);
    extract_subresources(&doc, record);
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

/// True when the page opens a section with an H2 before it has said what the
/// page is about with its H1. Headings are a document outline, and one that
/// starts at the second level asks every reader that walks it — search engine,
/// screen reader, summariser — to guess where the top level went.
///
/// A page with no H1 at all is not counted here: nothing is out of order when
/// there is no order, and `Missing H1` is the finding it has.
fn h2_precedes_h1(doc: &Html) -> bool {
    let Ok(selector) = Selector::parse("h1, h2") else {
        return false;
    };
    // `select` walks the tree in document order, so the first match is the
    // heading the reader meets first.
    doc.select(&selector)
        .next()
        .is_some_and(|first| first.value().name() == "h2")
        && count_elements(doc, "h1") > 0
}

/// True when the markup as served contains the named element's start tag.
///
/// Read off the source rather than the parsed tree on purpose: an HTML parser
/// is required to invent `<html>`, `<head>` and `<body>` when they are absent,
/// so every parsed document has a body whether or not the server sent one.
/// Only the source can answer whether the author wrote it.
///
/// Browsers recover, which is why a page like this looks fine and still counts:
/// what the parser had to guess is not what the site said.
fn has_element(html: &str, name: &str) -> bool {
    let lowered = html.to_ascii_lowercase();
    let needle = format!("<{name}");
    lowered.match_indices(&needle).any(|(index, _)| {
        // `<body` must be the whole tag name, so what follows it is whitespace,
        // the end of the tag, or a self-closing slash. Otherwise `<bodybuilder>`
        // would answer for `<body>`.
        lowered[index + needle.len()..]
            .chars()
            .next()
            .is_none_or(|next| next.is_whitespace() || next == '>' || next == '/')
    })
}

/// Compares the raw server-rendered HTML against the already-analyzed rendered
/// DOM (`record`) to detect pages whose content only appears after client-side
/// JavaScript. Call after `analyze_html` so the rendered fields are populated.
pub fn analyze_ssr(record: &mut PageRecord, raw_html: &str, content_selector: &str) {
    // Read off the server's markup, replacing whatever the rendered DOM said.
    // Chrome hands back a serialised document, and a serialised document always
    // has a body because the parser put one there — in Chrome mode the answer
    // is only available here, from the bytes the server actually sent.
    record.has_body_tag = Some(has_element(raw_html, "body"));

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

/// The hreflang alternates a `Link:` response header advertises.
///
/// A header carries comma-separated links, each a `<url>` followed by
/// semicolon-separated parameters:
/// `<https://a.test/de>; rel="alternate"; hreflang="de", <...>; ...`.
/// Only `rel=alternate` entries carrying an `hreflang` are hreflang tags; the
/// same header is also used for canonicals, preloads and pagination.
pub fn parse_link_header_hreflang(header: &str, base: &url::Url) -> Vec<(String, String)> {
    let mut tags = Vec::new();
    // Splitting on commas is safe here because a URL is bracketed and a
    // parameter value carrying a comma would have to be quoted; both are
    // handled by only accepting a well-formed `<...>` at the start.
    for entry in header.split(',') {
        let entry = entry.trim();
        let Some(rest) = entry.strip_prefix('<') else {
            continue;
        };
        let Some((raw_url, params)) = rest.split_once('>') else {
            continue;
        };
        let mut rel = None;
        let mut lang = None;
        for param in params.split(';') {
            let Some((name, value)) = param.split_once('=') else {
                continue;
            };
            let value = value.trim().trim_matches('"').to_string();
            match name.trim().to_ascii_lowercase().as_str() {
                "rel" => rel = Some(value),
                "hreflang" => lang = Some(value),
                _ => {}
            }
        }
        let (Some(rel), Some(lang)) = (rel, lang) else {
            continue;
        };
        if !rel
            .split_whitespace()
            .any(|value| value.eq_ignore_ascii_case("alternate"))
        {
            continue;
        }
        let Ok(url) = base.join(raw_url.trim()) else {
            continue;
        };
        tags.push((lang, url.to_string()));
    }
    tags
}

/// Merges hreflang tags from another source into the page's set, recording the
/// source when it contributed anything new. A tag already known from the HTML
/// is not a second tag, so duplicates are dropped rather than stacked.
pub fn merge_hreflang_tags(
    record: &mut PageRecord,
    tags: Vec<(String, String)>,
    source: HreflangSource,
) {
    let mut added = false;
    for (lang, url) in tags {
        if record
            .hreflang_tags
            .iter()
            .any(|(known_lang, known_url)| known_lang == &lang && known_url == &url)
        {
            continue;
        }
        record.hreflang_tags.push((lang, url));
        added = true;
    }
    if added && !record.hreflang_sources.contains(&source) {
        record.hreflang_sources.push(source);
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
            // Relative hrefs are legal here and must be resolved like
            // canonicals, otherwise the return-tag and self-reference checks
            // never find the target among the crawled pages.
            let resolved = crate::crawl::url_norm::resolve_url(&record.url, href)
                .unwrap_or_else(|| href.to_string());
            record
                .hreflang_tags
                .push((lang.trim().to_string(), resolved));
        }
    }
    if !record.hreflang_tags.is_empty() {
        record.hreflang_sources.push(HreflangSource::Html);
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
                extract_schema_objects(&value, SdFormat::JsonLd, record);
            }
            Err(_) => {
                record.sd_errors = record.sd_errors.saturating_add(1);
            }
        }
    }
}

/// Properties that *contain* another entity rather than describe this one.
/// Recursing through these finds a `Product` under `WebPage.mainEntity` or the
/// products of a category `ItemList`, without turning every `brand`,
/// `publisher` or `review` attribute into a separately validated item.
const CONTAINER_PROPERTIES: &[&str] =
    &["@graph", "mainEntity", "hasPart", "itemListElement", "item"];

fn schema_type_names(value: &serde_json::Value) -> Vec<&str> {
    match value {
        serde_json::Value::String(t) => vec![t.as_str()],
        serde_json::Value::Array(arr) => arr.iter().filter_map(|v| v.as_str()).collect(),
        _ => Vec::new(),
    }
}

fn extract_schema_objects(value: &serde_json::Value, format: SdFormat, record: &mut PageRecord) {
    match value {
        serde_json::Value::Array(arr) => {
            for item in arr {
                extract_schema_objects(item, format, record);
            }
        }
        serde_json::Value::Object(map) => {
            let type_names = map.get("@type").map(schema_type_names).unwrap_or_default();
            if !type_names.is_empty() {
                // Serialising the object itself, not the enclosing script, is
                // what lets the ecommerce audit and the details panel see a
                // Product that lives inside `@graph` or an array.
                let raw_json = serde_json::to_string_pretty(map).unwrap_or_default();
                for t in type_names {
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
                        raw_json: raw_json.clone(),
                    });
                }
            }
            for property in CONTAINER_PROPERTIES {
                if let Some(nested) = map.get(*property) {
                    extract_schema_objects(nested, format, record);
                }
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

        let properties = microdata_properties(el);
        let issues = rich_results::validate_type(&type_name, &properties);
        for issue in &issues {
            if issue.severity == SdSeverity::Error {
                record.sd_errors = record.sd_errors.saturating_add(1);
            } else {
                record.sd_warnings = record.sd_warnings.saturating_add(1);
            }
        }
        record.sd_issues.extend(issues);

        let mut object = properties;
        object.insert("@type".into(), serde_json::Value::String(type_name.clone()));
        record.sd_items.push(SdItem {
            format: SdFormat::Microdata,
            type_name,
            raw_json: serde_json::to_string_pretty(&object).unwrap_or_default(),
        });
    }
}

/// The `itemprop`s of one `itemscope`, as the JSON-LD-shaped object the
/// validators and the ecommerce audit already understand. Nested scopes become
/// nested objects; a property given more than once becomes an array.
fn microdata_properties(
    scope: scraper::ElementRef<'_>,
) -> serde_json::Map<String, serde_json::Value> {
    let mut properties = serde_json::Map::new();
    let mut stack: Vec<ego_tree::NodeRef<'_, scraper::Node>> = scope.children().collect();
    while let Some(node) = stack.pop() {
        let Some(el) = scraper::ElementRef::wrap(node) else {
            continue;
        };
        let value = el.value();
        let is_scope = value.attr("itemscope").is_some();
        if let Some(names) = value.attr("itemprop") {
            let prop_value = if is_scope {
                let mut nested = microdata_properties(el);
                if let Some(type_url) = value.attr("itemtype") {
                    let type_name = type_url.trim().rsplit('/').next().unwrap_or(type_url);
                    nested.insert(
                        "@type".into(),
                        serde_json::Value::String(type_name.to_string()),
                    );
                }
                serde_json::Value::Object(nested)
            } else {
                serde_json::Value::String(microdata_text_value(el))
            };
            for name in names.split_whitespace() {
                match properties.get_mut(name) {
                    Some(serde_json::Value::Array(existing)) => existing.push(prop_value.clone()),
                    Some(existing) => {
                        let first = existing.take();
                        *existing = serde_json::Value::Array(vec![first, prop_value.clone()]);
                    }
                    None => {
                        properties.insert(name.to_string(), prop_value.clone());
                    }
                }
            }
        }
        if !is_scope {
            stack.extend(el.children());
        }
    }
    properties
}

fn microdata_text_value(el: scraper::ElementRef<'_>) -> String {
    let value = el.value();
    let attr = match value.name() {
        "meta" => Some("content"),
        "img" | "audio" | "video" | "embed" | "iframe" | "source" | "track" => Some("src"),
        "a" | "link" | "area" => Some("href"),
        "object" => Some("data"),
        "data" | "meter" => Some("value"),
        "time" => Some("datetime"),
        _ => None,
    };
    if let Some(text) = attr.and_then(|a| value.attr(a)) {
        return text.trim().to_string();
    }
    if let Some(content) = value.attr("content") {
        return content.trim().to_string();
    }
    el.text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Replaces an inline `data:` image's payload with its media type, a hash of
/// the payload and its decoded size: `data:image/png;base64,#a1b2c3d4 (2048 B)`.
///
/// The payload itself is of no use to any report and is unbounded: these were
/// inline SVG flag icons on the site this was found on, but a base64 PNG would
/// put megabytes of string in SQLite and in the details panel for every page
/// referencing it. The hash keeps two different inline images apart and the
/// same one together, and the size is what an "image over 100 kB" rule needs.
fn summarize_inline_image_src(src: &str) -> String {
    let Some((prefix, payload)) = src.split_once(',') else {
        return src.to_string();
    };
    let mut hasher = DefaultHasher::new();
    payload.hash(&mut hasher);
    let digest = hasher.finish();

    // base64 carries 6 bits per character, less the padding.
    let bytes = if prefix.to_ascii_lowercase().contains("base64") {
        let padding = payload.bytes().rev().take_while(|&b| b == b'=').count();
        (payload.len() / 4 * 3).saturating_sub(padding)
    } else {
        payload.len()
    };

    format!("{prefix},#{digest:016x} ({bytes} B)")
}

/// Attributes lazy-loading libraries park the real image in while `src`
/// holds a placeholder.
const LAZY_SRC_ATTRIBUTES: &[&str] = &["data-src", "data-lazy-src", "data-original", "data-lazy"];
const LAZY_SRCSET_ATTRIBUTES: &[&str] = &["data-srcset", "data-lazy-srcset"];

/// The URLs in a `srcset` value, without their width/density descriptors.
fn srcset_candidates(srcset: &str) -> Vec<&str> {
    srcset
        .split(',')
        .filter_map(|candidate| candidate.split_whitespace().next())
        .filter(|url| !url.is_empty())
        .collect()
}

fn extract_images(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse("img") else {
        return;
    };
    let base = url::Url::parse(&record.url).ok();
    let resolve = |src: &str| -> String {
        if src.to_ascii_lowercase().starts_with("data:") {
            summarize_inline_image_src(src)
        } else {
            // Resolved against the page, so an image is identified by where it
            // actually lives: a bare "/logo.png" cannot be requested, cannot be
            // opened from the details panel, and would merge two different
            // origins' images into one row on a subdomain crawl.
            match base.as_ref().and_then(|base| base.join(src).ok()) {
                Some(resolved) => resolved.to_string(),
                None => src.to_string(),
            }
        }
    };
    for el in doc.select(&sel) {
        let attr = |name: &str| {
            el.value()
                .attr(name)
                .map(str::trim)
                .filter(|v| !v.is_empty())
        };
        // A lazy-loaded image's `src` is a placeholder (a data: pixel or a
        // spacer) and the picture a shopper sees is in a data-* attribute;
        // reporting the placeholder would hide every product image on a
        // lazy-loading theme from the alt and size rules.
        let src = attr("src");
        let lazy_src = LAZY_SRC_ATTRIBUTES.iter().find_map(|name| attr(name));
        let effective_src = match (src, lazy_src) {
            (Some(src), Some(lazy)) if src.to_ascii_lowercase().starts_with("data:") => Some(lazy),
            (Some(src), _) => Some(src),
            (None, lazy) => lazy,
        };

        let mut sources: Vec<String> = Vec::new();
        if let Some(src) = effective_src {
            sources.push(resolve(src));
        }
        // Responsive candidates, on the image itself and on the `<source>`
        // elements of an enclosing `<picture>`.
        let srcsets = ["srcset"]
            .iter()
            .chain(LAZY_SRCSET_ATTRIBUTES)
            .filter_map(|name| attr(name));
        for srcset in srcsets {
            sources.extend(srcset_candidates(srcset).into_iter().map(resolve));
        }
        if let Some(picture) = el
            .parent()
            .and_then(scraper::ElementRef::wrap)
            .filter(|parent| parent.value().name().eq_ignore_ascii_case("picture"))
        {
            for child in picture.children().filter_map(scraper::ElementRef::wrap) {
                if child.value().name().eq_ignore_ascii_case("source")
                    && let Some(srcset) = child.value().attr("srcset")
                {
                    sources.extend(srcset_candidates(srcset).into_iter().map(resolve));
                }
            }
        }
        // One row per distinct URL for this element; the same URL used by
        // another <img> on the page is another reference and stays.
        let mut seen = std::collections::HashSet::new();
        sources.retain(|src| seen.insert(src.clone()));

        let has_alt_attr = el.value().attr("alt").is_some();
        let alt = el.value().attr("alt").map(|a| a.to_string());
        let width = el.value().attr("width").and_then(|w| w.parse().ok());
        let height = el.value().attr("height").and_then(|h| h.parse().ok());

        for src in sources {
            record.images.push(ImageRef {
                src,
                alt: alt.clone(),
                width,
                height,
                has_alt_attr,
            });
        }
    }
}

/// Records the stylesheets and scripts the page pulls in, so the post-crawl
/// resource pass can status-check them. Images come from `extract_images` and
/// links from `extract_anchors`; these two have no other home.
fn extract_subresources(doc: &Html, record: &mut PageRecord) {
    let Some(base) = url::Url::parse(&record.url).ok() else {
        return;
    };
    const SOURCES: [(&str, &str, SubresourceKind); 2] = [
        (
            r#"link[rel="stylesheet" i][href]"#,
            "href",
            SubresourceKind::Stylesheet,
        ),
        ("script[src]", "src", SubresourceKind::Script),
    ];
    for (selector, attribute, kind) in SOURCES {
        let Ok(sel) = Selector::parse(selector) else {
            continue;
        };
        for el in doc.select(&sel) {
            let Some(value) = el.value().attr(attribute) else {
                continue;
            };
            let value = decode_entities(value.trim());
            if value.is_empty() {
                continue;
            }
            let Some(url) = base.join(&value).ok().map(|url| url.to_string()) else {
                continue;
            };
            let subresource = Subresource { url, kind };
            if !record.subresources.contains(&subresource) {
                record.subresources.push(subresource);
            }
        }
    }
}

/// The absolute destination of a link, or `None` for links that are not
/// navigations at all. The scheme and fragment checks run on the raw `href`:
/// once joined, `#main` has become the page's own URL and `javascript:` has
/// been rejected by the parser anyway. The fragment is dropped from the result
/// because `/a#reviews` and `/a` are the same document to a crawler.
fn resolve_href(base_url: &url::Url, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('#') {
        return None;
    }
    let lowered = href.to_ascii_lowercase();
    if ["javascript:", "mailto:", "tel:", "sms:", "data:"]
        .iter()
        .any(|scheme| lowered.starts_with(scheme))
    {
        return None;
    }
    let href = decode_entities(href);
    let mut resolved = base_url.join(&href).ok()?;
    if !matches!(resolved.scheme(), "http" | "https") {
        return None;
    }
    resolved.set_fragment(None);
    Some(resolved.to_string())
}

/// Collapses runs of whitespace, returning `None` when nothing is left.
fn collapse_whitespace(text: &str) -> Option<String> {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        None
    } else {
        Some(collapsed)
    }
}

/// The anchor text of a link: its accessible name, in the order a browser
/// computes one.
///
/// `aria-label` first, because an author who writes one is overriding whatever
/// is inside the element and both screen readers and search engines honour
/// that. Then the element's text content, which already includes the text of an
/// inline `<svg><title>`, since that title is a text node inside the anchor.
/// Only when all of that collapses to nothing does the `alt` of a wrapped image
/// stand in: search engines read an image link's `alt` as its anchor text, so a
/// header logo linking home with `alt="ByLynga"` is not an anchorless link, and
/// without the fallback one templated image link is reported on every page of a
/// site.
///
/// `alt` written on the `<a>` itself is deliberately not read. HTML allows the
/// attribute on `<img>`, `<area>` and `<input type="image">` only, so browsers
/// ignore it, screen readers announce the link unlabelled and a search engine
/// reads nothing. A link carrying only that is genuinely anchorless.
fn anchor_text(el: &scraper::ElementRef, image_alt: &Selector) -> Option<String> {
    if let Some(label) = el.value().attr("aria-label").and_then(collapse_whitespace) {
        return Some(label);
    }
    let text = el.text().collect::<Vec<_>>().join(" ");
    if let Some(anchor) = collapse_whitespace(&text) {
        return Some(anchor);
    }
    let alt = el
        .select(image_alt)
        .filter_map(|img| img.value().attr("alt"))
        .collect::<Vec<_>>()
        .join(" ");
    collapse_whitespace(&alt)
}

fn extract_anchors(doc: &Html, record: &mut PageRecord) {
    let Ok(sel) = Selector::parse("a[href]") else {
        return;
    };
    let Ok(image_alt) = Selector::parse("img[alt]") else {
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
        let anchor = anchor_text(&el, &image_alt);
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
        if let Some(brand) = map.get("brand").or_else(|| map.get("manufacturer")) {
            audit.brand = extract_brand_name(brand);
        }
        if let Some(sku) = map.get("sku").and_then(scalar_to_string) {
            audit.sku = Some(sku);
        }
        audit.gtin = ["gtin", "gtin13", "gtin12", "gtin8", "gtin14", "isbn"]
            .iter()
            .find_map(|k| map.get(*k).and_then(scalar_to_string));

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

/// A JSON string or number as text. Shops routinely emit `"sku": 12345` and
/// `"gtin13": 7312345678901` as bare numbers, which are still identifiers.
fn scalar_to_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
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
        // An `AggregateOffer` wraps the individual offers and carries the
        // price range itself.
        if let Some(nested) = map.get("offers") {
            extract_offers(nested, audit);
        }
        if audit.price.is_none() {
            audit.price = map
                .get("price")
                .or_else(|| map.get("lowPrice"))
                .and_then(scalar_to_string);
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

/// The document's `<title>` texts, in order. Inline `<svg><title>` elements
/// are accessibility labels for icons, not page titles, so they are skipped;
/// counting them reported "multiple titles" on every page with an icon set.
fn document_titles(doc: &Html) -> Vec<String> {
    let Ok(sel) = Selector::parse("title") else {
        return Vec::new();
    };
    doc.select(&sel)
        .filter(|el| {
            !el.ancestors().any(|node| {
                scraper::ElementRef::wrap(node)
                    .is_some_and(|ancestor| ancestor.value().name().eq_ignore_ascii_case("svg"))
            })
        })
        .map(|el| {
            let text: String = el.text().collect::<Vec<_>>().join(" ");
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect()
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

/// The page's own text: the body minus scripts, styles and the chrome every
/// page of the site shares. Navigation, sidebars and the site header and
/// footer are left out, otherwise two product pages that differ only in one
/// paragraph hash as near-identical and a 300-word footer makes every thin
/// page look substantial. A `<header>` or `<footer>` inside an `<article>`,
/// `<section>` or `<main>` belongs to that content and is kept.
fn extract_body_text(doc: &Html) -> String {
    let Ok(body_sel) = Selector::parse("body") else {
        return String::new();
    };
    let Some(body) = doc.select(&body_sel).next() else {
        return String::new();
    };
    let mut out = String::new();
    let mut skip_depth: usize = 0;
    let mut content_section_depth: usize = 0;
    for edge in body.traverse() {
        match edge {
            ego_tree::iter::Edge::Open(node) => {
                if let scraper::node::Node::Element(el) = node.value() {
                    let tag = el.name();
                    if skip_depth == 0 && is_boilerplate_element(el, content_section_depth) {
                        skip_depth = 1;
                        continue;
                    }
                    if skip_depth > 0 {
                        skip_depth += 1;
                        continue;
                    }
                    if matches!(tag, "article" | "section" | "main") {
                        content_section_depth += 1;
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
                    continue;
                }
                if let scraper::node::Node::Element(el) = node.value()
                    && matches!(el.name(), "article" | "section" | "main")
                {
                    content_section_depth = content_section_depth.saturating_sub(1);
                }
                if let scraper::node::Node::Element(el) = node.value()
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

fn is_boilerplate_element(el: &scraper::node::Element, content_section_depth: usize) -> bool {
    let tag = el.name();
    if matches!(
        tag,
        "script" | "style" | "noscript" | "template" | "nav" | "aside"
    ) {
        return true;
    }
    if matches!(tag, "header" | "footer") && content_section_depth == 0 {
        return true;
    }
    el.attr("role").is_some_and(|role| {
        matches!(
            role.trim().to_ascii_lowercase().as_str(),
            "navigation" | "banner" | "contentinfo" | "complementary"
        )
    })
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
        assert_eq!(r.title_count, 1);
        assert_eq!(r.title_2, None);
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
    fn relative_hreflang_hrefs_resolve_against_the_page() {
        let mut record = PageRecord {
            url: "https://example.com/en/shoes".into(),
            ..Default::default()
        };
        analyze_html(
            &mut record,
            r#"<html><head><title>T</title>
            <link rel="alternate" hreflang="de" href="/de/schuhe">
            <link rel="alternate" hreflang="en" href="shoes">
            </head><body></body></html>"#,
            "",
        );
        assert_eq!(
            record.hreflang_tags,
            vec![
                (
                    "de".to_string(),
                    "https://example.com/de/schuhe".to_string()
                ),
                ("en".to_string(), "https://example.com/en/shoes".to_string()),
            ]
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
    fn ecommerce_audit_from_product_inside_graph() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">{
                "@context": "https://schema.org",
                "@graph": [
                    {"@type": "WebSite", "name": "Shop"},
                    {"@type": ["Product", "IndividualProduct"], "name": "Widget",
                     "sku": 12345, "gtin13": 7312345678901,
                     "manufacturer": {"@type": "Organization", "name": "Acme"},
                     "image": "https://example.com/w.jpg",
                     "offers": {"@type": "AggregateOffer", "lowPrice": "5.00", "highPrice": "9.00",
                                "priceCurrency": "SEK",
                                "offers": [{"@type": "Offer", "availability": "https://schema.org/OutOfStock"}]}}
                ]
            }</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
        assert!(r.sd_types.contains(&"IndividualProduct".to_string()));
        let audit = r.ecommerce.as_ref().unwrap();
        assert_eq!(audit.sku.as_deref(), Some("12345"));
        assert_eq!(audit.gtin.as_deref(), Some("7312345678901"));
        assert_eq!(audit.brand.as_deref(), Some("Acme"));
        assert_eq!(audit.price.as_deref(), Some("5.00"));
        assert_eq!(audit.currency.as_deref(), Some("SEK"));
        assert_eq!(audit.availability.as_deref(), Some("outofstock"));
        assert!(audit.has_image);
        let product = r
            .sd_items
            .iter()
            .find(|i| i.type_name == "Product")
            .unwrap();
        assert!(!product.raw_json.contains("WebSite"));
    }

    #[test]
    fn ecommerce_audit_from_product_under_main_entity() {
        let r = analyze(
            r#"<html><head><title>T</title>
            <script type="application/ld+json">[{
                "@type": "WebPage",
                "mainEntity": {"@type": "Product", "name": "Widget",
                    "offers": {"@type": "Offer", "price": "1", "priceCurrency": "USD"}}
            }]</script>
            </head><body></body></html>"#,
        );
        assert!(r.sd_types.contains(&"Product".to_string()));
        assert_eq!(r.ecommerce.as_ref().unwrap().price.as_deref(), Some("1"));
    }

    #[test]
    fn ecommerce_audit_from_microdata_product() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <div itemscope itemtype="https://schema.org/Product">
                <h1 itemprop="name">Widget</h1>
                <img itemprop="image" src="/w.jpg">
                <meta itemprop="sku" content="WDG-1">
                <div itemprop="brand" itemscope itemtype="https://schema.org/Brand">
                    <span itemprop="name">Acme</span>
                </div>
                <div itemprop="offers" itemscope itemtype="https://schema.org/Offer">
                    <span itemprop="price" content="9.99">9,99</span>
                    <meta itemprop="priceCurrency" content="EUR">
                    <link itemprop="availability" href="https://schema.org/InStock">
                </div>
            </div></body></html>"#,
        );
        let audit = r.ecommerce.as_ref().unwrap();
        assert_eq!(audit.sku.as_deref(), Some("WDG-1"));
        assert_eq!(audit.brand.as_deref(), Some("Acme"));
        assert_eq!(audit.price.as_deref(), Some("9.99"));
        assert_eq!(audit.currency.as_deref(), Some("EUR"));
        assert_eq!(audit.availability.as_deref(), Some("instock"));
        assert!(audit.has_image);
        assert!(
            !r.sd_issues
                .iter()
                .any(|i| i.code == "missing-required:name"),
            "microdata name must be seen by the validator"
        );
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
    fn empty_selector_uses_body_minus_site_chrome() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body><nav>Nav</nav><main><p>Content</p></main></body></html>"#,
        );
        assert_eq!(r.word_count, Some(1));
    }

    #[test]
    fn boilerplate_is_left_out_of_content_but_article_headers_are_kept() {
        let r = analyze(
            r#"<html><head><title>T</title></head><body>
            <header>Site header words</header>
            <nav><a href="/">home</a> <a href="/shop">shop</a></nav>
            <div role="navigation">breadcrumb trail here</div>
            <main><article><header>Article title</header><p>Body text</p></article></main>
            <aside>Related products listing</aside>
            <footer>Footer legal words</footer>
            </body></html>"#,
        );
        assert_eq!(r.word_count, Some(4));
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

    #[test]
    fn link_headers_yield_hreflang_alternates() {
        let base = url::Url::parse("https://a.test/se/").expect("base");
        let header = r#"<https://a.test/de/>; rel="alternate"; hreflang="de", </fr/>; rel="alternate"; hreflang="fr""#;
        assert_eq!(
            parse_link_header_hreflang(header, &base),
            vec![
                ("de".to_string(), "https://a.test/de/".to_string()),
                // Relative hrefs resolve against the page, like any other link.
                ("fr".to_string(), "https://a.test/fr/".to_string()),
            ]
        );
    }

    #[test]
    fn link_headers_without_an_alternate_hreflang_are_ignored() {
        let base = url::Url::parse("https://a.test/se/").expect("base");
        // A canonical, a preload and an alternate with no hreflang: the same
        // header carries all of these.
        let header = r#"<https://a.test/se>; rel="canonical", </app.css>; rel=preload; as=style, <https://a.test/feed>; rel="alternate"; type="application/rss+xml""#;
        assert!(parse_link_header_hreflang(header, &base).is_empty());
        assert!(parse_link_header_hreflang("", &base).is_empty());
        assert!(parse_link_header_hreflang("not a link header", &base).is_empty());
    }

    #[test]
    fn merging_another_source_dedups_and_records_it() {
        let mut record = PageRecord {
            url: "https://a.test/se/".into(),
            hreflang_tags: vec![("sv".into(), "https://a.test/se/".into())],
            hreflang_sources: vec![HreflangSource::Html],
            ..Default::default()
        };
        // Already known from the HTML: not a second tag, and not a second
        // source either.
        merge_hreflang_tags(
            &mut record,
            vec![("sv".into(), "https://a.test/se/".into())],
            HreflangSource::HttpHeader,
        );
        assert_eq!(record.hreflang_tags.len(), 1);
        assert_eq!(record.hreflang_sources, vec![HreflangSource::Html]);

        merge_hreflang_tags(
            &mut record,
            vec![("de".into(), "https://a.test/de/".into())],
            HreflangSource::Sitemap,
        );
        assert_eq!(record.hreflang_tags.len(), 2);
        assert_eq!(
            record.hreflang_sources,
            vec![HreflangSource::Html, HreflangSource::Sitemap]
        );
    }

    #[test]
    fn inline_image_payloads_are_replaced_by_a_hash_and_a_size() {
        let payload = "A".repeat(4000);
        let r = analyze_at(
            "https://example.com/page",
            &format!(
                r#"<html><head><title>T</title></head><body>
                <img src="data:image/png;base64,{payload}" alt="Inline">
                <img src="data:image/png;base64,{payload}" alt="Same again">
                <img src="data:image/svg+xml,<svg/>" alt="Plain">
                <img src="/real.png" alt="Fetchable">
                </body></html>"#
            ),
        );
        assert_eq!(r.images.len(), 4);
        // The payload is gone, the media type and encoding stay, and 4000
        // base64 characters are 3000 bytes.
        assert!(
            r.images[0].src.starts_with("data:image/png;base64,#"),
            "got {}",
            r.images[0].src
        );
        assert!(
            r.images[0].src.ends_with("(3000 B)"),
            "got {}",
            r.images[0].src
        );
        assert!(r.images[0].src.len() < 60);
        // The same payload summarises the same way, so one inline image stays
        // one image rather than becoming two.
        assert_eq!(r.images[0].src, r.images[1].src);
        assert_ne!(r.images[0].src, r.images[2].src);
        // A fetchable source is resolved against the page.
        assert_eq!(r.images[3].src, "https://example.com/real.png");
    }

    #[test]
    fn image_alt_stands_in_for_missing_anchor_text() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/home"><img src="/logo.svg" alt="ByLynga"></a>
            <a href="https://example.com/cart"><img src="/cart.svg" alt=""></a>
            <a href="https://example.com/menu"><img src="/menu.svg"></a>
            <a href="https://example.com/about"><img src="/i.svg" alt="Icon"> About</a>
            </body></html>"#,
        );
        assert_eq!(r.outlinks.len(), 4);
        assert_eq!(r.outlinks[0].anchor.as_deref(), Some("ByLynga"));
        // A decorative image carries no anchor text and neither does a missing alt.
        assert_eq!(r.outlinks[1].anchor, None);
        assert_eq!(r.outlinks[2].anchor, None);
        // Real text wins over the alt of an icon sitting beside it.
        assert_eq!(r.outlinks[3].anchor.as_deref(), Some("About"));
    }

    #[test]
    fn multiple_image_alts_in_one_link_are_joined() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/se"><img src="/f.svg" alt="Swedish">
            <img src="/t.svg" alt="  flag  "></a>
            </body></html>"#,
        );
        assert_eq!(r.outlinks[0].anchor.as_deref(), Some("Swedish flag"));
    }

    #[test]
    fn a_document_with_no_body_element_is_recorded_as_missing_one() {
        // A parser invents `<body>`, so this can only be read off the source.
        let without = analyze_at(
            "https://example.com/shell",
            r#"<!doctype html><html lang="en"><head><title>T</title></head>
               <h1>Straight into content</h1><p>No body element anywhere.</p></html>"#,
        );
        assert_eq!(without.has_body_tag, Some(false));

        let with = analyze_at(
            "https://example.com/page",
            r#"<!doctype html><html><head><title>T</title></head><body><h1>H</h1></body></html>"#,
        );
        assert_eq!(with.has_body_tag, Some(true));
    }

    #[test]
    fn an_h2_before_the_first_h1_is_out_of_order() {
        let out_of_order = analyze_at(
            "https://example.com/shell",
            r#"<html><head><title>T</title></head><body>
               <h2>A section</h2><h1>The page</h1><h2>Another section</h2></body></html>"#,
        );
        assert_eq!(out_of_order.h2_non_sequential, Some(true));

        let in_order = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
               <h1>The page</h1><h2>A section</h2></body></html>"#,
        );
        assert_eq!(in_order.h2_non_sequential, Some(false));
    }

    /// Nothing is out of order when there is no order: a page with no H1 has
    /// `Missing H1` as its finding, and counting it here too would report the
    /// same defect twice under two names.
    #[test]
    fn an_h2_on_a_page_with_no_h1_is_not_out_of_order() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
               <h2>A section</h2><p>No h1 anywhere.</p></body></html>"#,
        );
        assert_eq!(r.h2_non_sequential, Some(false));
    }

    #[test]
    fn an_element_whose_name_merely_starts_the_same_does_not_count() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<!doctype html><html><head><title>T</title></head>
               <bodybuilder>not a body</bodybuilder></html>"#,
        );
        assert_eq!(r.has_body_tag, Some(false));
    }

    #[test]
    fn an_aria_label_names_a_link_that_has_no_text() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/next" aria-label="Nästa"><svg class="bi"></svg></a>
            <a href="https://example.com/prev" aria-label="   "><svg class="bi"></svg></a>
            <a href="https://example.com/first" alt="Första"><svg class="bi"></svg></a>
            </body></html>"#,
        );
        assert_eq!(r.outlinks[0].anchor.as_deref(), Some("Nästa"));
        // A whitespace-only label names nothing.
        assert_eq!(r.outlinks[1].anchor, None);
        // `alt` is not a valid attribute of `<a>`; browsers ignore it, so a
        // link carrying only that is genuinely unlabelled.
        assert_eq!(r.outlinks[2].anchor, None);
    }

    #[test]
    fn an_aria_label_overrides_the_text_inside_the_link() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/more" aria-label="Read the full review">More</a>
            </body></html>"#,
        );
        assert_eq!(
            r.outlinks[0].anchor.as_deref(),
            Some("Read the full review"),
            "an author's explicit label is what a browser announces"
        );
    }

    #[test]
    fn lazy_and_responsive_images_are_recorded_by_their_real_urls() {
        let r = analyze_at(
            "https://example.com/p",
            r#"<html><head><title>T</title></head><body>
            <img src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" data-src="/img/a.jpg" alt="A">
            <img src="/img/b.jpg" srcset="/img/b-400.jpg 400w, /img/b-800.jpg 800w" alt="B">
            <picture>
              <source srcset="/img/c.webp" type="image/webp">
              <img src="/img/c.jpg" alt="">
            </picture>
            </body></html>"#,
        );
        let sources: Vec<&str> = r.images.iter().map(|i| i.src.as_str()).collect();
        assert_eq!(
            sources,
            vec![
                "https://example.com/img/a.jpg",
                "https://example.com/img/b.jpg",
                "https://example.com/img/b-400.jpg",
                "https://example.com/img/b-800.jpg",
                "https://example.com/img/c.jpg",
                "https://example.com/img/c.webp",
            ]
        );
        assert!(r.images.iter().all(|i| i.has_alt_attr));
    }

    #[test]
    fn fragment_and_non_navigation_hrefs_are_not_outlinks() {
        let r = analyze_at(
            "https://example.com/page",
            r##"<html><head><title>T</title></head><body>
            <a href="#main">Skip</a>
            <a href="JavaScript:void(0)">Menu</a>
            <a href="mailto:a@b.c">Mail</a>
            <a href="tel:+4612345">Call</a>
            <a href="/shoes?size=42&amp;colour=red#reviews">Shoes</a>
            </body></html>"##,
        );
        let destinations: Vec<&str> = r.outlinks.iter().map(|o| o.dst_url.as_str()).collect();
        assert_eq!(
            destinations,
            vec!["https://example.com/shoes?size=42&colour=red"]
        );
    }

    #[test]
    fn an_inline_svg_title_names_a_link() {
        let r = analyze_at(
            "https://example.com/page",
            r#"<html><head><title>T</title></head><body>
            <a href="https://example.com/next"><svg><title>Next page</title><path/></svg></a>
            </body></html>"#,
        );
        assert_eq!(r.outlinks[0].anchor.as_deref(), Some("Next page"));
    }
}
