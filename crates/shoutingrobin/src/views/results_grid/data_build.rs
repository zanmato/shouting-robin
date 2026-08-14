use std::collections::HashMap;

use crate::crawl::engine::is_same_domain;
use crate::crawl::event::{HreflangIssue, PageRecord};
use crate::views::ResultTab;

use super::columns::{char_length, header_exists};
use super::types::{
    ChangeEntry, ChangeKind, FlatRow, ImageAggregateRow, IssueEntry, IssueFilter, IssuePriority,
    IssueType,
};

/// The rows one page contributes to a flattened tab that lists per-item rows.
fn page_item_rows(tab: ResultTab, page_index: usize, page: &PageRecord) -> Vec<FlatRow> {
    match tab {
        ResultTab::Accessibility => (0..page.a11y_issues.len())
            .map(|item| FlatRow::A11yIssue {
                page: page_index,
                item,
            })
            .collect(),
        // A page with no structured data still gets a row, so the tab can list
        // it as missing rather than dropping it.
        ResultTab::StructuredData => (0..page.sd_items.len().max(1))
            .map(|item| FlatRow::SdItem {
                page: page_index,
                item,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Whether an Overview issue entry belongs to the given sub-filter. Shared by
/// the row filtering and the counting engine so the two never diverge.
pub(super) fn issue_entry_matches(entry: &IssueEntry, filter: IssueFilter) -> bool {
    match filter {
        IssueFilter::IssueTypeError => entry.issue_type == IssueType::Issue,
        IssueFilter::IssueTypeOpportunity => entry.issue_type == IssueType::Opportunity,
        IssueFilter::IssueTypeWarning => entry.issue_type == IssueType::Warning,
        IssueFilter::PriorityHigh => entry.priority == IssuePriority::High,
        IssueFilter::PriorityMedium => entry.priority == IssuePriority::Medium,
        IssueFilter::PriorityLow => entry.priority == IssuePriority::Low,
        _ => true,
    }
}

/// Whether a Changes entry belongs to the given sub-filter. Shared by the row
/// filtering and the counting engine.
pub(super) fn change_entry_matches(entry: &ChangeEntry, filter: IssueFilter) -> bool {
    match filter {
        IssueFilter::ChangeAdded => entry.kind == ChangeKind::Added,
        IssueFilter::ChangeRemoved => entry.kind == ChangeKind::Removed,
        IssueFilter::ChangeChanged => entry.kind == ChangeKind::Changed,
        _ => true,
    }
}

/// Builds the full, unfiltered flat-row universe for a flattened tab from the
/// given gated page indices. Factored out of `rebuild_flat_rows` so that the
/// displayed grid and the counting engine share one row universe. `page_indices`
/// is ignored for tabs whose rows are not page-scoped (Overview issues, Changes,
/// directory aggregates).
pub(super) fn build_rows_for_tab(
    tab: ResultTab,
    page_indices: &[usize],
    pages: &[PageRecord],
    change_entries: &[ChangeEntry],
    root_origin: Option<&str>,
) -> Vec<FlatRow> {
    match tab {
        ResultTab::Overview => build_issues_rows(pages),
        ResultTab::Changes => (0..change_entries.len())
            .map(|index| FlatRow::ChangeRow { index })
            .collect(),
        ResultTab::SiteStructure => build_directory_aggregates(pages, root_origin),
        ResultTab::Images => build_image_aggregates(page_indices, pages),
        _ => page_indices
            .iter()
            .flat_map(|&page_index| match pages.get(page_index) {
                Some(page) => page_item_rows(tab, page_index, page),
                None => Vec::new(),
            })
            .collect(),
    }
}

pub fn overview_issue_target(label: &str) -> Option<(ResultTab, IssueFilter)> {
    match label {
        "Missing Page Title" => Some((ResultTab::PageTitles, IssueFilter::Missing)),
        "Duplicate Page Title" => Some((ResultTab::PageTitles, IssueFilter::Duplicate)),
        "Page Title Over 60 Characters" => Some((ResultTab::PageTitles, IssueFilter::OverLength)),
        "Missing Meta Description" => Some((ResultTab::MetaDesc, IssueFilter::Missing)),
        "Duplicate Meta Description" => Some((ResultTab::MetaDesc, IssueFilter::Duplicate)),
        "Missing H1" => Some((ResultTab::H1, IssueFilter::Missing)),
        "Duplicate H1" => Some((ResultTab::H1, IssueFilter::Duplicate)),
        "Non-Indexable Pages" => Some((ResultTab::Internal, IssueFilter::NonIndexable)),
        "Missing Canonical Tag" => Some((ResultTab::Canonicals, IssueFilter::MissingCanonical)),
        "Missing HTTPS" => Some((ResultTab::Security, IssueFilter::MissingHttps)),
        "Images Missing Alt" => Some((ResultTab::Images, IssueFilter::MissingAltText)),
        "Images Missing Alt Text" => Some((ResultTab::Images, IssueFilter::MissingAltText)),
        "Structured Data Errors" => Some((ResultTab::StructuredData, IssueFilter::SdErrors)),
        "Structured Data Warnings" => Some((ResultTab::StructuredData, IssueFilter::SdWarnings)),
        "Slow LCP" => Some((ResultTab::Performance, IssueFilter::SlowLcp)),
        "Slow CLS" => Some((ResultTab::Performance, IssueFilter::SlowCls)),
        "Slow Largest Contentful Paint" => Some((ResultTab::Performance, IssueFilter::SlowLcp)),
        "High Cumulative Layout Shift" => Some((ResultTab::Performance, IssueFilter::SlowCls)),
        "A11y Critical Issues" => Some((ResultTab::Accessibility, IssueFilter::All)),
        "A11y Warnings" => Some((ResultTab::Accessibility, IssueFilter::All)),
        "HTTP Errors (4xx/5xx)" => Some((ResultTab::ResponseCodes, IssueFilter::Status4xx)),
        "Near Duplicate Content" => Some((ResultTab::Content, IssueFilter::NearDuplicates)),
        "Low Content Pages" => Some((ResultTab::Content, IssueFilter::LowContent)),
        "Content Requires JavaScript (SSR)" => {
            Some((ResultTab::Content, IssueFilter::SsrContentMissing))
        }
        "Blocked by robots.txt" => Some((ResultTab::Content, IssueFilter::BlockedByRobots)),
        "Redirects" => Some((ResultTab::ResponseCodes, IssueFilter::Redirects)),
        "Missing HSTS" => Some((ResultTab::Security, IssueFilter::MissingHsts)),
        "Missing CSP" => Some((ResultTab::Security, IssueFilter::MissingCsp)),
        "Missing Frame Guard" => Some((ResultTab::Security, IssueFilter::MissingFrameGuard)),
        "Missing X-Content-Type" => {
            Some((ResultTab::Security, IssueFilter::MissingContentTypeOptions))
        }
        "Mixed Content" => Some((ResultTab::Security, IssueFilter::MixedContent)),
        "Non-ASCII URLs" => Some((ResultTab::Url, IssueFilter::UrlNonAscii)),
        "Uppercase URLs" => Some((ResultTab::Url, IssueFilter::UrlUppercase)),
        "URLs with Underscores" => Some((ResultTab::Url, IssueFilter::UrlUnderscores)),
        "Long URLs" => Some((ResultTab::Url, IssueFilter::UrlOverLength)),
        "Hreflang Missing Return Tags" => {
            Some((ResultTab::Hreflang, IssueFilter::HreflangMissingReturnTag))
        }
        "Hreflang Invalid Language Codes" => {
            Some((ResultTab::Hreflang, IssueFilter::HreflangInvalidLang))
        }
        "Hreflang Missing x-default" => {
            Some((ResultTab::Hreflang, IssueFilter::HreflangMissingXDefault))
        }
        "Hreflang Non-Canonical Targets" => {
            Some((ResultTab::Hreflang, IssueFilter::HreflangNonCanonical))
        }
        "Page Title Below 30 Characters" => Some((ResultTab::PageTitles, IssueFilter::UnderLength)),
        "Page Title Below 200 Pixels" => {
            Some((ResultTab::PageTitles, IssueFilter::UnderPixelWidth))
        }
        "Meta Description Below 70 Characters" => {
            Some((ResultTab::MetaDesc, IssueFilter::UnderLength))
        }
        "Meta Description Below 400 Pixels" => {
            Some((ResultTab::MetaDesc, IssueFilter::UnderPixelWidth))
        }
        "Missing H2" => Some((ResultTab::H2, IssueFilter::Missing)),
        "Multiple H1" => Some((ResultTab::H1, IssueFilter::Multiple)),
        "Canonicalised" => Some((ResultTab::Canonicals, IssueFilter::Canonicalised)),
        "Images Missing Alt Attribute" => {
            Some((ResultTab::Images, IssueFilter::MissingAltAttribute))
        }
        "Images Missing Size Attributes" => {
            Some((ResultTab::Images, IssueFilter::MissingSizeAttributes))
        }
        "Images Over 100 kB" => Some((ResultTab::Images, IssueFilter::ImageOver100Kb)),
        "Non-200 URLs in Sitemap" => Some((ResultTab::Sitemaps, IssueFilter::SitemapNon200)),
        "Non-Indexable URLs in Sitemap" => {
            Some((ResultTab::Sitemaps, IssueFilter::NonIndexableInSitemap))
        }
        "Sitemap URLs Not Crawled" => Some((ResultTab::Sitemaps, IssueFilter::SitemapOrphans)),
        "Broken Images" => Some((ResultTab::Images, IssueFilter::ImageBroken)),
        "URLs with Parameters" => Some((ResultTab::Url, IssueFilter::UrlParameters)),
        "Missing X-Frame-Options" => Some((ResultTab::Security, IssueFilter::MissingFrameGuard)),
        "Missing X-Content-Type-Options" => {
            Some((ResultTab::Security, IssueFilter::MissingContentTypeOptions))
        }
        "Missing Content-Security-Policy" => Some((ResultTab::Security, IssueFilter::MissingCsp)),
        "Missing Referrer-Policy" => {
            Some((ResultTab::Security, IssueFilter::MissingReferrerPolicy))
        }
        "Multiple H2" => Some((ResultTab::H2, IssueFilter::Multiple)),
        "Hreflang Missing Self Reference" => Some((
            ResultTab::Hreflang,
            IssueFilter::HreflangMissingSelfReference,
        )),
        "Internal Outlinks With No Anchor Text" => {
            Some((ResultTab::Links, IssueFilter::LinkNoAnchorText))
        }
        _ => None,
    }
}

pub(super) fn build_issues_entries(pages: &[PageRecord]) -> Vec<IssueEntry> {
    let internal: Vec<&PageRecord> = pages.iter().filter(|p| p.is_internal).collect();
    // Page-content checks (titles, headings, canonicals, content quality, a11y,
    // performance, hreflang) apply only to pages eligible to rank: navigated
    // HTML documents that aren't redirect sources (whose body is the target's)
    // or non-indexable (noindex/error, intentionally out of the index). Counting
    // subresources, redirects, or noindex pages here would flag e.g. missing
    // titles on pages that legitimately have none. The matching drill-down
    // filters gate on the same `is_content_eligible`, so the counts and their
    // denominator reconcile on click-through.
    let documents: Vec<&PageRecord> = internal
        .iter()
        .copied()
        .filter(|p| super::filter::is_content_eligible(p))
        .collect();
    let total = internal.len().max(1) as f32;
    let doc_total = documents.len().max(1) as f32;
    let all_total = pages.len().max(1) as f32;
    let mut entries = Vec::new();

    let missing_title = documents
        .iter()
        .filter(|p| p.title.as_deref().unwrap_or("").is_empty())
        .count();
    if missing_title > 0 {
        entries.push(IssueEntry {
            name: "Missing Page Title".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: missing_title,
            pct: missing_title as f32 / doc_total * 100.0,
            description: "Pages with an empty or missing <title> tag.".into(),
            hint: "Add a unique, descriptive title (30-60 chars) to each page.".into(),
        });
    }

    let duplicate_title = {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for p in &documents {
            let val = p.title.as_deref().unwrap_or("");
            if val.is_empty() {
                continue;
            }
            *counts.entry(val).or_insert(0) += 1;
        }
        documents
            .iter()
            .filter(|p| *counts.get(p.title.as_deref().unwrap_or("")).unwrap_or(&0) > 1)
            .count()
    };
    if duplicate_title > 0 {
        entries.push(IssueEntry {
            name: "Duplicate Page Title".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: duplicate_title,
            pct: duplicate_title as f32 / doc_total * 100.0,
            description: "Multiple pages share the same title text.".into(),
            hint: "Give each page a unique title that reflects its content.".into(),
        });
    }

    let over_title = documents
        .iter()
        .filter(|p| {
            p.title
                .as_deref()
                .is_some_and(|t| char_length(t) > 60 && !t.is_empty())
        })
        .count();
    if over_title > 0 {
        entries.push(IssueEntry {
            name: "Page Title Over 60 Characters".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::Medium,
            count: over_title,
            pct: over_title as f32 / doc_total * 100.0,
            description: "Titles exceeding 60 characters may be truncated in search results."
                .into(),
            hint: "Keep titles between 30 and 60 characters.".into(),
        });
    }

    let missing_desc = documents
        .iter()
        .filter(|p| p.meta_description.as_deref().unwrap_or("").is_empty())
        .count();
    if missing_desc > 0 {
        entries.push(IssueEntry {
            name: "Missing Meta Description".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: missing_desc,
            pct: missing_desc as f32 / doc_total * 100.0,
            description: "Pages with an empty or missing meta description.".into(),
            hint: "Write a compelling meta description (50-160 chars) for each page.".into(),
        });
    }

    let duplicate_desc = {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for p in &documents {
            let val = p.meta_description.as_deref().unwrap_or("");
            if val.is_empty() {
                continue;
            }
            *counts.entry(val).or_insert(0) += 1;
        }
        documents
            .iter()
            .filter(|p| {
                *counts
                    .get(p.meta_description.as_deref().unwrap_or(""))
                    .unwrap_or(&0)
                    > 1
            })
            .count()
    };
    if duplicate_desc > 0 {
        entries.push(IssueEntry {
            name: "Duplicate Meta Description".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::Medium,
            count: duplicate_desc,
            pct: duplicate_desc as f32 / doc_total * 100.0,
            description: "Multiple pages share the same meta description.".into(),
            hint: "Write a unique meta description for each page.".into(),
        });
    }

    let missing_h1 = documents
        .iter()
        .filter(|p| p.h1.as_deref().unwrap_or("").is_empty())
        .count();
    if missing_h1 > 0 {
        entries.push(IssueEntry {
            name: "Missing H1".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: missing_h1,
            pct: missing_h1 as f32 / doc_total * 100.0,
            description: "Pages with an empty or missing H1 heading.".into(),
            hint: "Add a single H1 heading that describes the page topic.".into(),
        });
    }

    let duplicate_h1 = {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for p in &documents {
            let val = p.h1.as_deref().unwrap_or("");
            if val.is_empty() {
                continue;
            }
            *counts.entry(val).or_insert(0) += 1;
        }
        documents
            .iter()
            .filter(|p| *counts.get(p.h1.as_deref().unwrap_or("")).unwrap_or(&0) > 1)
            .count()
    };
    if duplicate_h1 > 0 {
        entries.push(IssueEntry {
            name: "Duplicate H1".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::Medium,
            count: duplicate_h1,
            pct: duplicate_h1 as f32 / doc_total * 100.0,
            description: "Multiple pages share the same H1 heading text.".into(),
            hint: "Make each H1 unique to the page's primary topic.".into(),
        });
    }

    let non_indexable = internal
        .iter()
        .filter(|p| p.indexability.as_deref() == Some("Non-Indexable"))
        .count();
    if non_indexable > 0 {
        entries.push(IssueEntry {
            name: "Non-Indexable Pages".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::High,
            count: non_indexable,
            pct: non_indexable as f32 / total * 100.0,
            description: "Pages excluded from the index by a noindex directive, a canonical pointing elsewhere, a redirect or an error status.".into(),
            hint: "Verify each non-indexable page is intentionally excluded. Remove noindex from pages that should rank.".into(),
        });
    }

    let missing_canonical = documents
        .iter()
        .filter(|p| p.canonical.as_deref() == Some(""))
        .count();
    if missing_canonical > 0 {
        entries.push(IssueEntry {
            name: "Missing Canonical Tag".into(),
            issue_type: IssueType::Opportunity,
            priority: IssuePriority::Medium,
            count: missing_canonical,
            pct: missing_canonical as f32 / doc_total * 100.0,
            description: "Pages without a self-referencing canonical link element.".into(),
            hint: "Add a canonical tag to every page to prevent duplicate content issues.".into(),
        });
    }

    let status_errors = pages
        .iter()
        .filter(|p| p.status.is_some_and(|c| c >= 400))
        .count();
    if status_errors > 0 {
        entries.push(IssueEntry {
            name: "HTTP Errors (4xx/5xx)".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: status_errors,
            pct: status_errors as f32 / all_total * 100.0,
            description: "Pages returning client or server error status codes.".into(),
            hint: "Fix broken links (404), resolve server errors, and redirect moved content."
                .into(),
        });
    }

    let redirects = pages.iter().filter(|p| p.is_redirect()).count();
    if redirects > 0 {
        entries.push(IssueEntry {
            name: "Redirects".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::Medium,
            count: redirects,
            pct: redirects as f32 / all_total * 100.0,
            description: "URLs that redirect to another location.".into(),
            hint: "Update internal links to point directly to the final URL.".into(),
        });
    }

    // Counted in pages, not image instances. Every other row in this table is a
    // count of pages over the page total, and a row that silently switched to
    // images over the image total made the percentage column impossible to read
    // across rows: 4 images out of 1865 showed as 0.2% next to page percentages.
    // The drill-down still lists the offending images, as it does for the
    // accessibility rules.
    let missing_alt = documents
        .iter()
        .filter(|p| {
            p.images
                .iter()
                .any(|img| !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty()))
        })
        .count();
    if missing_alt > 0 {
        entries.push(IssueEntry {
            name: "Images Missing Alt Text".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::Medium,
            count: missing_alt,
            pct: missing_alt as f32 / doc_total * 100.0,
            description: "Pages carrying an image without alt text or with an empty alt attribute."
                .into(),
            hint: "Add descriptive alt text to every meaningful image.".into(),
        });
    }

    let sd_errors = documents.iter().filter(|p| p.sd_errors > 0).count();
    if sd_errors > 0 {
        entries.push(IssueEntry {
            name: "Structured Data Errors".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: sd_errors,
            pct: sd_errors as f32 / doc_total * 100.0,
            description: "Pages with invalid structured data that may prevent rich results.".into(),
            hint: "Fix JSON-LD or microdata syntax errors. Test with Google's Rich Results Test."
                .into(),
        });
    }

    let near_dups = documents
        .iter()
        .filter(|p| p.near_duplicate_count.is_some_and(|c| c > 0))
        .count();
    if near_dups > 0 {
        entries.push(IssueEntry {
            name: "Near Duplicate Content".into(),
            issue_type: IssueType::Opportunity,
            priority: IssuePriority::Medium,
            count: near_dups,
            pct: near_dups as f32 / doc_total * 100.0,
            description: "Pages with highly similar content (90%+ match).".into(),
            hint: "Differentiate pages with unique content, merge thin variants, or use canonical tags.".into(),
        });
    }

    let low_content = documents
        .iter()
        .filter(|p| super::filter::is_low_content(p))
        .count();
    if low_content > 0 {
        entries.push(IssueEntry {
            name: "Low Content Pages".into(),
            issue_type: IssueType::Opportunity,
            priority: IssuePriority::Low,
            count: low_content,
            pct: low_content as f32 / doc_total * 100.0,
            description: format!(
                "Pages with fewer than {} words of body text.",
                super::filter::LOW_CONTENT_WORD_COUNT
            ),
            hint: "Add substantive content or consolidate thin pages.".into(),
        });
    }

    let ssr_content_missing = documents
        .iter()
        .filter(|p| p.ssr_content_missing == Some(true))
        .count();
    if ssr_content_missing > 0 {
        entries.push(IssueEntry {
            name: "Content Requires JavaScript (SSR)".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: ssr_content_missing,
            pct: ssr_content_missing as f32 / doc_total * 100.0,
            description: "The server-rendered HTML is missing content that only appears after \
                          client-side JavaScript runs."
                .into(),
            hint: "Ensure critical content (headings, copy) is present in the initial HTML so \
                   search engines relying on the server response can index it."
                .into(),
        });
    }

    let blocked_by_robots = documents
        .iter()
        .filter(|p| p.blocked_by_robots == Some(true))
        .count();
    if blocked_by_robots > 0 {
        entries.push(IssueEntry {
            name: "Blocked by robots.txt".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: blocked_by_robots,
            pct: blocked_by_robots as f32 / doc_total * 100.0,
            description: "These internal URLs are disallowed by the site's robots.txt file.".into(),
            hint: "Review robots.txt rules to ensure important pages are not accidentally blocked."
                .into(),
        });
    }

    let slow_lcp = documents
        .iter()
        .filter(|p| p.lcp_ms.is_some_and(|ms| ms > 4000))
        .count();
    if slow_lcp > 0 {
        entries.push(IssueEntry {
            name: "Slow Largest Contentful Paint".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: slow_lcp,
            pct: slow_lcp as f32 / doc_total * 100.0,
            description: "Pages with LCP over 4 seconds.".into(),
            hint: "Optimize images, eliminate render-blocking resources, improve server response time.".into(),
        });
    }

    let slow_cls = documents
        .iter()
        .filter(|p| p.cls.is_some_and(|v| v > 0.25))
        .count();
    if slow_cls > 0 {
        entries.push(IssueEntry {
            name: "High Cumulative Layout Shift".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::Medium,
            count: slow_cls,
            pct: slow_cls as f32 / doc_total * 100.0,
            description: "Pages with CLS above 0.25, causing visible layout shifts.".into(),
            hint: "Set explicit dimensions on images/videos, avoid inserting content above existing content.".into(),
        });
    }

    // Pages, not individual violations, for the same reason as the alt-text rule
    // above: a tally of violations over a tally of violations answers a
    // different question from every other percentage in this table, and could
    // read 100% on a site with one bad page.
    let a11y_critical = documents
        .iter()
        .filter(|p| {
            p.a11y_issues
                .iter()
                .any(|i| matches!(i.impact.as_str(), "critical" | "serious"))
        })
        .count();
    if a11y_critical > 0 {
        entries.push(IssueEntry {
            name: "Accessibility Critical Issues".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: a11y_critical,
            pct: a11y_critical as f32 / doc_total * 100.0,
            description: "Pages with critical or serious accessibility violations.".into(),
            hint: "Fix missing labels, ARIA roles, color contrast, and heading hierarchy.".into(),
        });
    }

    let missing_https = internal
        .iter()
        .filter(|p| !p.url.starts_with("https://"))
        .count();
    if missing_https > 0 {
        entries.push(IssueEntry {
            name: "Missing HTTPS".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: missing_https,
            pct: missing_https as f32 / total * 100.0,
            description: "Pages served over HTTP instead of HTTPS.".into(),
            hint: "Enable HTTPS across the entire site and redirect HTTP to HTTPS.".into(),
        });
    }

    let missing_hsts = internal
        .iter()
        .filter(|p| !header_exists(&p.headers, "strict-transport-security"))
        .count();
    if missing_hsts > 0 {
        entries.push(IssueEntry {
            name: "Missing HSTS Header".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::Low,
            count: missing_hsts,
            pct: missing_hsts as f32 / total * 100.0,
            description: "Pages missing the Strict-Transport-Security header.".into(),
            hint: "Add the Strict-Transport-Security header to enforce HTTPS.".into(),
        });
    }

    let mixed_content = internal.iter().filter(|p| p.has_mixed_content).count();
    if mixed_content > 0 {
        entries.push(IssueEntry {
            name: "Mixed Content".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: mixed_content,
            pct: mixed_content as f32 / total * 100.0,
            description: "HTTPS pages that load scripts, styles, images or other \
                          subresources over insecure HTTP."
                .into(),
            hint: "Serve every subresource over HTTPS. Browsers block or warn on \
                   mixed content, which can break the page and weaken security."
                .into(),
        });
    }

    let hreflang_missing_return = documents
        .iter()
        .filter(|p| {
            p.hreflang_issues
                .iter()
                .any(|i| matches!(i, HreflangIssue::MissingReturnTag { .. }))
        })
        .count();
    if hreflang_missing_return > 0 {
        entries.push(IssueEntry {
            name: "Hreflang Missing Return Tags".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: hreflang_missing_return,
            pct: hreflang_missing_return as f32 / doc_total * 100.0,
            description: "Pages with hreflang tags that are not reciprocated by the target URL.".into(),
            hint: "Ensure every hreflang link is bidirectional: if A links to B, B must link back to A.".into(),
        });
    }

    let hreflang_invalid_lang = documents
        .iter()
        .filter(|p| {
            p.hreflang_issues
                .iter()
                .any(|i| matches!(i, HreflangIssue::InvalidLanguageCode { .. }))
        })
        .count();
    if hreflang_invalid_lang > 0 {
        entries.push(IssueEntry {
            name: "Hreflang Invalid Language Codes".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::Medium,
            count: hreflang_invalid_lang,
            pct: hreflang_invalid_lang as f32 / doc_total * 100.0,
            description: "Pages using hreflang codes that don't follow the BCP-47 standard.".into(),
            hint: "Use valid ISO 639-1 language codes (e.g. 'en', 'de') and optional region subtags (e.g. 'en-US').".into(),
        });
    }

    let hreflang_missing_xdefault = documents
        .iter()
        .filter(|p| {
            p.hreflang_issues
                .iter()
                .any(|i| matches!(i, HreflangIssue::MissingXDefault))
        })
        .count();
    if hreflang_missing_xdefault > 0 {
        entries.push(IssueEntry {
            name: "Hreflang Missing x-default".into(),
            issue_type: IssueType::Warning,
            priority: IssuePriority::Medium,
            count: hreflang_missing_xdefault,
            pct: hreflang_missing_xdefault as f32 / doc_total * 100.0,
            description: "Pages with hreflang but no x-default fallback tag.".into(),
            hint: "Add an hreflang x-default tag pointing to the default page for unmatched languages.".into(),
        });
    }

    let hreflang_noncanonical = documents
        .iter()
        .filter(|p| {
            p.hreflang_issues
                .iter()
                .any(|i| matches!(i, HreflangIssue::NonCanonicalUrl { .. }))
        })
        .count();
    if hreflang_noncanonical > 0 {
        entries.push(IssueEntry {
            name: "Hreflang Non-Canonical Targets".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: hreflang_noncanonical,
            pct: hreflang_noncanonical as f32 / doc_total * 100.0,
            description:
                "Hreflang URLs pointing to pages whose canonical differs from the hreflang target."
                    .into(),
            hint: "Ensure hreflang URLs match the canonical URL of the target page.".into(),
        });
    }

    // Counted in pages, like the image rules: the drill-down lists the
    // individual links. Only internal links count, because we can't fix anchor
    // text on someone else's site, and an image link with no alt text lands
    // here too, which is the common cause.
    let no_anchor_text = documents
        .iter()
        .filter(|p| {
            p.outlinks.iter().any(|link| {
                is_same_domain(&p.url, &link.dst_url) && super::filter::link_lacks_anchor_text(link)
            })
        })
        .count();
    if no_anchor_text > 0 {
        entries.push(IssueEntry {
            name: "Internal Outlinks With No Anchor Text".into(),
            issue_type: IssueType::Opportunity,
            priority: IssuePriority::Low,
            count: no_anchor_text,
            pct: no_anchor_text as f32 / doc_total * 100.0,
            description: "Pages linking internally with no anchor text at all.".into(),
            hint: "Anchor text tells search engines what the target page is about. Give \
                   every link words, or an image link alt text."
                .into(),
        });
    }

    // Rules whose predicate already exists as a tab sub-filter. Counting them
    // through the filter itself means the headline figure and the rows you land
    // on after clicking through are produced by the same code, so they cannot
    // drift the way two hand-written copies of a predicate do.
    // Several rules share a tab, and the occurrence map is the expensive part of
    // setting one up (a string key per page), so build one per tab, not per rule.
    let mut occurrences_by_tab: HashMap<ResultTab, HashMap<String, usize>> = HashMap::new();
    for rule in FILTER_DERIVED_RULES {
        let denominator = match rule.denominator {
            Denominator::Documents => doc_total,
            Denominator::InternalUrls => total,
        };
        let occurrences = occurrences_by_tab
            .entry(rule.tab)
            .or_insert_with(|| super::columns::build_occurrence_counts(rule.tab, pages));
        if let Some(entry) = filter_derived_entry(pages, rule, denominator, occurrences) {
            entries.push(entry);
        }
    }

    entries.sort_by(|a, b| match a.issue_type.cmp(&b.issue_type) {
        std::cmp::Ordering::Equal => match a.priority.cmp(&b.priority) {
            std::cmp::Ordering::Equal => b.count.cmp(&a.count),
            other => other,
        },
        other => other,
    });

    entries
}

/// Which population a filter-derived rule's percentage is a share of.
#[derive(Clone, Copy)]
enum Denominator {
    /// Pages eligible for on-page content issues, the denominator every
    /// document-derived rule above uses.
    Documents,
    /// Every internal URL, including subresources, for rules that apply to
    /// assets as much as to pages (URL shape, response headers).
    InternalUrls,
}

struct FilterDerivedRule {
    name: &'static str,
    tab: ResultTab,
    filter: IssueFilter,
    issue_type: IssueType,
    priority: IssuePriority,
    denominator: Denominator,
    description: &'static str,
    hint: &'static str,
}

/// Overview rules that are a straight promotion of an existing tab sub-filter.
/// Adding one here also needs a line in `overview_issue_target` so the row stays
/// clickable; `every_filter_derived_rule_has_a_click_through_target` enforces it.
static FILTER_DERIVED_RULES: &[FilterDerivedRule] = &[
    FilterDerivedRule {
        name: "Page Title Below 30 Characters",
        tab: ResultTab::PageTitles,
        filter: IssueFilter::UnderLength,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Titles short enough to leave SERP space unused.",
        hint: "Work the page's primary keyword and a qualifier into the title.",
    },
    FilterDerivedRule {
        name: "Page Title Below 200 Pixels",
        tab: ResultTab::PageTitles,
        filter: IssueFilter::UnderPixelWidth,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Titles that render narrower than 200 pixels in search results.",
        hint: "Lengthen the title; Google renders roughly 580 pixels of it.",
    },
    FilterDerivedRule {
        name: "Meta Description Below 70 Characters",
        tab: ResultTab::MetaDesc,
        filter: IssueFilter::UnderLength,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Descriptions short enough to leave SERP space unused.",
        hint: "Describe the page in 70 to 155 characters, with a reason to click.",
    },
    FilterDerivedRule {
        name: "Meta Description Below 400 Pixels",
        tab: ResultTab::MetaDesc,
        filter: IssueFilter::UnderPixelWidth,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Descriptions that render narrower than 400 pixels in search results.",
        hint: "Lengthen the description; Google renders roughly 970 pixels of it.",
    },
    FilterDerivedRule {
        name: "Missing H2",
        tab: ResultTab::H2,
        filter: IssueFilter::Missing,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Pages with no H2 subheading.",
        hint: "Break long pages into sections with descriptive H2 headings.",
    },
    FilterDerivedRule {
        name: "Multiple H1",
        tab: ResultTab::H1,
        filter: IssueFilter::Multiple,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Pages with more than one H1 heading.",
        hint: "Keep one H1 per page for the primary topic and demote the rest to H2.",
    },
    FilterDerivedRule {
        name: "Canonicalised",
        tab: ResultTab::Canonicals,
        filter: IssueFilter::Canonicalised,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "Pages whose canonical points at a different URL, keeping them out of \
                      the index.",
        hint: "Confirm each is deliberate. A canonical pointing at the wrong URL silently \
               removes the page from search.",
    },
    FilterDerivedRule {
        name: "Non-200 URLs in Sitemap",
        tab: ResultTab::Sitemaps,
        filter: IssueFilter::SitemapNon200,
        issue_type: IssueType::Issue,
        priority: IssuePriority::High,
        denominator: Denominator::Documents,
        description: "URLs a sitemap advertises that answered with a redirect or an error.",
        hint: "A sitemap should list the final, indexable URL. Update the entry or drop it.",
    },
    FilterDerivedRule {
        name: "Non-Indexable URLs in Sitemap",
        tab: ResultTab::Sitemaps,
        filter: IssueFilter::NonIndexableInSitemap,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "URLs a sitemap advertises that are noindex, canonicalised or redirected.",
        hint: "A sitemap is a list of URLs you want indexed. Remove the ones you don't.",
    },
    FilterDerivedRule {
        name: "Sitemap URLs Not Crawled",
        tab: ResultTab::Sitemaps,
        filter: IssueFilter::SitemapOrphans,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "URLs listed in a sitemap that no page on the site links to.",
        hint: "Either link to them so they can be found, or drop them from the sitemap.",
    },
    FilterDerivedRule {
        name: "Images Over 100 kB",
        tab: ResultTab::Images,
        filter: IssueFilter::ImageOver100Kb,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "Pages carrying an image over 100 kB, measured when it was fetched.",
        hint: "Compress and resize the image, and serve a modern format such as WebP or AVIF.",
    },
    FilterDerivedRule {
        name: "Broken Images",
        tab: ResultTab::Images,
        filter: IssueFilter::ImageBroken,
        issue_type: IssueType::Issue,
        priority: IssuePriority::High,
        denominator: Denominator::Documents,
        description: "Pages referencing an image that did not load when it was fetched.",
        hint: "Fix the src or remove the image. A broken image is a wasted request and a \
               visible hole in the page.",
    },
    FilterDerivedRule {
        name: "Images Missing Alt Attribute",
        tab: ResultTab::Images,
        filter: IssueFilter::MissingAltAttribute,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "Pages carrying an image with no alt attribute at all.",
        hint: "Add alt to every image; use alt=\"\" for purely decorative ones.",
    },
    FilterDerivedRule {
        name: "Images Missing Size Attributes",
        tab: ResultTab::Images,
        filter: IssueFilter::MissingSizeAttributes,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Pages carrying an image without width and height attributes.",
        hint: "Set width and height so the browser reserves space, which avoids layout shift.",
    },
    FilterDerivedRule {
        name: "URLs with Parameters",
        tab: ResultTab::Url,
        filter: IssueFilter::UrlParameters,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs carrying a query string.",
        hint: "Parameterised URLs multiply into near-duplicates. Canonicalise them to the \
               clean URL.",
    },
    FilterDerivedRule {
        name: "Uppercase URLs",
        tab: ResultTab::Url,
        filter: IssueFilter::UrlUppercase,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs containing uppercase characters.",
        hint: "Servers usually treat /Page and /page as different URLs. Standardise on \
               lowercase and redirect the rest.",
    },
    FilterDerivedRule {
        name: "Missing X-Frame-Options",
        tab: ResultTab::Security,
        filter: IssueFilter::MissingFrameGuard,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs missing both X-Frame-Options and a CSP frame-ancestors directive.",
        hint: "Send X-Frame-Options or frame-ancestors to prevent clickjacking.",
    },
    FilterDerivedRule {
        name: "Missing X-Content-Type-Options",
        tab: ResultTab::Security,
        filter: IssueFilter::MissingContentTypeOptions,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs missing the X-Content-Type-Options header.",
        hint: "Send X-Content-Type-Options: nosniff so browsers honour the declared type.",
    },
    FilterDerivedRule {
        name: "Missing Referrer-Policy",
        tab: ResultTab::Security,
        filter: IssueFilter::MissingReferrerPolicy,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs missing a Referrer-Policy header, or sending one that still leaks \
                      the full URL to other origins.",
        hint: "Send Referrer-Policy: strict-origin-when-cross-origin so paths and query \
               strings stay on your own site.",
    },
    FilterDerivedRule {
        name: "Multiple H2",
        tab: ResultTab::H2,
        filter: IssueFilter::Multiple,
        issue_type: IssueType::Opportunity,
        priority: IssuePriority::Low,
        denominator: Denominator::Documents,
        description: "Pages with more than one H2 subheading.",
        hint: "Several H2s are fine on a long page. Check they describe distinct sections \
               rather than repeating the H1.",
    },
    FilterDerivedRule {
        name: "Hreflang Missing Self Reference",
        tab: ResultTab::Hreflang,
        filter: IssueFilter::HreflangMissingSelfReference,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Medium,
        denominator: Denominator::Documents,
        description: "Pages whose hreflang set doesn't list the page itself.",
        hint: "Every page in a cluster must reference itself, or search engines may \
               discard the whole set.",
    },
    FilterDerivedRule {
        name: "Missing Content-Security-Policy",
        tab: ResultTab::Security,
        filter: IssueFilter::MissingCsp,
        issue_type: IssueType::Warning,
        priority: IssuePriority::Low,
        denominator: Denominator::InternalUrls,
        description: "URLs missing a Content-Security-Policy header.",
        hint: "Add a policy restricting where scripts, styles and frames may load from.",
    },
];

fn filter_derived_entry(
    pages: &[PageRecord],
    rule: &FilterDerivedRule,
    denominator: f32,
    occurrence_counts: &HashMap<String, usize>,
) -> Option<IssueEntry> {
    let count =
        super::filter::filter_for_tab(rule.tab, rule.filter, pages, occurrence_counts).len();
    if count == 0 {
        return None;
    }
    Some(IssueEntry {
        name: rule.name.into(),
        issue_type: rule.issue_type,
        priority: rule.priority,
        count,
        pct: count as f32 / denominator * 100.0,
        description: rule.description.into(),
        hint: rule.hint.into(),
    })
}

/// Compares the loaded crawl's internal pages against a baseline crawl, keyed by
/// URL, producing one entry per page that was added, removed, or changed.
pub(super) fn build_change_entries(
    current: &[PageRecord],
    baseline: &[PageRecord],
) -> Vec<ChangeEntry> {
    let current_by_url: HashMap<&str, &PageRecord> = current
        .iter()
        .filter(|p| p.is_internal)
        .map(|p| (p.url.as_str(), p))
        .collect();
    let baseline_by_url: HashMap<&str, &PageRecord> = baseline
        .iter()
        .filter(|p| p.is_internal)
        .map(|p| (p.url.as_str(), p))
        .collect();

    let mut entries = Vec::new();

    for (url, page) in &current_by_url {
        match baseline_by_url.get(url) {
            None => entries.push(ChangeEntry {
                url: (*url).to_string(),
                kind: ChangeKind::Added,
                status_before: None,
                status_after: page.status,
                changes: Vec::new(),
            }),
            Some(previous) => {
                let changes = describe_changes(previous, page);
                if !changes.is_empty() {
                    entries.push(ChangeEntry {
                        url: (*url).to_string(),
                        kind: ChangeKind::Changed,
                        status_before: previous.status,
                        status_after: page.status,
                        changes,
                    });
                }
            }
        }
    }

    for (url, previous) in &baseline_by_url {
        if !current_by_url.contains_key(url) {
            entries.push(ChangeEntry {
                url: (*url).to_string(),
                kind: ChangeKind::Removed,
                status_before: previous.status,
                status_after: None,
                changes: Vec::new(),
            });
        }
    }

    entries.sort_by(|a, b| a.kind.cmp(&b.kind).then_with(|| a.url.cmp(&b.url)));
    entries
}

fn describe_changes(before: &PageRecord, after: &PageRecord) -> Vec<String> {
    fn opt_num(value: Option<u32>) -> String {
        value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
    }
    fn opt_status(value: Option<u16>) -> String {
        value.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
    }

    let mut out = Vec::new();
    if before.status != after.status {
        out.push(format!(
            "status {} → {}",
            opt_status(before.status),
            opt_status(after.status)
        ));
    }
    if before.indexability != after.indexability {
        out.push(format!(
            "{} → {}",
            before.indexability.as_deref().unwrap_or("-"),
            after.indexability.as_deref().unwrap_or("-")
        ));
    }
    if before.title != after.title {
        out.push("title changed".into());
    }
    if before.h1 != after.h1 {
        out.push("h1 changed".into());
    }
    if before.meta_description != after.meta_description {
        out.push("meta description changed".into());
    }
    if before.word_count != after.word_count {
        out.push(format!(
            "words {} → {}",
            opt_num(before.word_count),
            opt_num(after.word_count)
        ));
    }
    out
}

pub(super) fn build_issues_rows(pages: &[PageRecord]) -> Vec<FlatRow> {
    build_issues_entries(pages)
        .into_iter()
        .enumerate()
        .map(|(index, _)| FlatRow::IssuesRow { index })
        .collect()
}

pub(super) fn directory_path(url_path: &str) -> String {
    let path = url_path.trim_end_matches('/');
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    let last_segment = path.rsplit('/').next().unwrap_or("");
    if last_segment.contains('.') {
        let parent = &path[..path.len() - last_segment.len()];
        if parent.is_empty() || parent == "/" {
            return "/".to_string();
        }
        format!("{}/", parent.trim_end_matches('/'))
    } else {
        format!("{}/", path)
    }
}

pub(super) fn dir_format_size(bytes: u64) -> String {
    if bytes == 0 {
        return "-".into();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

/// True when an image source is an inline payload rather than a fetchable
/// resource. The Images tab leaves these out: they cannot be requested, sized
/// or status-checked, and on the reference crawl 872 of 1865 image rows were
/// inline SVG flag icons, which is most of the tab.
pub(super) fn is_inline_image(src: &str) -> bool {
    src.trim_start().to_ascii_lowercase().starts_with("data:")
}

/// One row per unique image source across the given pages, carrying how many
/// `img` tags point at it and which pages those are.
pub(super) fn build_image_aggregates(page_indices: &[usize], pages: &[PageRecord]) -> Vec<FlatRow> {
    // What the post-crawl resource pass found for each image URL. The images
    // themselves are recorded on the pages that reference them and carry no
    // status or size of their own.
    let checked: HashMap<&str, (Option<u16>, u64)> = pages
        .iter()
        .filter(|page| page.is_resource)
        .map(|page| (page.url.as_str(), (page.status, page.size_bytes)))
        .collect();
    let mut by_src: HashMap<&str, ImageAggregateRow> = HashMap::new();
    // Insertion order, so the tab lists images in the order the crawl met them
    // rather than in hash order, which would reshuffle between runs.
    let mut order: Vec<&str> = Vec::new();

    for &page_index in page_indices {
        let Some(page) = pages.get(page_index) else {
            continue;
        };
        for image in &page.images {
            if is_inline_image(&image.src) {
                continue;
            }
            let entry = by_src.entry(image.src.as_str()).or_insert_with(|| {
                order.push(image.src.as_str());
                let (status, size_bytes) = checked
                    .get(image.src.as_str())
                    .copied()
                    .unwrap_or((None, 0));
                ImageAggregateRow {
                    src: image.src.clone(),
                    status,
                    size_bytes,
                    alt: None,
                    width: None,
                    height: None,
                    missing_alt_attr: false,
                    missing_alt_text: false,
                    alt_over_100: false,
                    missing_size_attrs: false,
                    reference_count: 0,
                    pages: Vec::new(),
                }
            });

            entry.reference_count += 1;
            if entry.pages.last() != Some(&page_index) {
                entry.pages.push(page_index);
            }
            if entry.alt.is_none() {
                entry.alt = image.alt.clone().filter(|alt| !alt.is_empty());
            }
            entry.width = entry.width.or(image.width);
            entry.height = entry.height.or(image.height);
            entry.missing_alt_attr |= !image.has_alt_attr;
            entry.missing_alt_text |=
                image.has_alt_attr && image.alt.as_deref().is_none_or(|alt| alt.is_empty());
            entry.alt_over_100 |= image
                .alt
                .as_deref()
                .is_some_and(|alt| char_length(alt) > 100);
            entry.missing_size_attrs |= image.width.is_none() || image.height.is_none();
        }
    }

    order
        .into_iter()
        .filter_map(|src| by_src.remove(src))
        .map(|row| FlatRow::ImageAggregate(Box::new(row)))
        .collect()
}

pub(super) fn build_directory_aggregates(
    pages: &[PageRecord],
    root_origin: Option<&str>,
) -> Vec<FlatRow> {
    let Some(origin) = root_origin else {
        return Vec::new();
    };

    let mut dir_data: HashMap<String, DirAccumulator> = HashMap::new();

    for page in pages
        .iter()
        .filter(|p| p.is_internal && p.is_page && !p.is_resource)
    {
        let path = page.url.strip_prefix(origin).unwrap_or(&page.url);
        let dir_path = directory_path(path);

        let acc = dir_data.entry(dir_path).or_default();
        acc.page_count += 1;
        acc.total_word_count += page.word_count.unwrap_or(0) as u64;
        acc.total_size += page.size_bytes;
        if page.indexability.as_deref() == Some("Non-Indexable") {
            acc.non_indexable += 1;
        } else {
            acc.indexable += 1;
        }
    }

    let mut rows: Vec<FlatRow> = dir_data
        .into_iter()
        .map(|(path, acc)| FlatRow::DirectoryAggregate {
            depth: path.matches('/').count().saturating_sub(1) as u32,
            avg_word_count: if acc.page_count > 0 {
                acc.total_word_count / acc.page_count as u64
            } else {
                0
            },
            total_size: acc.total_size,
            page_count: acc.page_count,
            non_indexable: acc.non_indexable,
            indexable: acc.indexable,
            path,
        })
        .collect();

    rows.sort_by(|a, b| {
        let a_path = match a {
            FlatRow::DirectoryAggregate { path, .. } => path,
            _ => "",
        };
        let b_path = match b {
            FlatRow::DirectoryAggregate { path, .. } => path,
            _ => "",
        };
        a_path.cmp(b_path)
    });

    rows
}

#[derive(Default)]
struct DirAccumulator {
    page_count: usize,
    total_word_count: u64,
    total_size: u64,
    non_indexable: usize,
    indexable: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crawl::event::PageRecord;

    fn page(url: &str, h1: Option<&str>) -> PageRecord {
        PageRecord {
            url: url.into(),
            is_internal: true,
            is_page: true,
            h1: h1.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn count_for(entries: &[IssueEntry], name: &str) -> usize {
        entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.count)
            .unwrap_or(0)
    }

    #[test]
    fn missing_h1_is_not_counted_as_duplicate() {
        let pages = vec![
            page("https://a.test/1", None),
            page("https://a.test/2", Some("")),
            page("https://a.test/3", None),
        ];
        let entries = build_issues_entries(&pages);
        assert_eq!(count_for(&entries, "Missing H1"), 3);
        assert_eq!(count_for(&entries, "Duplicate H1"), 0);
    }

    #[test]
    fn shared_non_empty_h1_is_counted_as_duplicate() {
        let pages = vec![
            page("https://a.test/1", Some("Same Heading")),
            page("https://a.test/2", Some("Same Heading")),
            page("https://a.test/3", Some("Unique Heading")),
        ];
        let entries = build_issues_entries(&pages);
        assert_eq!(count_for(&entries, "Duplicate H1"), 2);
        assert_eq!(count_for(&entries, "Missing H1"), 0);
    }

    fn change_for<'a>(entries: &'a [ChangeEntry], url: &str) -> Option<&'a ChangeEntry> {
        entries.iter().find(|e| e.url == url)
    }

    #[test]
    fn change_entries_classify_added_removed_changed() {
        let baseline = vec![
            page("https://a.test/keep", Some("Heading")),
            page("https://a.test/gone", Some("Heading")),
        ];
        let mut changed = page("https://a.test/keep", Some("New Heading"));
        changed.status = Some(200);
        let current = vec![changed, page("https://a.test/new", Some("Heading"))];

        let entries = build_change_entries(&current, &baseline);

        assert_eq!(
            change_for(&entries, "https://a.test/new").unwrap().kind,
            ChangeKind::Added
        );
        assert_eq!(
            change_for(&entries, "https://a.test/gone").unwrap().kind,
            ChangeKind::Removed
        );
        let keep = change_for(&entries, "https://a.test/keep").unwrap();
        assert_eq!(keep.kind, ChangeKind::Changed);
        assert!(keep.changes.iter().any(|c| c.contains("h1")));
    }

    #[test]
    fn change_entries_skip_unchanged_pages() {
        let baseline = vec![page("https://a.test/keep", Some("Heading"))];
        let current = vec![page("https://a.test/keep", Some("Heading"))];
        assert!(build_change_entries(&current, &baseline).is_empty());
    }
}

#[cfg(test)]
mod overview_denominator_tests {
    use super::*;
    use crate::crawl::event::{A11yIssue, ImageRef};

    fn document(url: &str) -> PageRecord {
        PageRecord {
            url: url.into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            title: Some(format!("Title for {url}")),
            meta_description: Some(format!("Meta description for {url}, long enough to pass.")),
            h1: Some(format!("H1 for {url}")),
            word_count: Some(500),
            ..Default::default()
        }
    }

    fn image_without_alt(src: &str) -> ImageRef {
        ImageRef {
            src: src.into(),
            alt: None,
            width: Some(10),
            height: Some(10),
            has_alt_attr: false,
        }
    }

    fn critical_issue(rule: &str) -> A11yIssue {
        A11yIssue {
            rule: rule.into(),
            impact: "critical".into(),
            target: None,
            html: None,
        }
    }

    fn entry<'a>(entries: &'a [IssueEntry], name: &str) -> &'a IssueEntry {
        entries
            .iter()
            .find(|e| e.name == name)
            .unwrap_or_else(|| panic!("no {name} entry in {:?}", entries.iter().map(|e| &e.name)))
    }

    /// One page carrying many offending images or violations must not report a
    /// count or a percentage in those units: the whole table is pages over the
    /// page total, and instance tallies made the percentage column unreadable
    /// across rows.
    #[test]
    fn instance_derived_rules_are_counted_in_pages() {
        let mut offender = document("https://a.test/gallery");
        offender.images = vec![
            image_without_alt("/one.png"),
            image_without_alt("/two.png"),
            image_without_alt("/three.png"),
        ];
        offender.a11y_issues = vec![critical_issue("image-alt"), critical_issue("label")];
        let pages = vec![offender, document("https://a.test/clean")];

        let entries = build_issues_entries(&pages);
        let alt = entry(&entries, "Images Missing Alt Text");
        assert_eq!(alt.count, 1, "one page, not three images");
        assert!((alt.pct - 50.0).abs() < 0.01, "pct was {}", alt.pct);

        let a11y = entry(&entries, "Accessibility Critical Issues");
        assert_eq!(a11y.count, 1, "one page, not two violations");
        assert!((a11y.pct - 50.0).abs() < 0.01, "pct was {}", a11y.pct);
    }

    #[test]
    fn no_overview_percentage_can_exceed_one_hundred() {
        let mut offender = document("https://a.test/gallery");
        offender.images = (0..40)
            .map(|index| image_without_alt(&format!("/{index}.png")))
            .collect();
        offender.a11y_issues = (0..40).map(|_| critical_issue("image-alt")).collect();
        let pages = vec![offender];

        for entry in build_issues_entries(&pages) {
            assert!(entry.pct <= 100.0, "{} reported {}%", entry.name, entry.pct);
        }
    }
}

#[cfg(test)]
mod filter_derived_rule_tests {
    use super::*;

    #[test]
    fn every_filter_derived_rule_has_a_click_through_target() {
        for rule in FILTER_DERIVED_RULES {
            let target = overview_issue_target(rule.name);
            assert_eq!(
                target,
                Some((rule.tab, rule.filter)),
                "{} lands on {target:?} instead of its own filter",
                rule.name
            );
        }
    }

    #[test]
    fn rule_names_are_unique() {
        let mut names: Vec<&str> = FILTER_DERIVED_RULES.iter().map(|r| r.name).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "duplicate rule name");
    }

    #[test]
    fn a_rule_with_no_matches_is_not_listed() {
        let clean = PageRecord {
            url: "https://a.test/".into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            title: Some("A perfectly ordinary page title of adequate length".into()),
            meta_description: Some(
                "A meta description with enough characters in it to clear the lower bound \
                 comfortably."
                    .into(),
            ),
            h1: Some("Heading".into()),
            h2: Some("Subheading".into()),
            h1_count: 1,
            word_count: Some(500),
            ..Default::default()
        };
        let entries = build_issues_entries(&[clean]);
        assert!(
            !entries.iter().any(|e| e.name == "Missing H2"),
            "a zero count must not produce a row"
        );
    }
}
