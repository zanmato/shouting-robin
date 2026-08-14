use std::collections::HashMap;

use gpui::{SharedString, px};
use gpui_component::table::{Column, ColumnFixed};

use crate::crawl::event::PageRecord;
use crate::views::ResultTab;

/// The most hreflang column pairs the Hreflang tab will show. A page with a
/// pathological number of tags would otherwise make the grid unusably wide.
const MAX_HREFLANG_COLUMNS: usize = 10;

/// How many hreflang column pairs the loaded crawl needs: the largest number of
/// tags any one page carries, so no page's tags are cut off unless a page
/// exceeds the cap.
pub(super) fn hreflang_column_count(pages: &[PageRecord]) -> usize {
    pages
        .iter()
        .map(|page| page.hreflang_tags.len())
        .max()
        .unwrap_or(0)
        .min(MAX_HREFLANG_COLUMNS)
}

pub(super) fn columns_for_tab(tab: ResultTab, hreflang_columns: usize) -> Vec<Column> {
    match tab {
        ResultTab::Internal => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("status", "Status", 90., None),
            col("indexability", "Indexability", 110., None),
            col("indexability_status", "Index. Status", 120., None),
            col("title", "Title", 280., None),
            col("title_length", "Title Len", 80., None),
            col("meta_desc", "Meta Desc", 280., None),
            col("h1", "H1", 220., None),
            col("h2", "H2", 220., None),
            col("robots", "Meta Robots", 120., None),
            col("canonical", "Canonical", 280., None),
            col("size", "Size", 80., None),
            col("words", "Words", 80., None),
            col("depth", "Depth", 60., None),
            col("folder_depth", "Folder Depth", 90., None),
            col("inlinks", "Inlinks", 70., None),
            col("unique_inlinks", "Unique In", 85., None),
            col("csr_inlinks", "CSR In", 70., None),
            col("csr_inlinks_pct", "CSR In %", 80., None),
            col("outlinks_count", "Outlinks", 70., None),
            col("unique_outlinks", "Unique Out", 90., None),
            col("external_outlinks", "Ext. Out", 75., None),
            col("unique_external_outlinks", "Unique Ext. Out", 120., None),
            col("csr_outlinks", "CSR Out", 70., None),
            col("csr_outlinks_pct", "CSR Out %", 80., None),
            col("last_modified", "Last Modified", 130., None),
            col("redirect_url", "Redirect URI", 350., None),
            col("closest_similarity", "Closest Sim.", 90., None),
            col("near_duplicate_count", "Near Dups", 80., None),
            col("link_score", "Link Score", 80., None),
        ],
        ResultTab::External => vec![
            col("address", "Address", 420., Some(ColumnFixed::Left)),
            col("content", "Content Type", 160., None),
            col("status_code", "Code", 70., None),
            col("status", "Status", 100., None),
            col("size", "Size", 90., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::ResponseCodes => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("status", "Status", 90., None),
            col("indexability", "Indexability", 110., None),
            col("redirect_url", "Redirect URI", 350., None),
        ],
        ResultTab::PageTitles => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("title", "Title", 350., None),
            col("title_2", "Title 2", 350., None),
            col("title_length", "Title Len", 90., None),
            col("title_pixel_width", "Pixel Width", 90., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::MetaDesc => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("meta_desc", "Meta Desc", 350., None),
            col("meta_desc_2", "Meta Desc 2", 350., None),
            col("meta_desc_length", "Meta Desc Len", 110., None),
            col("meta_desc_pixel_width", "Pixel Width", 90., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::H1 => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("h1", "H1", 300., None),
            col("h1_2", "H1-2", 300., None),
            col("h1_length", "H1 Len", 80., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::H2 => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("h2", "H2", 300., None),
            col("h2_2", "H2-2", 300., None),
            col("h2_length", "H2 Len", 80., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Content => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("content_hash", "Hash", 100., None),
            col("words", "Words", 80., None),
            col("closest_similarity", "Closest Sim.", 90., None),
            col("near_duplicate_count", "Near Dups", 80., None),
            col("ssr_words", "SSR Words", 90., None),
            col("ssr_diff", "SSR Diff", 90., None),
            col("body_tag", "Body Tag", 80., None),
            col("indexability", "Indexability", 110., None),
        ],
        // One row per unique image source. A footer logo referenced from every
        // page was a row per page before; the pages referencing an image are
        // the details panel's Referenced By section.
        ResultTab::Images => vec![
            col("image_src", "Src", 420., Some(ColumnFixed::Left)),
            col("image_alt", "Alt Text", 260., None),
            col("image_inlinks", "IMG Inlinks", 100., None),
            col("image_status", "Code", 70., None),
            col("image_size", "Size", 90., None),
            col("image_width", "Width", 80., None),
            col("image_height", "Height", 80., None),
            col("image_has_alt", "Has Alt", 80., None),
        ],
        ResultTab::Canonicals => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("canonical", "Canonical", 350., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        // One row per URL, with a language/URL column pair per tag, the same
        // treatment Title/Title 2 and H1/H1-2 already get. A row per tag put
        // 614 rows against 125 URLs on the reference crawl.
        ResultTab::Hreflang => {
            let mut cols = vec![
                col("address", "Address", 380., Some(ColumnFixed::Left)),
                col("hreflang_count", "Tags", 70., None),
                col("hreflang_sources", "Source", 110., None),
            ];
            for pair in 1..=hreflang_columns {
                cols.push(col(
                    &format!("hreflang_{pair}"),
                    &format!("hreflang {pair}"),
                    110.,
                    None,
                ));
                cols.push(col(
                    &format!("hreflang_{pair}_url"),
                    &format!("hreflang {pair} URL"),
                    320.,
                    None,
                ));
            }
            cols.push(col("indexability", "Indexability", 110., None));
            cols
        }
        ResultTab::StructuredData => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("sd_format", "Format", 100., None),
            col("sd_type", "Type", 200., None),
            col("sd_errors", "Errors", 70., None),
            col("sd_warnings", "Warnings", 80., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Sitemaps => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("status", "Status", 90., None),
            col("indexability", "Indexability", 110., None),
            col("in_sitemap", "In Sitemap", 90., None),
            col("sitemap_lastmod", "Last Mod", 130., None),
            col("sitemap_url", "Sitemap URL", 300., None),
        ],
        ResultTab::Accessibility => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("a11y_rule", "Rule", 150., None),
            col("a11y_impact", "Impact", 100., None),
            col("a11y_target", "Target", 250., None),
            col("a11y_html", "HTML", 300., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Performance => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("status_code", "Code", 70., None),
            col("ttfb", "TTFB", 80., None),
            col("fcp", "FCP", 80., None),
            col("lcp", "LCP", 80., None),
            col("cls", "CLS", 80., None),
        ],
        ResultTab::Ecommerce => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("title", "Title", 280., None),
            col("ecom_price", "Price", 80., None),
            col("ecom_currency", "Currency", 80., None),
            col("ecom_availability", "Availability", 100., None),
            col("ecom_sku", "SKU", 120., None),
            col("ecom_gtin", "GTIN", 120., None),
            col("ecom_brand", "Brand", 120., None),
            col("ecom_has_image", "Has Image", 80., None),
            col("ecom_has_review", "Has Review", 90., None),
            col("status_code", "Code", 70., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Security => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("sec_https", "HTTPS", 70., None),
            col("sec_hsts", "HSTS", 70., None),
            col("sec_csp", "CSP", 70., None),
            col("sec_frame_guard", "X-Frame", 80., None),
            col("sec_content_type_opts", "X-Content-Type", 110., None),
            col("sec_referrer_policy", "Referrer-Policy", 120., None),
            col("sec_mixed_content", "Mixed Content", 110., None),
        ],
        ResultTab::Url => vec![
            col("address", "Address", 450., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("url_length", "Length", 80., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Directives => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("robots", "Meta Robots", 150., None),
            col("x_robots_tag", "X-Robots-Tag", 200., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Overview => vec![
            col("issue_name", "Issue", 360., Some(ColumnFixed::Left)),
            col("issue_type", "Type", 110., None),
            col("priority", "Priority", 90., None),
            col("count", "URLs", 70., None),
            col("pct", "% of Total", 80., None),
            col("description", "Description", 300., None),
            col("hint", "Hint", 300., None),
        ],
        // One row per URL with its link counts. The individual links behind
        // these figures are the details panel's Inlinks and Outlinks sections:
        // a row per link instance put a 50k-page site's million-odd links in
        // one in-memory grid, and the counts are what the tab is read for.
        // Each link count is followed by its CSR half: how much of it exists
        // only after rendering. The pair reads as one question — "how much of
        // this page's link graph does a crawler without JavaScript see?" — so
        // the columns sit together rather than in a block of their own.
        ResultTab::Links => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("inlinks", "Inlinks", 80., None),
            col("unique_inlinks", "Unique In", 90., None),
            col("csr_inlinks", "CSR In", 80., None),
            col("unique_csr_inlinks", "Unique CSR In", 115., None),
            col("outlinks_count", "Outlinks", 80., None),
            col("unique_outlinks", "Unique Out", 95., None),
            col("csr_outlinks", "CSR Out", 85., None),
            col("unique_csr_outlinks", "Unique CSR Out", 120., None),
            col("external_outlinks", "Ext. Out", 80., None),
            col("unique_external_outlinks", "Unique Ext. Out", 120., None),
            col("external_csr_outlinks", "Ext. CSR Out", 110., None),
            col(
                "unique_external_csr_outlinks",
                "Unique Ext. CSR Out",
                145.,
                None,
            ),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::SiteStructure => vec![
            col("dir_path", "Directory", 400., Some(ColumnFixed::Left)),
            col("dir_page_count", "Pages", 80., None),
            col("dir_depth", "Depth", 70., None),
            col("dir_avg_words", "Avg Words", 100., None),
            col("dir_total_size", "Total Size", 100., None),
            col("dir_indexable", "Indexable", 80., None),
            col("dir_non_indexable", "Non-Idx", 80., None),
        ],
        ResultTab::Changes => vec![
            col("change_url", "Address", 420., Some(ColumnFixed::Left)),
            col("change_kind", "Change", 100., None),
            col("change_status", "Status", 120., None),
            col("change_detail", "Detail", 400., None),
        ],
    }
}

/// Returns the columns for a tab, adding the comparison columns (`Prev`/`Δ`) to
/// the Overview tab when a baseline crawl is active.
pub(super) fn columns_for_tab_with_baseline(
    tab: ResultTab,
    has_baseline: bool,
    hreflang_columns: usize,
) -> Vec<Column> {
    let mut cols = columns_for_tab(tab, hreflang_columns);
    if tab == ResultTab::Overview
        && has_baseline
        && let Some(pos) = cols.iter().position(|c| c.key.as_ref() == "count")
    {
        cols.insert(pos + 1, col("count_prev", "Prev", 70., None));
        cols.insert(pos + 2, col("count_delta", "Δ", 70., None));
    }
    cols
}

pub(super) fn primary_field_key(tab: ResultTab) -> Option<&'static str> {
    match tab {
        ResultTab::PageTitles => Some("title"),
        ResultTab::MetaDesc => Some("meta_description"),
        ResultTab::H1 => Some("h1"),
        ResultTab::H2 => Some("h2"),
        ResultTab::Canonicals => Some("canonical"),
        _ => None,
    }
}

pub(super) fn field_value<'a>(record: &'a PageRecord, field: &str) -> Option<&'a str> {
    match field {
        "title" => record.title.as_deref(),
        "meta_description" => record.meta_description.as_deref(),
        "h1" => record.h1.as_deref(),
        "h2" => record.h2.as_deref(),
        "canonical" => record.canonical.as_deref(),
        _ => None,
    }
}

pub(super) fn field_count(record: &PageRecord, field: &str) -> u32 {
    match field {
        "title" => record.title_count,
        "h1" => record.h1_count,
        "h2" => record.h2_count,
        _ => 1,
    }
}

/// The length of `text` in characters, which is what the `LENGTH` columns and
/// the over/under-length thresholds mean.
///
/// `str::len()` returns bytes, so every non-ASCII character inflates the figure:
/// `Kvalitetsbett för dig och din häst | ByLynga` is 44 characters but 46 bytes.
/// On a Swedish site that was most of the pages reporting a length nobody could
/// reproduce by counting, and the same skew silently moved pages in and out of
/// the over/under-length filters.
pub(super) fn char_length(text: &str) -> usize {
    text.chars().count()
}

/// The (minimum, maximum) sensible character count per tab. The minimums are
/// the point below which a snippet leaves search-result space unused; the
/// maximums the point where it starts being truncated.
///
/// The meta description minimum is 70 rather than the 50 we used before: 50
/// characters is around a third of the space a description is given, so pages
/// well short of a usable snippet were passing.
pub(super) fn length_thresholds(tab: ResultTab) -> Option<(usize, usize)> {
    match tab {
        ResultTab::PageTitles => Some((30, 60)),
        ResultTab::MetaDesc => Some((70, 160)),
        ResultTab::H1 => Some((1, 70)),
        ResultTab::H2 => Some((1, 70)),
        _ => None,
    }
}

/// The same, in rendered pixels, which is what search engines actually truncate
/// on. See `crawl::font_metrics` for how the widths are measured.
///
/// The meta description minimum is 400 rather than 200: at the 14px description
/// size 200px is barely two words, which no page would ever fall below.
pub(super) fn pixel_width_thresholds(tab: ResultTab) -> Option<(u32, u32)> {
    match tab {
        ResultTab::PageTitles => Some((200, 580)),
        ResultTab::MetaDesc => Some((400, 970)),
        _ => None,
    }
}

pub(super) fn build_occurrence_counts(
    tab: ResultTab,
    pages: &[PageRecord],
) -> HashMap<String, usize> {
    let Some(key) = primary_field_key(tab) else {
        return HashMap::new();
    };
    let mut counts: HashMap<String, usize> = HashMap::new();
    for page in pages {
        // Only count pages eligible for content-issue flags, so a noindex or
        // redirected page sharing a title/H1 with a real page doesn't inflate
        // the duplicate count and mislabel the indexable page as duplicated.
        if !super::filter::is_content_eligible(page) {
            continue;
        }
        let val = field_value(page, key).unwrap_or("");
        if val.is_empty() {
            continue;
        }
        *counts.entry(val.to_string()).or_insert(0) += 1;
    }
    counts
}

pub(super) fn header_exists(headers: &[(String, String)], name: &str) -> bool {
    headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

pub(super) fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}

pub(super) fn is_numeric_column(key: &str) -> bool {
    matches!(
        key,
        "status_code"
            | "title_length"
            | "title_pixel_width"
            | "meta_desc_length"
            | "meta_desc_pixel_width"
            | "h1_length"
            | "h2_length"
            | "size"
            | "words"
            | "depth"
            | "folder_depth"
            | "inlinks"
            | "unique_inlinks"
            | "outlinks_count"
            | "unique_outlinks"
            | "external_outlinks"
            | "unique_external_outlinks"
            | "csr_inlinks"
            | "csr_outlinks"
            | "unique_csr_inlinks"
            | "unique_csr_outlinks"
            | "external_csr_outlinks"
            | "unique_external_csr_outlinks"
            | "csr_inlinks_pct"
            | "csr_outlinks_pct"
            | "closest_similarity"
            | "near_duplicate_count"
            | "occurrences"
            | "sd_errors"
            | "sd_warnings"
            | "ttfb"
            | "fcp"
            | "lcp"
            | "cls"
            | "image_width"
            | "image_height"
            | "image_inlinks"
            | "image_status"
            | "url_length"
            | "hreflang_count"
            | "dir_page_count"
            | "dir_depth"
            | "dir_avg_words"
            | "dir_indexable"
            | "dir_non_indexable"
            | "ssr_words"
            | "ssr_diff"
    )
}

/// Columns whose cells are read as quantities and so are right aligned, digits
/// under digits.
///
/// Wider than [`is_numeric_column`], which also decides how a column sorts: a
/// formatted size ("1.2 KB") and a percentage read as numbers but must not be
/// compared as one, or 900 B would sort above 1.2 KB.
pub(super) fn is_right_aligned_column(key: &str) -> bool {
    is_numeric_column(key)
        || matches!(
            key,
            "size"
                | "image_size"
                | "dir_total_size"
                | "link_score"
                | "count"
                | "count_prev"
                | "count_delta"
                | "pct"
                | "response_time"
        )
}

pub(super) fn is_tag_column(key: &str) -> bool {
    matches!(
        key,
        "status_code"
            | "indexability"
            | "in_sitemap"
            | "ecom_has_image"
            | "ecom_has_review"
            | "sd_errors"
            | "sd_warnings"
            | "sd_format"
            | "a11y_impact"
            | "near_duplicate_count"
            | "image_has_alt"
            | "image_status"
            | "indexability_status"
            | "sec_https"
            | "sec_hsts"
            | "sec_csp"
            | "sec_frame_guard"
            | "sec_content_type_opts"
            | "sec_referrer_policy"
            | "sec_mixed_content"
            | "body_tag"
    )
}

pub(super) fn is_mono_column(key: &str) -> bool {
    is_numeric_column(key)
        || is_tag_column(key)
        || matches!(
            key,
            "address"
                | "image_src"
                | "canonical"
                | "a11y_target"
                | "a11y_html"
                | "last_modified"
                | "sitemap_lastmod"
                | "redirect_url"
                | "dir_path"
                | "change_url"
        )
}

pub(super) fn compare_numeric(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parsed = parse_numeric(a);
    let b_parsed = parse_numeric(b);
    match (a_parsed, b_parsed) {
        (Some(a_num), Some(b_num)) => a_num
            .partial_cmp(&b_num)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.cmp(b),
    }
}

fn parse_numeric(s: &str) -> Option<f64> {
    if let Ok(v) = s.parse::<f64>() {
        return Some(v);
    }
    let trimmed = s.trim();
    let end = trimmed
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_ascii_digit())
        .map(|(i, _)| i + 1)
        .unwrap_or(0);
    if end == 0 {
        return None;
    }
    trimmed[..end].parse::<f64>().ok()
}

fn col(key: &str, name: &str, width: f32, fixed: Option<ColumnFixed>) -> Column {
    Column {
        key: SharedString::from(key.to_string()),
        name: SharedString::from(name.to_uppercase()),
        width: px(width),
        fixed,
        ..Default::default()
    }
    .sortable()
}
