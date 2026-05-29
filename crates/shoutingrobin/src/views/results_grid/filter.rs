use std::collections::HashMap;

use crate::crawl::engine::is_same_domain;
use crate::crawl::event::{A11yIssue, HreflangIssue, ImageRef, PageRecord, SdFormat, SdItem};
use crate::views::ResultTab;

use super::columns::{
    field_count, field_value, header_exists, header_value, length_thresholds,
    pixel_width_thresholds, primary_field_key,
};
use super::types::IssueFilter;

pub fn filters_for_tab(tab: ResultTab) -> &'static [IssueFilter] {
    match tab {
        ResultTab::Internal => &[
            IssueFilter::All,
            IssueFilter::Html,
            IssueFilter::Css,
            IssueFilter::JavaScript,
            IssueFilter::Images,
            IssueFilter::Pdf,
            IssueFilter::OtherResource,
            IssueFilter::NonIndexable,
        ],
        ResultTab::External => &[IssueFilter::All],
        ResultTab::ResponseCodes => &[
            IssueFilter::All,
            IssueFilter::Status2xx,
            IssueFilter::Status3xx,
            IssueFilter::Status4xx,
            IssueFilter::Status5xx,
            IssueFilter::Redirects,
            IssueFilter::RedirectLoop,
        ],
        ResultTab::PageTitles => &[
            IssueFilter::All,
            IssueFilter::Missing,
            IssueFilter::Duplicate,
            IssueFilter::OverLength,
            IssueFilter::UnderLength,
            IssueFilter::OverPixelWidth,
            IssueFilter::UnderPixelWidth,
            IssueFilter::Multiple,
            IssueFilter::SameAsH1,
        ],
        ResultTab::MetaDesc => &[
            IssueFilter::All,
            IssueFilter::Missing,
            IssueFilter::Duplicate,
            IssueFilter::OverLength,
            IssueFilter::UnderLength,
            IssueFilter::OverPixelWidth,
            IssueFilter::UnderPixelWidth,
            IssueFilter::Multiple,
        ],
        ResultTab::H1 => &[
            IssueFilter::All,
            IssueFilter::Missing,
            IssueFilter::Duplicate,
            IssueFilter::OverLength,
            IssueFilter::Multiple,
        ],
        ResultTab::H2 => &[
            IssueFilter::All,
            IssueFilter::Missing,
            IssueFilter::Duplicate,
            IssueFilter::OverLength,
        ],
        ResultTab::Content => &[
            IssueFilter::All,
            IssueFilter::ExactDuplicates,
            IssueFilter::NearDuplicates,
            IssueFilter::LowContent,
            IssueFilter::SsrContentMissing,
            IssueFilter::BlockedByRobots,
            IssueFilter::ReadabilityDifficult,
            IssueFilter::ReadabilityVeryDifficult,
        ],
        ResultTab::Images => &[
            IssueFilter::All,
            IssueFilter::MissingAltText,
            IssueFilter::MissingAltAttribute,
            IssueFilter::AltOver100,
            IssueFilter::MissingSizeAttributes,
        ],
        ResultTab::Canonicals => &[
            IssueFilter::All,
            IssueFilter::ContainsCanonical,
            IssueFilter::SelfReferencing,
            IssueFilter::Canonicalised,
            IssueFilter::MissingCanonical,
        ],
        ResultTab::Hreflang => &[
            IssueFilter::All,
            IssueFilter::ContainsHreflang,
            IssueFilter::MissingHreflang,
            IssueFilter::HreflangMissingReturnTag,
            IssueFilter::HreflangInvalidLang,
            IssueFilter::HreflangMissingXDefault,
            IssueFilter::HreflangNonCanonical,
        ],
        ResultTab::StructuredData => &[
            IssueFilter::All,
            IssueFilter::HasStructuredData,
            IssueFilter::MissingStructuredData,
            IssueFilter::SdErrors,
            IssueFilter::SdWarnings,
            IssueFilter::JsonLdUrls,
            IssueFilter::MicrodataUrls,
            IssueFilter::ParseErrors,
            IssueFilter::SdTypeArticle,
            IssueFilter::SdTypeProduct,
            IssueFilter::SdTypeFaq,
            IssueFilter::SdTypeHowTo,
            IssueFilter::SdTypeRecipe,
            IssueFilter::SdTypeVideo,
            IssueFilter::SdTypeBreadcrumb,
            IssueFilter::SdTypeOrganization,
        ],
        ResultTab::Accessibility => &[
            IssueFilter::All,
            IssueFilter::A11yImageAlt,
            IssueFilter::A11yLabel,
            IssueFilter::A11yLinkName,
            IssueFilter::A11yButtonName,
            IssueFilter::A11yColorContrast,
            IssueFilter::A11yHtmlHasLang,
            IssueFilter::A11yHeadingOrder,
        ],
        ResultTab::Performance => &[
            IssueFilter::All,
            IssueFilter::SlowLcp,
            IssueFilter::SlowCls,
            IssueFilter::SlowInp,
            IssueFilter::SlowTtfb,
        ],
        ResultTab::Ecommerce => &[
            IssueFilter::All,
            IssueFilter::IsProductPage,
            IssueFilter::MissingPrice,
            IssueFilter::MissingAvailability,
            IssueFilter::MissingSku,
            IssueFilter::MissingGtin,
            IssueFilter::MissingBrand,
            IssueFilter::MissingReviewRating,
            IssueFilter::MissingProductImage,
        ],
        ResultTab::Sitemaps => &[
            IssueFilter::All,
            IssueFilter::UrlsInSitemap,
            IssueFilter::UrlsNotInSitemap,
            IssueFilter::SitemapOrphans,
            IssueFilter::NonIndexableInSitemap,
        ],
        ResultTab::Security => &[
            IssueFilter::All,
            IssueFilter::MissingHttps,
            IssueFilter::MissingHsts,
            IssueFilter::MissingCsp,
            IssueFilter::MissingFrameGuard,
            IssueFilter::MissingContentTypeOptions,
        ],
        ResultTab::Url => &[
            IssueFilter::All,
            IssueFilter::UrlNonAscii,
            IssueFilter::UrlUppercase,
            IssueFilter::UrlUnderscores,
            IssueFilter::UrlMultipleSlashes,
            IssueFilter::UrlParameters,
            IssueFilter::UrlOverLength,
            IssueFilter::UrlSpaces,
        ],
        ResultTab::Directives => &[
            IssueFilter::All,
            IssueFilter::DirectiveNoindex,
            IssueFilter::DirectiveNofollow,
            IssueFilter::DirectiveNoarchive,
            IssueFilter::DirectiveNosnippet,
            IssueFilter::DirectiveNone,
        ],
        ResultTab::Overview => &[
            IssueFilter::All,
            IssueFilter::IssueTypeError,
            IssueFilter::IssueTypeOpportunity,
            IssueFilter::IssueTypeWarning,
            IssueFilter::PriorityHigh,
            IssueFilter::PriorityMedium,
            IssueFilter::PriorityLow,
        ],
        ResultTab::Links => &[
            IssueFilter::All,
            IssueFilter::LinkBroken,
            IssueFilter::LinkRedirected,
            IssueFilter::LinkNofollow,
            IssueFilter::LinkExternal,
        ],
        ResultTab::SiteStructure => &[
            IssueFilter::All,
            IssueFilter::DepthShallow,
            IssueFilter::DepthMedium,
            IssueFilter::DepthDeep,
        ],
    }
}

/// Test-facing wrapper: returns the URLs of the pages a given tab+filter
/// selects. Mirrors exactly what the grid does (builds the same occurrence
/// counts), so the filter-coverage test can assert filter behavior end-to-end.
#[cfg(test)]
pub fn matching_urls(tab: ResultTab, filter: IssueFilter, pages: &[PageRecord]) -> Vec<String> {
    let occurrence_counts = super::columns::build_occurrence_counts(tab, pages);
    filter_for_tab(tab, filter, pages, &occurrence_counts)
        .into_iter()
        .map(|idx| pages[idx].url.clone())
        .collect()
}

pub(super) fn filter_for_tab(
    tab: ResultTab,
    issue_filter: IssueFilter,
    pages: &[PageRecord],
    occurrence_counts: &HashMap<String, usize>,
) -> Vec<usize> {
    let mut indices: Vec<usize> = match tab {
        ResultTab::Internal
        | ResultTab::PageTitles
        | ResultTab::MetaDesc
        | ResultTab::H1
        | ResultTab::H2
        | ResultTab::Content
        | ResultTab::Canonicals
        | ResultTab::StructuredData
        | ResultTab::Hreflang
        | ResultTab::Accessibility
        | ResultTab::Security
        | ResultTab::Url
        | ResultTab::Directives
        | ResultTab::SiteStructure => pages
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_internal)
            .map(|(i, _)| i)
            .collect(),
        ResultTab::External => pages
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.is_internal
                    && p.outlinks
                        .iter()
                        .any(|link| !is_same_domain(&p.url, &link.dst_url))
            })
            .map(|(i, _)| i)
            .collect(),
        ResultTab::Images => pages
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_internal && !p.images.is_empty())
            .map(|(i, _)| i)
            .collect(),
        ResultTab::Links => pages
            .iter()
            .enumerate()
            .filter(|(_, p)| p.is_internal && !p.outlinks.is_empty())
            .map(|(i, _)| i)
            .collect(),
        _ => (0..pages.len()).collect(),
    };

    if issue_filter != IssueFilter::All {
        if let Some(field_key) = primary_field_key(tab) {
            let thresholds = length_thresholds(tab);
            indices.retain(|&idx| {
                let page = &pages[idx];
                let val = field_value(page, field_key).unwrap_or("");
                let len = val.len();
                let occurrences = occurrence_counts
                    .get(val)
                    .copied()
                    .unwrap_or(if val.is_empty() { 0 } else { 1 });
                let count = field_count(page, field_key);

                match issue_filter {
                    IssueFilter::Missing => val.is_empty(),
                    IssueFilter::Duplicate => occurrences > 1,
                    IssueFilter::OverLength => {
                        thresholds.is_some_and(|(_, max)| len > max && !val.is_empty())
                    }
                    IssueFilter::UnderLength => {
                        thresholds.is_some_and(|(min, _)| len < min && len > 0)
                    }
                    IssueFilter::Multiple => {
                        let has_secondary = match field_key {
                            "title" => page.title_2.is_some(),
                            "meta_description" => page.meta_description_2.is_some(),
                            "h1" => page.h1_2.is_some(),
                            "h2" => page.h2_2.is_some(),
                            _ => false,
                        };
                        count > 1 || has_secondary
                    }
                    IssueFilter::SameAsH1 => page
                        .h1
                        .as_deref()
                        .is_some_and(|h1| val == h1 && !val.is_empty()),
                    _ => true,
                }
            });
        }

        match issue_filter {
            IssueFilter::Html => indices.retain(|&idx| {
                pages[idx]
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.starts_with("text/html"))
            }),
            IssueFilter::Images => indices.retain(|&idx| {
                pages[idx]
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.starts_with("image/"))
            }),
            IssueFilter::Css => indices.retain(|&idx| {
                pages[idx]
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.contains("css"))
            }),
            IssueFilter::JavaScript => indices.retain(|&idx| {
                pages[idx]
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.contains("javascript"))
            }),
            IssueFilter::Pdf => indices.retain(|&idx| {
                pages[idx]
                    .content_type
                    .as_deref()
                    .is_some_and(|ct| ct.contains("pdf"))
            }),
            IssueFilter::OtherResource => indices.retain(|&idx| {
                let ct = pages[idx].content_type.as_deref().unwrap_or("");
                !ct.starts_with("text/html")
                    && !ct.starts_with("image/")
                    && !ct.contains("css")
                    && !ct.contains("javascript")
                    && !ct.contains("pdf")
            }),
            IssueFilter::NonIndexable => indices.retain(|&idx| {
                pages[idx]
                    .indexability
                    .as_deref()
                    .is_some_and(|v| v == "Non-Indexable")
            }),
            IssueFilter::Status2xx => {
                indices.retain(|&idx| pages[idx].status.is_some_and(|c| (200..300).contains(&c)))
            }
            IssueFilter::Status3xx => {
                indices.retain(|&idx| pages[idx].status.is_some_and(|c| (300..400).contains(&c)))
            }
            IssueFilter::Status4xx => {
                indices.retain(|&idx| pages[idx].status.is_some_and(|c| (400..500).contains(&c)))
            }
            IssueFilter::Status5xx => {
                indices.retain(|&idx| pages[idx].status.is_some_and(|c| c >= 500))
            }
            IssueFilter::ContainsCanonical => indices.retain(|&idx| {
                pages[idx]
                    .canonical
                    .as_deref()
                    .is_some_and(|c| !c.is_empty())
            }),
            IssueFilter::SelfReferencing => indices.retain(|&idx| {
                let page = &pages[idx];
                page.canonical
                    .as_deref()
                    .is_some_and(|c| !c.is_empty() && c == page.url)
            }),
            IssueFilter::Canonicalised => indices.retain(|&idx| {
                let page = &pages[idx];
                page.canonical
                    .as_deref()
                    .is_some_and(|c| !c.is_empty() && c != page.url)
            }),
            IssueFilter::MissingCanonical => {
                indices.retain(|&idx| pages[idx].canonical.as_deref().is_none_or(|c| c.is_empty()))
            }
            IssueFilter::ContainsHreflang => {
                indices.retain(|&idx| !pages[idx].hreflang_tags.is_empty())
            }
            IssueFilter::MissingHreflang => {
                indices.retain(|&idx| pages[idx].hreflang_tags.is_empty())
            }
            IssueFilter::HreflangMissingReturnTag => indices.retain(|&idx| {
                pages[idx]
                    .hreflang_issues
                    .iter()
                    .any(|i| matches!(i, HreflangIssue::MissingReturnTag { .. }))
            }),
            IssueFilter::HreflangInvalidLang => indices.retain(|&idx| {
                pages[idx]
                    .hreflang_issues
                    .iter()
                    .any(|i| matches!(i, HreflangIssue::InvalidLanguageCode { .. }))
            }),
            IssueFilter::HreflangMissingXDefault => indices.retain(|&idx| {
                pages[idx]
                    .hreflang_issues
                    .iter()
                    .any(|i| matches!(i, HreflangIssue::MissingXDefault))
            }),
            IssueFilter::HreflangNonCanonical => indices.retain(|&idx| {
                pages[idx]
                    .hreflang_issues
                    .iter()
                    .any(|i| matches!(i, HreflangIssue::NonCanonicalUrl { .. }))
            }),
            IssueFilter::HasStructuredData => {
                indices.retain(|&idx| !pages[idx].sd_types.is_empty())
            }
            IssueFilter::MissingStructuredData => {
                indices.retain(|&idx| pages[idx].sd_types.is_empty())
            }
            IssueFilter::SdErrors => indices.retain(|&idx| pages[idx].sd_errors > 0),
            IssueFilter::SdWarnings => indices.retain(|&idx| pages[idx].sd_warnings > 0),
            IssueFilter::JsonLdUrls => indices.retain(|&idx| pages[idx].sd_jsonld_count > 0),
            IssueFilter::MicrodataUrls => indices.retain(|&idx| pages[idx].sd_microdata_count > 0),
            IssueFilter::ParseErrors => indices.retain(|&idx| pages[idx].sd_errors > 0),
            IssueFilter::MissingAltText => indices.retain(|&idx| {
                pages[idx]
                    .images
                    .iter()
                    .any(|img| img.has_alt_attr && img.alt.as_deref().is_none_or(|a| a.is_empty()))
            }),
            IssueFilter::MissingAltAttribute => {
                indices.retain(|&idx| pages[idx].images.iter().any(|img| !img.has_alt_attr))
            }
            IssueFilter::AltOver100 => indices.retain(|&idx| {
                pages[idx]
                    .images
                    .iter()
                    .any(|img| img.alt.as_deref().is_some_and(|a| a.len() > 100))
            }),
            IssueFilter::MissingSizeAttributes => indices.retain(|&idx| {
                pages[idx]
                    .images
                    .iter()
                    .any(|img| img.width.is_none() || img.height.is_none())
            }),
            IssueFilter::UrlsInSitemap => {
                indices.retain(|&idx| pages[idx].in_sitemap == Some(true))
            }
            IssueFilter::UrlsNotInSitemap => {
                indices.retain(|&idx| pages[idx].in_sitemap == Some(false))
            }
            IssueFilter::SitemapOrphans => indices
                .retain(|&idx| pages[idx].in_sitemap == Some(true) && pages[idx].status.is_none()),
            IssueFilter::NonIndexableInSitemap => indices.retain(|&idx| {
                pages[idx].in_sitemap == Some(true)
                    && pages[idx]
                        .indexability
                        .as_deref()
                        .is_some_and(|v| v == "Non-Indexable")
            }),
            IssueFilter::IsProductPage => indices.retain(|&idx| pages[idx].ecommerce.is_some()),
            IssueFilter::MissingPrice => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| a.price.is_none())
            }),
            IssueFilter::MissingAvailability => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| a.availability.is_none())
            }),
            IssueFilter::MissingSku => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| a.sku.is_none())
            }),
            IssueFilter::MissingGtin => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| a.gtin.is_none())
            }),
            IssueFilter::MissingBrand => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| a.brand.is_none())
            }),
            IssueFilter::MissingReviewRating => indices.retain(|&idx| {
                pages[idx]
                    .ecommerce
                    .as_ref()
                    .is_some_and(|a| !a.has_review_or_rating)
            }),
            IssueFilter::MissingProductImage => {
                indices.retain(|&idx| pages[idx].ecommerce.as_ref().is_some_and(|a| !a.has_image))
            }
            IssueFilter::A11yImageAlt => {
                indices.retain(|&idx| pages[idx].a11y_issues.iter().any(|i| i.rule == "image-alt"))
            }
            IssueFilter::A11yLabel => {
                indices.retain(|&idx| pages[idx].a11y_issues.iter().any(|i| i.rule == "label"))
            }
            IssueFilter::A11yLinkName => {
                indices.retain(|&idx| pages[idx].a11y_issues.iter().any(|i| i.rule == "link-name"))
            }
            IssueFilter::A11yButtonName => indices.retain(|&idx| {
                pages[idx]
                    .a11y_issues
                    .iter()
                    .any(|i| i.rule == "button-name")
            }),
            IssueFilter::A11yColorContrast => indices.retain(|&idx| {
                pages[idx]
                    .a11y_issues
                    .iter()
                    .any(|i| i.rule == "color-contrast")
            }),
            IssueFilter::A11yHtmlHasLang => indices.retain(|&idx| {
                pages[idx]
                    .a11y_issues
                    .iter()
                    .any(|i| i.rule == "html-has-lang")
            }),
            IssueFilter::A11yHeadingOrder => indices.retain(|&idx| {
                pages[idx]
                    .a11y_issues
                    .iter()
                    .any(|i| i.rule == "heading-order")
            }),
            IssueFilter::ExactDuplicates => {
                let mut hash_counts: HashMap<&str, usize> = HashMap::new();
                for page in pages {
                    if let Some(hash) = page.content_hash.as_deref() {
                        *hash_counts.entry(hash).or_insert(0) += 1;
                    }
                }
                indices.retain(|&idx| {
                    pages[idx]
                        .content_hash
                        .as_deref()
                        .is_some_and(|h| *hash_counts.get(h).unwrap_or(&0) > 1)
                });
            }
            IssueFilter::NearDuplicates => {
                indices.retain(|&idx| pages[idx].near_duplicate_count.is_some_and(|c| c > 0))
            }
            IssueFilter::LowContent => {
                indices.retain(|&idx| pages[idx].word_count.is_some_and(|w| w > 0 && w < 100))
            }
            IssueFilter::SsrContentMissing => {
                indices.retain(|&idx| pages[idx].ssr_content_missing == Some(true))
            }
            IssueFilter::BlockedByRobots => {
                indices.retain(|&idx| pages[idx].blocked_by_robots == Some(true))
            }
            IssueFilter::ReadabilityDifficult => indices.retain(|&idx| {
                pages[idx]
                    .flesch_reading_ease
                    .is_some_and(|s| (30.0..50.0).contains(&s))
            }),
            IssueFilter::ReadabilityVeryDifficult => {
                indices.retain(|&idx| pages[idx].flesch_reading_ease.is_some_and(|s| s < 30.0))
            }
            IssueFilter::SlowLcp => {
                indices.retain(|&idx| pages[idx].lcp_ms.is_some_and(|ms| ms > 4000))
            }
            IssueFilter::SlowCls => indices.retain(|&idx| pages[idx].cls.is_some_and(|v| v > 0.25)),
            IssueFilter::SlowInp => {
                indices.retain(|&idx| pages[idx].inp_ms.is_some_and(|ms| ms > 500))
            }
            IssueFilter::SlowTtfb => {
                indices.retain(|&idx| pages[idx].ttfb_ms.is_some_and(|ms| ms > 1800))
            }
            IssueFilter::SdTypeArticle => indices.retain(|&idx| {
                pages[idx]
                    .sd_types
                    .iter()
                    .any(|t| t == "Article" || t == "NewsArticle" || t == "BlogPosting")
            }),
            IssueFilter::SdTypeProduct => {
                indices.retain(|&idx| pages[idx].sd_types.iter().any(|t| t == "Product"))
            }
            IssueFilter::SdTypeFaq => {
                indices.retain(|&idx| pages[idx].sd_types.iter().any(|t| t == "FAQPage"))
            }
            IssueFilter::SdTypeHowTo => {
                indices.retain(|&idx| pages[idx].sd_types.iter().any(|t| t == "HowTo"))
            }
            IssueFilter::SdTypeRecipe => {
                indices.retain(|&idx| pages[idx].sd_types.iter().any(|t| t == "Recipe"))
            }
            IssueFilter::SdTypeVideo => indices.retain(|&idx| {
                pages[idx]
                    .sd_types
                    .iter()
                    .any(|t| t == "VideoObject" || t == "Video")
            }),
            IssueFilter::SdTypeBreadcrumb => {
                indices.retain(|&idx| pages[idx].sd_types.iter().any(|t| t == "BreadcrumbList"))
            }
            IssueFilter::SdTypeOrganization => indices.retain(|&idx| {
                pages[idx]
                    .sd_types
                    .iter()
                    .any(|t| t == "Organization" || t == "LocalBusiness")
            }),
            IssueFilter::MissingHttps => {
                indices.retain(|&idx| !pages[idx].url.starts_with("https://"))
            }
            IssueFilter::MissingHsts => indices
                .retain(|&idx| !header_exists(&pages[idx].headers, "strict-transport-security")),
            IssueFilter::MissingCsp => indices
                .retain(|&idx| !header_exists(&pages[idx].headers, "content-security-policy")),
            IssueFilter::MissingFrameGuard => {
                indices.retain(|&idx| !header_exists(&pages[idx].headers, "x-frame-options"))
            }
            IssueFilter::MissingContentTypeOptions => {
                indices.retain(|&idx| !header_exists(&pages[idx].headers, "x-content-type-options"))
            }
            IssueFilter::UrlNonAscii => indices.retain(|&idx| !pages[idx].url.is_ascii()),
            IssueFilter::UrlUppercase => {
                indices.retain(|&idx| pages[idx].url.chars().any(|c| c.is_ascii_uppercase()))
            }
            IssueFilter::UrlUnderscores => indices.retain(|&idx| pages[idx].url.contains('_')),
            IssueFilter::UrlMultipleSlashes => indices.retain(|&idx| {
                if let Ok(parsed) = url::Url::parse(&pages[idx].url) {
                    parsed.path().contains("//")
                } else {
                    false
                }
            }),
            IssueFilter::UrlParameters => indices.retain(|&idx| pages[idx].url.contains('?')),
            IssueFilter::UrlOverLength => indices.retain(|&idx| pages[idx].url.len() > 115),
            IssueFilter::UrlSpaces => indices.retain(|&idx| pages[idx].url.contains(' ')),
            IssueFilter::DirectiveNoindex => indices.retain(|&idx| {
                let page = &pages[idx];
                page.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("noindex"))
                    || header_value(&page.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("noindex"))
            }),
            IssueFilter::DirectiveNofollow => indices.retain(|&idx| {
                let page = &pages[idx];
                page.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("nofollow"))
                    || header_value(&page.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("nofollow"))
            }),
            IssueFilter::DirectiveNoarchive => indices.retain(|&idx| {
                let page = &pages[idx];
                page.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("noarchive"))
                    || header_value(&page.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("noarchive"))
            }),
            IssueFilter::DirectiveNosnippet => indices.retain(|&idx| {
                let page = &pages[idx];
                page.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("nosnippet"))
                    || header_value(&page.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("nosnippet"))
            }),
            IssueFilter::DirectiveNone => indices.retain(|&idx| {
                let page = &pages[idx];
                page.robots.as_deref().is_some_and(|r| {
                    r.to_ascii_lowercase()
                        .split(',')
                        .any(|d| d.trim() == "none")
                }) || header_value(&page.headers, "x-robots-tag").is_some_and(|v| {
                    v.to_ascii_lowercase()
                        .split(',')
                        .any(|d| d.trim() == "none")
                })
            }),
            IssueFilter::Redirects => indices.retain(|&idx| pages[idx].redirect_url.is_some()),
            IssueFilter::RedirectLoop => {
                let url_set: HashMap<String, String> = pages
                    .iter()
                    .filter_map(|p| p.redirect_url.as_ref().map(|r| (p.url.clone(), r.clone())))
                    .collect();
                indices.retain(|&idx| {
                    let page = &pages[idx];
                    let Some(ref redirect) = page.redirect_url else {
                        return false;
                    };
                    let mut visited = vec![];
                    let mut current = redirect.clone();
                    loop {
                        if current == page.url {
                            return true;
                        }
                        if visited.contains(&current) {
                            return false;
                        }
                        visited.push(current.clone());
                        let Some(next) = url_set.get(&current) else {
                            return false;
                        };
                        current = next.clone();
                    }
                });
            }
            IssueFilter::OverPixelWidth => {
                if let Some((_, max_pw)) = pixel_width_thresholds(tab) {
                    indices.retain(|&idx| {
                        let pw = match tab {
                            ResultTab::MetaDesc => pages[idx].meta_description_pixel_width,
                            _ => pages[idx].title_pixel_width,
                        };
                        pw.is_some_and(|w| w > max_pw)
                    });
                }
            }
            IssueFilter::UnderPixelWidth => {
                if let Some((min_pw, _)) = pixel_width_thresholds(tab) {
                    indices.retain(|&idx| {
                        let pw = match tab {
                            ResultTab::MetaDesc => pages[idx].meta_description_pixel_width,
                            _ => pages[idx].title_pixel_width,
                        };
                        pw.is_some_and(|w| w < min_pw && w > 0)
                    });
                }
            }
            _ => {}
        }
    }

    indices
}

use super::types::FlatRow;

fn link_row_matches_filter(
    link: &crate::crawl::event::Outlink,
    page: &PageRecord,
    filter: IssueFilter,
    all_pages: &[PageRecord],
) -> bool {
    let dst_is_external = !is_same_domain(&page.url, &link.dst_url);
    let rel_nofollow = link
        .rel
        .as_deref()
        .is_some_and(|r| r.to_ascii_lowercase().contains("nofollow"));
    let dst_status = all_pages
        .iter()
        .find(|p| p.url == link.dst_url)
        .and_then(|p| p.status);

    match filter {
        IssueFilter::LinkExternal => dst_is_external,
        IssueFilter::LinkNofollow => rel_nofollow,
        IssueFilter::LinkBroken => dst_status.is_some_and(|c| c >= 400),
        IssueFilter::LinkRedirected => dst_status.is_some_and(|c| (300..400).contains(&c)),
        _ => true,
    }
}

pub(super) fn flat_row_matches_filter(
    row: &FlatRow,
    page: &PageRecord,
    filter: IssueFilter,
    all_pages: &[PageRecord],
) -> bool {
    match row {
        FlatRow::Image { item, .. } => {
            let Some(image) = page.images.get(*item) else {
                return false;
            };
            image_matches_filter(image, filter)
        }
        FlatRow::A11yIssue { item, .. } => {
            let Some(issue) = page.a11y_issues.get(*item) else {
                return false;
            };
            a11y_issue_matches_filter(issue, filter)
        }
        FlatRow::SdItem { item, .. } => {
            let Some(sd_item) = page.sd_items.get(*item) else {
                return false;
            };
            sd_item_matches_filter(sd_item, page, filter)
        }
        FlatRow::LinkRow { item, .. } => {
            let Some(link) = page.outlinks.get(*item) else {
                return false;
            };
            link_row_matches_filter(link, page, filter, all_pages)
        }
        FlatRow::Outlink { .. } | FlatRow::Hreflang { .. } | FlatRow::IssuesRow { .. } => true,
        FlatRow::DirectoryAggregate { depth, .. } => match filter {
            IssueFilter::DepthShallow => *depth <= 1,
            IssueFilter::DepthMedium => *depth >= 2 && *depth <= 3,
            IssueFilter::DepthDeep => *depth >= 4,
            _ => true,
        },
    }
}

fn image_matches_filter(image: &ImageRef, filter: IssueFilter) -> bool {
    match filter {
        IssueFilter::MissingAltText => {
            image.has_alt_attr && image.alt.as_deref().is_none_or(|a| a.is_empty())
        }
        IssueFilter::MissingAltAttribute => !image.has_alt_attr,
        IssueFilter::AltOver100 => image.alt.as_deref().is_some_and(|a| a.len() > 100),
        IssueFilter::MissingSizeAttributes => image.width.is_none() || image.height.is_none(),
        _ => true,
    }
}

fn a11y_issue_matches_filter(issue: &A11yIssue, filter: IssueFilter) -> bool {
    match filter {
        IssueFilter::A11yImageAlt => issue.rule == "image-alt",
        IssueFilter::A11yLabel => issue.rule == "label",
        IssueFilter::A11yLinkName => issue.rule == "link-name",
        IssueFilter::A11yButtonName => issue.rule == "button-name",
        IssueFilter::A11yColorContrast => issue.rule == "color-contrast",
        IssueFilter::A11yHtmlHasLang => issue.rule == "html-has-lang",
        IssueFilter::A11yHeadingOrder => issue.rule == "heading-order",
        _ => true,
    }
}

fn sd_item_matches_filter(sd_item: &SdItem, page: &PageRecord, filter: IssueFilter) -> bool {
    match filter {
        IssueFilter::JsonLdUrls => sd_item.format == SdFormat::JsonLd,
        IssueFilter::MicrodataUrls => sd_item.format == SdFormat::Microdata,
        IssueFilter::HasStructuredData => true,
        IssueFilter::MissingStructuredData => false,
        IssueFilter::SdErrors => page.sd_errors > 0,
        IssueFilter::SdWarnings => page.sd_warnings > 0,
        IssueFilter::ParseErrors => page.sd_errors > 0,
        IssueFilter::SdTypeArticle => {
            sd_item.type_name == "Article"
                || sd_item.type_name == "NewsArticle"
                || sd_item.type_name == "BlogPosting"
        }
        IssueFilter::SdTypeProduct => sd_item.type_name == "Product",
        IssueFilter::SdTypeFaq => sd_item.type_name == "FAQPage",
        IssueFilter::SdTypeHowTo => sd_item.type_name == "HowTo",
        IssueFilter::SdTypeRecipe => sd_item.type_name == "Recipe",
        IssueFilter::SdTypeVideo => {
            sd_item.type_name == "VideoObject" || sd_item.type_name == "Video"
        }
        IssueFilter::SdTypeBreadcrumb => sd_item.type_name == "BreadcrumbList",
        IssueFilter::SdTypeOrganization => {
            sd_item.type_name == "Organization" || sd_item.type_name == "LocalBusiness"
        }
        _ => true,
    }
}
