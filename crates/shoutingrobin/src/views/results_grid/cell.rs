use std::collections::HashMap;

use gpui::{ParentElement, SharedString};

use crate::crawl::event::{A11yIssue, ImageRef, Outlink, PageRecord, SdFormat, SdItem};
use crate::ui::tag::{Tone, count_tone, indexability_tone, status_code_tone, tone_tag};
use crate::views::ResultTab;

use super::columns::{field_value, header_value, primary_field_key};
use super::data_build::dir_format_size;
use super::types::FlatRow;

pub(super) fn page_address(record: &PageRecord, root_origin: Option<&str>) -> SharedString {
    if record.is_internal
        && let Some(origin) = root_origin
        && let Some(stripped) = record.url.strip_prefix(origin)
    {
        return SharedString::from(if stripped.is_empty() {
            "/".to_string()
        } else {
            stripped.to_string()
        });
    }
    SharedString::from(record.url.clone())
}

pub(super) fn url_to_path(url: &str, root_origin: Option<&str>) -> SharedString {
    let Some(origin) = root_origin else {
        return SharedString::from(url.to_string());
    };
    if let Some(stripped) = url.strip_prefix(origin) {
        if stripped.is_empty() {
            return SharedString::from("/");
        }
        return SharedString::from(stripped.to_string());
    }
    SharedString::from(url.to_string())
}

pub(super) fn flat_cell_text(
    record: &PageRecord,
    row: &FlatRow,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match row {
        FlatRow::Image { item, .. } => {
            let Some(image) = record.images.get(*item) else {
                return SharedString::default();
            };
            image_cell_text(record, image, col_key, root_origin)
        }
        FlatRow::Outlink { item, .. } => {
            let Some(outlink) = record.outlinks.get(*item) else {
                return SharedString::default();
            };
            outlink_cell_text(record, outlink, col_key, root_origin)
        }
        FlatRow::A11yIssue { item, .. } => {
            let Some(issue) = record.a11y_issues.get(*item) else {
                return SharedString::default();
            };
            a11y_cell_text(record, issue, col_key, root_origin)
        }
        FlatRow::Hreflang { item, .. } => {
            if let Some((lang, url)) = record.hreflang_tags.get(*item) {
                hreflang_cell_text(record, lang, url, col_key, root_origin)
            } else {
                match col_key {
                    "address" => page_address(record, root_origin),
                    "indexability" => SharedString::from(
                        record.indexability.clone().unwrap_or_else(|| "-".into()),
                    ),
                    _ => SharedString::from("-"),
                }
            }
        }
        FlatRow::SdItem { item, .. } => {
            if let Some(sd_item) = record.sd_items.get(*item) {
                sd_item_cell_text(record, sd_item, col_key, root_origin)
            } else {
                match col_key {
                    "address" => page_address(record, root_origin),
                    "indexability" => SharedString::from(
                        record.indexability.clone().unwrap_or_else(|| "-".into()),
                    ),
                    _ => SharedString::from("-"),
                }
            }
        }
        FlatRow::IssuesRow { .. } | FlatRow::LinkRow { .. } => {
            SharedString::default()
        }
        FlatRow::DirectoryAggregate {
            path,
            depth,
            page_count,
            avg_word_count,
            total_size,
            non_indexable,
            indexable,
            ..
        } => match col_key {
            "dir_path" => SharedString::from(path.clone()),
            "dir_page_count" => SharedString::from(page_count.to_string()),
            "dir_depth" => SharedString::from(depth.to_string()),
            "dir_avg_words" => SharedString::from(avg_word_count.to_string()),
            "dir_total_size" => SharedString::from(dir_format_size(*total_size)),
            "dir_indexable" => SharedString::from(indexable.to_string()),
            "dir_non_indexable" => SharedString::from(non_indexable.to_string()),
            _ => SharedString::default(),
        },
    }
}

fn image_cell_text(
    record: &PageRecord,
    image: &ImageRef,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => page_address(record, root_origin),
        "image_src" => SharedString::from(image.src.clone()),
        "image_alt" => SharedString::from(image.alt.clone().unwrap_or_default()),
        "image_width" => SharedString::from(
            image
                .width
                .map(|w| w.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
        "image_height" => SharedString::from(
            image
                .height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
        "image_has_alt" => SharedString::from(if image.has_alt_attr { "Yes" } else { "No" }),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        _ => SharedString::default(),
    }
}

fn outlink_cell_text(
    record: &PageRecord,
    outlink: &Outlink,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => page_address(record, root_origin),
        "outlink_dst" => SharedString::from(outlink.dst_url.clone()),
        "outlink_anchor" => SharedString::from(outlink.anchor.clone().unwrap_or_default()),
        "outlink_rel" => SharedString::from(outlink.rel.clone().unwrap_or_default()),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        _ => SharedString::default(),
    }
}

fn a11y_cell_text(
    record: &PageRecord,
    issue: &A11yIssue,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => page_address(record, root_origin),
        "a11y_rule" => SharedString::from(issue.rule.clone()),
        "a11y_impact" => SharedString::from(issue.impact.clone()),
        "a11y_target" => SharedString::from(issue.target.clone().unwrap_or_default()),
        "a11y_html" => SharedString::from(issue.html.clone().unwrap_or_default()),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        _ => SharedString::default(),
    }
}

fn hreflang_cell_text(
    record: &PageRecord,
    lang: &str,
    url: &str,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => page_address(record, root_origin),
        "hreflang_lang" => SharedString::from(lang.to_string()),
        "hreflang_url" => SharedString::from(url.to_string()),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        _ => SharedString::default(),
    }
}

fn sd_item_cell_text(
    record: &PageRecord,
    sd_item: &SdItem,
    col_key: &str,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => page_address(record, root_origin),
        "sd_format" => SharedString::from(match sd_item.format {
            SdFormat::JsonLd => "JSON-LD",
            SdFormat::Microdata => "Microdata",
        }),
        "sd_type" => SharedString::from(sd_item.type_name.clone()),
        "sd_raw" => {
            let truncated = if sd_item.raw_json.len() > 200 {
                format!("{}...", &sd_item.raw_json[..200])
            } else {
                sd_item.raw_json.clone()
            };
            SharedString::from(truncated)
        }
        "sd_errors" => SharedString::from(record.sd_errors.to_string()),
        "sd_warnings" => SharedString::from(record.sd_warnings.to_string()),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        _ => SharedString::default(),
    }
}

pub(super) fn cell_text(
    record: &PageRecord,
    col_key: &str,
    occurrence_counts: &HashMap<String, usize>,
    tab: ResultTab,
    root_origin: Option<&str>,
) -> SharedString {
    match col_key {
        "address" => {
            if record.is_internal {
                if let Some(origin) = root_origin {
                    if let Some(stripped) = record.url.strip_prefix(origin) {
                        SharedString::from(if stripped.is_empty() { "/" } else { stripped })
                    } else {
                        SharedString::from(record.url.clone())
                    }
                } else {
                    SharedString::from(record.url.clone())
                }
            } else {
                SharedString::from(record.url.clone())
            }
        }
        "content" => SharedString::from(record.content_type.clone().unwrap_or_default()),
        "status_code" => {
            if record.redirect_url.is_some() {
                record
                    .redirect_status
                    .map(|s| SharedString::from(s.to_string()))
                    .unwrap_or_else(|| SharedString::from("301"))
            } else {
                record
                    .status
                    .map(|s| SharedString::from(s.to_string()))
                    .unwrap_or_else(|| SharedString::from("-"))
            }
        }
        "status" => status_label(record),
        "indexability" => {
            SharedString::from(record.indexability.clone().unwrap_or_else(|| "-".into()))
        }
        "title" => SharedString::from(record.title.clone().unwrap_or_default()),
        "title_length" => record
            .title
            .as_ref()
            .map(|t| SharedString::from(t.len().to_string()))
            .unwrap_or_else(|| SharedString::from("0")),
        "title_pixel_width" => record
            .title_pixel_width
            .map(|w| SharedString::from(w.to_string()))
            .unwrap_or_else(|| SharedString::from("-")),
        "meta_desc" => SharedString::from(record.meta_description.clone().unwrap_or_default()),
        "meta_desc_length" => record
            .meta_description
            .as_ref()
            .map(|d| SharedString::from(d.len().to_string()))
            .unwrap_or_else(|| SharedString::from("0")),
        "meta_desc_pixel_width" => record
            .meta_description_pixel_width
            .map(|w| SharedString::from(w.to_string()))
            .unwrap_or_else(|| SharedString::from("-")),
        "h1" => SharedString::from(record.h1.clone().unwrap_or_default()),
        "h1_length" => record
            .h1
            .as_ref()
            .map(|h| SharedString::from(h.len().to_string()))
            .unwrap_or_else(|| SharedString::from("0")),
        "h2" => SharedString::from(record.h2.clone().unwrap_or_default()),
        "h2_length" => record
            .h2
            .as_ref()
            .map(|h| SharedString::from(h.len().to_string()))
            .unwrap_or_else(|| SharedString::from("0")),
        "canonical" => SharedString::from(record.canonical.clone().unwrap_or_default()),
        "robots" => SharedString::from(record.robots.clone().unwrap_or_default()),
        "size" => format_size(record.size_bytes),
        "words" => SharedString::from(
            record
                .word_count
                .map(|w| w.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
        "depth" => SharedString::from(record.depth.to_string()),
        "response_time" => SharedString::from(format!("{}ms", record.response_time.as_millis())),
        "closest_similarity" => SharedString::from(
            record
                .closest_similarity
                .map(|s| format!("{s}%"))
                .unwrap_or_else(|| "-".into()),
        ),
        "near_duplicate_count" => SharedString::from(
            record
                .near_duplicate_count
                .map(|c| c.to_string())
                .unwrap_or_else(|| "-".into()),
        ),
        "occurrences" => {
            let Some(field_key) = primary_field_key(tab) else {
                return SharedString::from("-");
            };
            let val = field_value(record, field_key).unwrap_or("");
            let count = occurrence_counts
                .get(val)
                .copied()
                .unwrap_or(if val.is_empty() { 0 } else { 1 });
            SharedString::from(count.to_string())
        }
        "hreflang_count" => SharedString::from(record.hreflang_tags.len().to_string()),
        "hreflang_tags" => {
            let tags: Vec<String> = record
                .hreflang_tags
                .iter()
                .map(|(lang, url)| format!("{lang}: {url}"))
                .collect();
            SharedString::from(tags.join(", "))
        }
        "sd_types" => SharedString::from(record.sd_types.join(", ")),
        "sd_total_types" => SharedString::from(record.sd_items.len().to_string()),
        "sd_jsonld" => SharedString::from(record.sd_jsonld_count.to_string()),
        "sd_microdata" => SharedString::from(record.sd_microdata_count.to_string()),
        "sd_errors" => SharedString::from(record.sd_errors.to_string()),
        "sd_warnings" => SharedString::from(record.sd_warnings.to_string()),
        "ttfb" => SharedString::from(
            record
                .ttfb_ms
                .map(|ms| format!("{ms}ms"))
                .unwrap_or_else(|| "-".into()),
        ),
        "lcp" => SharedString::from(
            record
                .lcp_ms
                .map(|ms| format!("{ms}ms"))
                .unwrap_or_else(|| "-".into()),
        ),
        "cls" => SharedString::from(
            record
                .cls
                .map(|v| format!("{v:.3}"))
                .unwrap_or_else(|| "-".into()),
        ),
        "inp" => SharedString::from(
            record
                .inp_ms
                .map(|ms| format!("{ms}ms"))
                .unwrap_or_else(|| "-".into()),
        ),
        "in_sitemap" => SharedString::from(
            record
                .in_sitemap
                .map(|v| if v { "Yes" } else { "No" })
                .unwrap_or("-"),
        ),
        "sitemap_url" => SharedString::from(record.sitemap_url.clone().unwrap_or_default()),
        "ecom_price" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.price.clone())
                .unwrap_or_default(),
        ),
        "ecom_currency" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.currency.clone())
                .unwrap_or_default(),
        ),
        "ecom_availability" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.availability.clone())
                .unwrap_or_default(),
        ),
        "ecom_sku" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.sku.clone())
                .unwrap_or_default(),
        ),
        "ecom_gtin" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.gtin.clone())
                .unwrap_or_default(),
        ),
        "ecom_brand" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .and_then(|a| a.brand.clone())
                .unwrap_or_default(),
        ),
        "ecom_has_image" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .map(|a| if a.has_image { "Yes" } else { "No" })
                .unwrap_or("-"),
        ),
        "ecom_has_review" => SharedString::from(
            record
                .ecommerce
                .as_ref()
                .map(|a| if a.has_review_or_rating { "Yes" } else { "No" })
                .unwrap_or("-"),
        ),
        "inlinks" => SharedString::from(if record.inlinks_count > 0 {
            record.inlinks_count.to_string()
        } else {
            "0".into()
        }),
        "outlinks_count" => SharedString::from(record.outlinks.len().to_string()),
        "folder_depth" => {
            let depth = url::Url::parse(&record.url)
                .ok()
                .map(|u| u.path().matches('/').count().saturating_sub(1) as u32)
                .unwrap_or(0);
            SharedString::from(depth.to_string())
        }
        "indexability_status" => SharedString::from(compute_indexability_status(record)),
        "content_hash" => SharedString::from(record.content_hash.as_deref().unwrap_or("-")),
        "sec_https" => SharedString::from(if record.url.starts_with("https://") {
            "Yes"
        } else {
            "No"
        }),
        "sec_hsts" => SharedString::from(
            header_value(&record.headers, "strict-transport-security")
                .map(|_| "Yes")
                .unwrap_or("No"),
        ),
        "sec_csp" => SharedString::from(
            header_value(&record.headers, "content-security-policy")
                .map(|_| "Yes")
                .unwrap_or("No"),
        ),
        "sec_frame_guard" => SharedString::from(
            header_value(&record.headers, "x-frame-options")
                .map(|_| "Yes")
                .unwrap_or("No"),
        ),
        "sec_content_type_opts" => SharedString::from(
            header_value(&record.headers, "x-content-type-options")
                .map(|_| "Yes")
                .unwrap_or("No"),
        ),
        "last_modified" => {
            SharedString::from(header_value(&record.headers, "last-modified").unwrap_or("-"))
        }
        "redirect_url" => SharedString::from(record.redirect_url.as_deref().unwrap_or("-")),
        "url_length" => SharedString::from(record.url.len().to_string()),
        "x_robots_tag" => {
            SharedString::from(header_value(&record.headers, "x-robots-tag").unwrap_or("-"))
        }
        "a11y_errors" => SharedString::from(if record.a11y_errors > 0 {
            record.a11y_errors.to_string()
        } else {
            "0".into()
        }),
        "a11y_warnings" => SharedString::from(if record.a11y_warnings > 0 {
            record.a11y_warnings.to_string()
        } else {
            "0".into()
        }),
        "link_score" => SharedString::from(
            record
                .link_score
                .map(|s| format!("{s:.1}"))
                .unwrap_or_else(|| "-".into()),
        ),
        "title_2" => SharedString::from(record.title_2.as_deref().unwrap_or("-")),
        "meta_desc_2" => SharedString::from(record.meta_description_2.as_deref().unwrap_or("-")),
        "h1_2" => SharedString::from(record.h1_2.as_deref().unwrap_or("-")),
        "h2_2" => SharedString::from(record.h2_2.as_deref().unwrap_or("-")),
        _ => SharedString::default(),
    }
}

fn status_label(record: &PageRecord) -> SharedString {
    match record.status {
        Some(c) if (200..300).contains(&c) => SharedString::from("OK"),
        Some(c) if (300..400).contains(&c) => SharedString::from("Redirect"),
        Some(c) if (400..500).contains(&c) => SharedString::from("Client Err"),
        Some(c) if c >= 500 => SharedString::from("Server Err"),
        _ => SharedString::from("-"),
    }
}

fn compute_indexability_status(record: &PageRecord) -> String {
    if record.status.is_none_or(|c| !(200..300).contains(&c)) {
        return record
            .status
            .map(|c| format!("Non-Indexable ({c})"))
            .unwrap_or_else(|| "Non-Indexable".into());
    }
    if record
        .robots
        .as_deref()
        .is_some_and(|r| r.to_ascii_lowercase().contains("noindex"))
    {
        return "Noindex Meta Tag".into();
    }
    if record
        .canonical
        .as_deref()
        .is_some_and(|c| !c.is_empty() && c != record.url)
    {
        return "Canonicalised".into();
    }
    "Indexable".into()
}

fn format_size(bytes: u64) -> SharedString {
    if bytes == 0 {
        return SharedString::from("-");
    }
    if bytes < 1024 {
        return SharedString::from(format!("{bytes} B"));
    }
    if bytes < 1024 * 1024 {
        return SharedString::from(format!("{:.1} KB", bytes as f64 / 1024.0));
    }
    SharedString::from(format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0)))
}

pub(super) fn render_cell_tag(
    record: &PageRecord,
    col_key: &str,
    text: &SharedString,
) -> Option<gpui_component::tag::Tag> {
    let tone = match col_key {
        "status_code" => {
            if record.redirect_url.is_some() {
                status_code_tone(record.redirect_status.unwrap_or(301))
            } else {
                record.status.map(status_code_tone).unwrap_or(Tone::Neutral)
            }
        }
        "indexability" => indexability_tone(text),
        "in_sitemap" => match text.as_ref() {
            "Yes" => Tone::Ok,
            "No" => Tone::Warn,
            _ => return None,
        },
        "ecom_has_image" | "ecom_has_review" | "image_has_alt" => match text.as_ref() {
            "Yes" => Tone::Ok,
            "No" => Tone::Warn,
            _ => return None,
        },
        "sd_errors" => count_tone(record.sd_errors as i64, Tone::Err),
        "sd_warnings" => count_tone(record.sd_warnings as i64, Tone::Warn),
        "sd_format" => Tone::Accent,
        "a11y_impact" => match text.as_ref() {
            "critical" | "serious" => Tone::Err,
            "moderate" => Tone::Warn,
            _ => Tone::Neutral,
        },
        "near_duplicate_count" => match record.near_duplicate_count {
            Some(c) if c > 0 => Tone::Warn,
            Some(_) => Tone::Ok,
            None => return None,
        },
        "indexability_status" => {
            if text.starts_with("Non-Indexable") || text.starts_with("Noindex") {
                Tone::Err
            } else if text.starts_with("Canonicalised") {
                Tone::Warn
            } else {
                Tone::Ok
            }
        }
        "sec_https" | "sec_hsts" | "sec_csp" | "sec_frame_guard" | "sec_content_type_opts" => {
            match text.as_ref() {
                "Yes" => Tone::Ok,
                "No" => Tone::Warn,
                _ => return None,
            }
        }
        _ => return None,
    };
    Some(tone_tag(tone).child(text.clone()))
}
