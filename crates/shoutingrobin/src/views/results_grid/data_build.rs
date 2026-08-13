use std::collections::HashMap;

use crate::crawl::engine::is_same_domain;
use crate::crawl::event::{HreflangIssue, PageRecord};
use crate::views::ResultTab;

use super::columns::header_exists;
use super::types::{
    ChangeEntry, ChangeKind, FlatRow, IssueEntry, IssueFilter, IssuePriority, IssueType,
};

pub(super) fn flat_row_item_count(page: &PageRecord, tab: ResultTab) -> usize {
    match tab {
        ResultTab::Images => page.images.len(),
        ResultTab::Accessibility => page.a11y_issues.len(),
        ResultTab::Hreflang => page.hreflang_tags.len().max(1),
        ResultTab::StructuredData => page.sd_items.len().max(1),
        _ => 0,
    }
}

pub(super) fn flat_row_variant(tab: ResultTab, page: usize, item: usize) -> FlatRow {
    match tab {
        ResultTab::Images => FlatRow::Image { page, item },
        ResultTab::Accessibility => FlatRow::A11yIssue { page, item },
        ResultTab::Hreflang => FlatRow::Hreflang { page, item },
        ResultTab::StructuredData => FlatRow::SdItem { page, item },
        _ => FlatRow::Image { page, item },
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
        // The Links tab only lists internal outlinks; mirror that here so counts
        // match what the grid shows.
        ResultTab::Links => page_indices
            .iter()
            .flat_map(|&page_index| {
                let Some(page) = pages.get(page_index) else {
                    return Vec::<FlatRow>::new();
                };
                page.outlinks
                    .iter()
                    .enumerate()
                    .filter(|(_, link)| is_same_domain(&page.url, &link.dst_url))
                    .map(|(item_index, _)| FlatRow::LinkRow {
                        page: page_index,
                        item: item_index,
                    })
                    .collect()
            })
            .collect(),
        _ => page_indices
            .iter()
            .flat_map(|&page_index| {
                let item_count = pages
                    .get(page_index)
                    .map(|page| flat_row_item_count(page, tab))
                    .unwrap_or(0);
                (0..item_count)
                    .map(move |item_index| flat_row_variant(tab, page_index, item_index))
                    .collect::<Vec<_>>()
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
                .is_some_and(|t| t.len() > 60 && !t.is_empty())
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

    let redirects = pages.iter().filter(|p| p.redirect_url.is_some()).count();
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

    let missing_alt: usize = documents
        .iter()
        .flat_map(|p| p.images.iter())
        .filter(|img| !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty()))
        .count();
    if missing_alt > 0 {
        entries.push(IssueEntry {
            name: "Images Missing Alt Text".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::Medium,
            count: missing_alt,
            pct: missing_alt as f32
                / documents
                    .iter()
                    .map(|p| p.images.len())
                    .sum::<usize>()
                    .max(1) as f32
                * 100.0,
            description: "Images without alt text or with an empty alt attribute.".into(),
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
        .filter(|p| p.word_count.is_some_and(|w| w > 0 && w < 100))
        .count();
    if low_content > 0 {
        entries.push(IssueEntry {
            name: "Low Content Pages".into(),
            issue_type: IssueType::Opportunity,
            priority: IssuePriority::Low,
            count: low_content,
            pct: low_content as f32 / doc_total * 100.0,
            description: "Pages with fewer than 100 words of body text.".into(),
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

    let a11y_critical = documents
        .iter()
        .flat_map(|p| p.a11y_issues.iter())
        .filter(|i| matches!(i.impact.as_str(), "critical" | "serious"))
        .count();
    if a11y_critical > 0 {
        // count is a tally of individual violations, not pages, so the
        // percentage must be relative to the total number of a11y issues
        // (as with "Images Missing Alt Text"), otherwise it can exceed 100%.
        let a11y_total = documents
            .iter()
            .map(|p| p.a11y_issues.len())
            .sum::<usize>()
            .max(1) as f32;
        entries.push(IssueEntry {
            name: "Accessibility Critical Issues".into(),
            issue_type: IssueType::Issue,
            priority: IssuePriority::High,
            count: a11y_critical,
            pct: a11y_critical as f32 / a11y_total * 100.0,
            description: "Critical or serious accessibility violations.".into(),
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

    entries.sort_by(|a, b| match a.issue_type.cmp(&b.issue_type) {
        std::cmp::Ordering::Equal => match a.priority.cmp(&b.priority) {
            std::cmp::Ordering::Equal => b.count.cmp(&a.count),
            other => other,
        },
        other => other,
    });

    entries
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
