use std::collections::HashMap;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Window, div, px,
};
use gpui_component::{
    ActiveTheme,
    table::{Column, ColumnFixed, ColumnSort, DataTable, TableDelegate, TableEvent, TableState},
};

use crate::crawl::engine::is_same_domain;
use crate::crawl::event::{A11yIssue, ImageRef, Outlink, PageRecord, SdFormat, SdItem};
use crate::ui::tag::{Tone, count_tone, indexability_tone, status_code_tone, tone_tag};
use crate::views::ResultTab;

#[derive(Clone, Debug)]
enum FlatRow {
    Image { page: usize, item: usize },
    Outlink { page: usize, item: usize },
    A11yIssue { page: usize, item: usize },
    Hreflang { page: usize, item: usize },
    SdItem { page: usize, item: usize },
    OverviewIssue { label: String, count: usize },
}

fn tab_is_flattened(tab: ResultTab) -> bool {
    matches!(
        tab,
        ResultTab::Images
            | ResultTab::External
            | ResultTab::Accessibility
            | ResultTab::Hreflang
            | ResultTab::StructuredData
            | ResultTab::Overview
    )
}

#[derive(Clone, Debug, Default)]
pub struct TabCounts {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Clone, Debug)]
pub enum ResultsGridEvent {
    Selected(usize),
    OverviewDrillDown { tab: ResultTab, filter: IssueFilter },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueFilter {
    All,
    Missing,
    Duplicate,
    OverLength,
    UnderLength,
    Multiple,
    SameAsH1,
    NonIndexable,
    Html,
    #[allow(dead_code)]
    Images,
    Status2xx,
    Status3xx,
    Status4xx,
    Status5xx,
    ContainsCanonical,
    SelfReferencing,
    Canonicalised,
    MissingCanonical,
    ContainsHreflang,
    MissingHreflang,
    HasStructuredData,
    MissingStructuredData,
    SdErrors,
    SdWarnings,
    JsonLdUrls,
    MicrodataUrls,
    ParseErrors,
    MissingAltText,
    MissingAltAttribute,
    AltOver100,
    MissingSizeAttributes,
    UrlsInSitemap,
    UrlsNotInSitemap,
    SitemapOrphans,
    NonIndexableInSitemap,
    IsProductPage,
    MissingPrice,
    MissingAvailability,
    MissingSku,
    MissingGtin,
    MissingBrand,
    MissingReviewRating,
    MissingProductImage,
    A11yImageAlt,
    A11yLabel,
    A11yLinkName,
    A11yButtonName,
    A11yColorContrast,
    A11yHtmlHasLang,
    A11yHeadingOrder,
    ExactDuplicates,
    NearDuplicates,
    LowContent,
    SlowLcp,
    SlowCls,
    SlowInp,
    SlowTtfb,
    AllGoodPerformance,
    SdTypeArticle,
    SdTypeProduct,
    SdTypeFaq,
    SdTypeHowTo,
    SdTypeRecipe,
    SdTypeVideo,
    SdTypeBreadcrumb,
    SdTypeOrganization,
    MissingHttps,
    MissingHsts,
    MissingCsp,
    MissingFrameGuard,
    MissingContentTypeOptions,
    UrlNonAscii,
    UrlUppercase,
    UrlUnderscores,
    UrlMultipleSlashes,
    UrlParameters,
    UrlOverLength,
    UrlSpaces,
    DirectiveNoindex,
    DirectiveNofollow,
    DirectiveNoarchive,
    DirectiveNosnippet,
    DirectiveNone,
    Redirects,
    RedirectLoop,
    OverPixelWidth,
    UnderPixelWidth,
}

impl IssueFilter {
    pub fn label(self) -> &'static str {
        match self {
            IssueFilter::All => "All",
            IssueFilter::Missing => "Missing",
            IssueFilter::Duplicate => "Duplicate",
            IssueFilter::OverLength => "Over Length",
            IssueFilter::UnderLength => "Under Length",
            IssueFilter::Multiple => "Multiple",
            IssueFilter::SameAsH1 => "Same as H1",
            IssueFilter::NonIndexable => "Non-Indexable",
            IssueFilter::Html => "HTML",
            IssueFilter::Images => "Images",
            IssueFilter::Status2xx => "2xx",
            IssueFilter::Status3xx => "3xx",
            IssueFilter::Status4xx => "4xx",
            IssueFilter::Status5xx => "5xx",
            IssueFilter::ContainsCanonical => "Contains Canonical",
            IssueFilter::SelfReferencing => "Self-Referencing",
            IssueFilter::Canonicalised => "Canonicalised",
            IssueFilter::MissingCanonical => "Missing",
            IssueFilter::ContainsHreflang => "Contains Hreflang",
            IssueFilter::MissingHreflang => "Missing",
            IssueFilter::HasStructuredData => "Has Structured Data",
            IssueFilter::MissingStructuredData => "Missing",
            IssueFilter::SdErrors => "Errors",
            IssueFilter::SdWarnings => "Warnings",
            IssueFilter::JsonLdUrls => "JSON-LD",
            IssueFilter::MicrodataUrls => "Microdata",
            IssueFilter::ParseErrors => "Parse Errors",
            IssueFilter::MissingAltText => "Missing Alt Text",
            IssueFilter::MissingAltAttribute => "Missing Alt Attribute",
            IssueFilter::AltOver100 => "Alt Over 100",
            IssueFilter::MissingSizeAttributes => "Missing Size Attrs",
            IssueFilter::UrlsInSitemap => "In Sitemap",
            IssueFilter::UrlsNotInSitemap => "Not in Sitemap",
            IssueFilter::SitemapOrphans => "Orphan URLs",
            IssueFilter::NonIndexableInSitemap => "Non-Indexable in Sitemap",
            IssueFilter::IsProductPage => "Product Pages",
            IssueFilter::MissingPrice => "Missing Price",
            IssueFilter::MissingAvailability => "Missing Availability",
            IssueFilter::MissingSku => "Missing SKU",
            IssueFilter::MissingGtin => "Missing GTIN",
            IssueFilter::MissingBrand => "Missing Brand",
            IssueFilter::MissingReviewRating => "Missing Review/Rating",
            IssueFilter::MissingProductImage => "Missing Image",
            IssueFilter::A11yImageAlt => "image-alt",
            IssueFilter::A11yLabel => "label",
            IssueFilter::A11yLinkName => "link-name",
            IssueFilter::A11yButtonName => "button-name",
            IssueFilter::A11yColorContrast => "color-contrast",
            IssueFilter::A11yHtmlHasLang => "html-has-lang",
            IssueFilter::A11yHeadingOrder => "heading-order",
            IssueFilter::ExactDuplicates => "Exact Duplicates",
            IssueFilter::NearDuplicates => "Near Duplicates",
            IssueFilter::LowContent => "Low Content",
            IssueFilter::SlowLcp => "Slow LCP",
            IssueFilter::SlowCls => "Slow CLS",
            IssueFilter::SlowInp => "Slow INP",
            IssueFilter::SlowTtfb => "Slow TTFB",
            IssueFilter::AllGoodPerformance => "All Good",
            IssueFilter::SdTypeArticle => "Article",
            IssueFilter::SdTypeProduct => "Product",
            IssueFilter::SdTypeFaq => "FAQ",
            IssueFilter::SdTypeHowTo => "HowTo",
            IssueFilter::SdTypeRecipe => "Recipe",
            IssueFilter::SdTypeVideo => "Video",
            IssueFilter::SdTypeBreadcrumb => "Breadcrumb",
            IssueFilter::SdTypeOrganization => "Organization",
            IssueFilter::MissingHttps => "Missing HTTPS",
            IssueFilter::MissingHsts => "Missing HSTS",
            IssueFilter::MissingCsp => "Missing CSP",
            IssueFilter::MissingFrameGuard => "Missing Frame Guard",
            IssueFilter::MissingContentTypeOptions => "Missing X-Content-Type",
            IssueFilter::UrlNonAscii => "Non-ASCII",
            IssueFilter::UrlUppercase => "Uppercase",
            IssueFilter::UrlUnderscores => "Underscores",
            IssueFilter::UrlMultipleSlashes => "Multiple Slashes",
            IssueFilter::UrlParameters => "Parameters",
            IssueFilter::UrlOverLength => "Over 115 Chars",
            IssueFilter::UrlSpaces => "Contains Space",
            IssueFilter::DirectiveNoindex => "Noindex",
            IssueFilter::DirectiveNofollow => "Nofollow",
            IssueFilter::DirectiveNoarchive => "Noarchive",
            IssueFilter::DirectiveNosnippet => "Nosnippet",
            IssueFilter::DirectiveNone => "None",
            IssueFilter::Redirects => "Redirects",
            IssueFilter::RedirectLoop => "Redirect Loop",
            IssueFilter::OverPixelWidth => "Over Pixel Width",
            IssueFilter::UnderPixelWidth => "Under Pixel Width",
        }
    }

    pub fn tone(self) -> crate::ui::tag::Tone {
        use crate::ui::tag::Tone;
        match self {
            Self::All
            | Self::Html
            | Self::Images
            | Self::Status2xx
            | Self::Status3xx
            | Self::ContainsCanonical
            | Self::SelfReferencing
            | Self::ContainsHreflang
            | Self::HasStructuredData
            | Self::JsonLdUrls
            | Self::MicrodataUrls
            | Self::UrlsInSitemap
            | Self::IsProductPage
            | Self::AllGoodPerformance
            | Self::SdTypeArticle
            | Self::SdTypeProduct
            | Self::SdTypeFaq
            | Self::SdTypeHowTo
            | Self::SdTypeRecipe
            | Self::SdTypeVideo
            | Self::SdTypeBreadcrumb
            | Self::SdTypeOrganization
            | Self::DirectiveNone => Tone::Neutral,

            Self::Status4xx
            | Self::Status5xx
            | Self::SdErrors
            | Self::ParseErrors
            | Self::SitemapOrphans
            | Self::A11yImageAlt
            | Self::A11yLabel
            | Self::A11yLinkName
            | Self::A11yButtonName
            | Self::A11yColorContrast
            | Self::A11yHtmlHasLang
            | Self::RedirectLoop
            | Self::MissingHttps
            | Self::ExactDuplicates
            | Self::UrlNonAscii
            | Self::UrlSpaces
            | Self::DirectiveNoindex => Tone::Err,

            Self::NonIndexable
            | Self::Missing
            | Self::Duplicate
            | Self::OverLength
            | Self::UnderLength
            | Self::Multiple
            | Self::SameAsH1
            | Self::Canonicalised
            | Self::MissingCanonical
            | Self::MissingHreflang
            | Self::MissingStructuredData
            | Self::SdWarnings
            | Self::A11yHeadingOrder
            | Self::MissingAltText
            | Self::MissingAltAttribute
            | Self::AltOver100
            | Self::MissingSizeAttributes
            | Self::NonIndexableInSitemap
            | Self::UrlsNotInSitemap
            | Self::MissingPrice
            | Self::MissingAvailability
            | Self::MissingSku
            | Self::MissingGtin
            | Self::MissingBrand
            | Self::MissingReviewRating
            | Self::MissingProductImage
            | Self::NearDuplicates
            | Self::LowContent
            | Self::SlowLcp
            | Self::SlowCls
            | Self::SlowInp
            | Self::SlowTtfb
            | Self::Redirects
            | Self::MissingHsts
            | Self::MissingCsp
            | Self::MissingFrameGuard
            | Self::MissingContentTypeOptions
            | Self::UrlUppercase
            | Self::UrlUnderscores
            | Self::UrlMultipleSlashes
            | Self::UrlParameters
            | Self::UrlOverLength
            | Self::DirectiveNofollow
            | Self::DirectiveNoarchive
            | Self::DirectiveNosnippet
            | Self::OverPixelWidth
            | Self::UnderPixelWidth => Tone::Warn,
        }
    }
}

pub fn filters_for_tab(tab: ResultTab) -> &'static [IssueFilter] {
    match tab {
        ResultTab::Internal => &[
            IssueFilter::All,
            IssueFilter::Html,
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
            IssueFilter::AllGoodPerformance,
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
        ResultTab::Overview => &[IssueFilter::All],
    }
}

pub struct ResultsDelegate {
    all_pages: Vec<PageRecord>,
    filtered_indices: Vec<usize>,
    flat_rows: Vec<FlatRow>,
    occurrence_counts: HashMap<String, usize>,
    columns: Vec<Column>,
    active_tab: ResultTab,
    issue_filter: IssueFilter,
    root_origin: Option<String>,
}

impl ResultsDelegate {
    fn new() -> Self {
        let tab = ResultTab::Internal;
        Self {
            all_pages: Vec::new(),
            filtered_indices: Vec::new(),
            flat_rows: Vec::new(),
            occurrence_counts: HashMap::new(),
            columns: columns_for_tab(tab),
            active_tab: tab,
            issue_filter: IssueFilter::All,
            root_origin: None,
        }
    }

    pub fn switch_tab(&mut self, tab: ResultTab) {
        self.active_tab = tab;
        self.issue_filter = IssueFilter::All;
        self.columns = columns_for_tab(tab);
        self.rebuild_filter();
    }

    pub fn set_issue_filter(&mut self, filter: IssueFilter) {
        self.issue_filter = filter;
        self.rebuild_filter();
    }

    pub fn set_root_url(&mut self, root_url: &str) {
        self.root_origin = url::Url::parse(root_url)
            .ok()
            .map(|u| u.origin().ascii_serialization());
    }

    pub fn push(&mut self, record: PageRecord) {
        self.all_pages.push(record);
        self.rebuild_filter();
    }

    pub fn clear(&mut self) {
        self.all_pages.clear();
        self.filtered_indices.clear();
        self.flat_rows.clear();
        self.occurrence_counts.clear();
        self.root_origin = None;
    }

    pub fn record_at(&self, index: usize) -> Option<&PageRecord> {
        if tab_is_flattened(self.active_tab) {
            let page_index = match self.flat_rows.get(index)? {
                FlatRow::Image { page, .. }
                | FlatRow::Outlink { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. } => *page,
                FlatRow::OverviewIssue { .. } => return None,
            };
            self.all_pages.get(page_index)
        } else {
            self.filtered_indices
                .get(index)
                .and_then(|&idx| self.all_pages.get(idx))
        }
    }

    pub fn filtered_count(&self) -> usize {
        if tab_is_flattened(self.active_tab) {
            self.flat_rows.len()
        } else {
            self.filtered_indices.len()
        }
    }

    #[allow(dead_code)]
    pub fn active_tab(&self) -> ResultTab {
        self.active_tab
    }

    pub(super) fn flat_rows(&self) -> &[FlatRow] {
        &self.flat_rows
    }

    pub fn compute_tab_counts(&self) -> HashMap<ResultTab, TabCounts> {
        let internal: Vec<&PageRecord> = self.all_pages.iter().filter(|p| p.is_internal).collect();

        let errors = self
            .all_pages
            .iter()
            .filter(|p| p.status.is_some_and(|c| c >= 400))
            .count();

        let non_indexable = internal
            .iter()
            .filter(|p| p.indexability.as_deref() == Some("Non-Indexable"))
            .count();

        let missing_title = internal
            .iter()
            .filter(|p| p.title.as_deref() == Some(""))
            .count();
        let duplicate_title = {
            let mut title_counts: HashMap<&str, usize> = HashMap::new();
            for p in &internal {
                let t = p.title.as_deref().unwrap_or("");
                *title_counts.entry(t).or_insert(0) += 1;
            }
            internal
                .iter()
                .filter(|p| {
                    *title_counts
                        .get(p.title.as_deref().unwrap_or(""))
                        .unwrap_or(&0)
                        > 1
                })
                .count()
        };

        let over_length_title = internal
            .iter()
            .filter(|p| p.title.as_deref().is_some_and(|t| t.len() > 60))
            .count();

        let missing_desc = internal
            .iter()
            .filter(|p| p.meta_description.as_deref() == Some(""))
            .count();
        let over_length_desc = internal
            .iter()
            .filter(|p| p.meta_description.as_deref().is_some_and(|t| t.len() > 160))
            .count();
        let missing_h1 = internal
            .iter()
            .filter(|p| p.h1.as_deref() == Some(""))
            .count();
        let over_length_h1 = internal
            .iter()
            .filter(|p| p.h1.as_deref().is_some_and(|t| t.len() > 70))
            .count();
        let missing_h2 = internal
            .iter()
            .filter(|p| p.h2.as_deref() == Some(""))
            .count();
        let over_length_h2 = internal
            .iter()
            .filter(|p| p.h2.as_deref().is_some_and(|t| t.len() > 70))
            .count();
        let missing_canonical = internal
            .iter()
            .filter(|p| p.canonical.as_deref() == Some(""))
            .count();

        let mut counts = HashMap::new();
        counts.insert(
            ResultTab::Internal,
            TabCounts {
                total: internal.len(),
                errors,
                warnings: non_indexable,
            },
        );
        counts.insert(
            ResultTab::External,
            TabCounts {
                total: internal
                    .iter()
                    .map(|p| {
                        p.outlinks
                            .iter()
                            .filter(|link| !is_same_domain(&p.url, &link.dst_url))
                            .count()
                    })
                    .sum(),
                errors: 0,
                warnings: 0,
            },
        );
        counts.insert(
            ResultTab::ResponseCodes,
            TabCounts {
                total: self.all_pages.len(),
                errors,
                warnings: self
                    .all_pages
                    .iter()
                    .filter(|p| p.redirect_url.is_some())
                    .count(),
            },
        );
        counts.insert(
            ResultTab::PageTitles,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_title + duplicate_title + over_length_title,
            },
        );
        counts.insert(
            ResultTab::MetaDesc,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_desc + over_length_desc,
            },
        );
        counts.insert(
            ResultTab::H1,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_h1 + over_length_h1,
            },
        );
        counts.insert(
            ResultTab::H2,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_h2 + over_length_h2,
            },
        );
        let exact_dup_count = {
            let mut hash_counts: HashMap<&str, usize> = HashMap::new();
            for p in &internal {
                if let Some(hash) = p.content_hash.as_deref() {
                    *hash_counts.entry(hash).or_insert(0) += 1;
                }
            }
            internal
                .iter()
                .filter(|p| {
                    p.content_hash
                        .as_deref()
                        .is_some_and(|h| *hash_counts.get(h).unwrap_or(&0) > 1)
                })
                .count()
        };
        let near_dup_count = internal
            .iter()
            .filter(|p| p.near_duplicate_count.is_some_and(|c| c > 0))
            .count();
        counts.insert(
            ResultTab::Content,
            TabCounts {
                total: internal.len(),
                errors: exact_dup_count,
                warnings: near_dup_count,
            },
        );
        counts.insert(
            ResultTab::Images,
            TabCounts {
                total: internal.iter().map(|p| p.images.len()).sum(),
                errors: 0,
                warnings: internal
                    .iter()
                    .flat_map(|p| p.images.iter())
                    .filter(|img| {
                        !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty())
                    })
                    .count(),
            },
        );
        counts.insert(
            ResultTab::Canonicals,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_canonical,
            },
        );
        counts.insert(
            ResultTab::Hreflang,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: internal
                    .iter()
                    .filter(|p| p.hreflang_tags.is_empty())
                    .count(),
            },
        );
        let sd_missing = internal.iter().filter(|p| p.sd_types.is_empty()).count();
        let sd_error_count = internal.iter().filter(|p| p.sd_errors > 0).count();
        counts.insert(
            ResultTab::StructuredData,
            TabCounts {
                total: internal.len(),
                errors: sd_error_count,
                warnings: sd_missing,
            },
        );
        counts.insert(
            ResultTab::Performance,
            TabCounts {
                total: internal.len(),
                errors: internal
                    .iter()
                    .filter(|p| p.lcp_ms.is_some_and(|ms| ms > 4000))
                    .count(),
                warnings: internal
                    .iter()
                    .filter(|p| {
                        p.lcp_ms.is_some_and(|ms| ms > 2500 && ms <= 4000)
                            || p.cls.is_some_and(|v| v > 0.1)
                            || p.inp_ms.is_some_and(|ms| ms > 200)
                    })
                    .count(),
            },
        );
        counts.insert(
            ResultTab::Accessibility,
            TabCounts {
                total: self
                    .all_pages
                    .iter()
                    .filter(|p| p.is_internal)
                    .map(|p| p.a11y_issues.len())
                    .sum(),
                errors: self
                    .all_pages
                    .iter()
                    .flat_map(|p| p.a11y_issues.iter())
                    .filter(|i| matches!(i.impact.as_str(), "critical" | "serious"))
                    .count(),
                warnings: self
                    .all_pages
                    .iter()
                    .flat_map(|p| p.a11y_issues.iter())
                    .filter(|i| !matches!(i.impact.as_str(), "critical" | "serious"))
                    .count(),
            },
        );
        let product_count = self
            .all_pages
            .iter()
            .filter(|p| p.ecommerce.is_some())
            .count();
        let missing_price = self
            .all_pages
            .iter()
            .filter(|p| p.ecommerce.as_ref().is_some_and(|a| a.price.is_none()))
            .count();
        counts.insert(
            ResultTab::Ecommerce,
            TabCounts {
                total: product_count,
                errors: 0,
                warnings: missing_price,
            },
        );
        let sitemap_orphan_count = self
            .all_pages
            .iter()
            .filter(|p| p.in_sitemap == Some(true) && p.status.is_none())
            .count();
        let non_indexable_in_sitemap = self
            .all_pages
            .iter()
            .filter(|p| {
                p.in_sitemap == Some(true) && p.indexability.as_deref() == Some("Non-Indexable")
            })
            .count();
        counts.insert(
            ResultTab::Sitemaps,
            TabCounts {
                total: self
                    .all_pages
                    .iter()
                    .filter(|p| p.in_sitemap.is_some())
                    .count(),
                errors: sitemap_orphan_count,
                warnings: non_indexable_in_sitemap,
            },
        );
        let missing_https = internal
            .iter()
            .filter(|p| !p.url.starts_with("https://"))
            .count();
        let missing_hsts = internal
            .iter()
            .filter(|p| !header_exists(&p.headers, "strict-transport-security"))
            .count();
        counts.insert(
            ResultTab::Security,
            TabCounts {
                total: internal.len(),
                errors: missing_https,
                warnings: missing_hsts,
            },
        );
        let url_non_ascii = internal.iter().filter(|p| !p.url.is_ascii()).count();
        let url_uppercase = internal
            .iter()
            .filter(|p| p.url.chars().any(|c| c.is_ascii_uppercase()))
            .count();
        counts.insert(
            ResultTab::Url,
            TabCounts {
                total: internal.len(),
                errors: url_non_ascii + url_uppercase,
                warnings: internal.iter().filter(|p| p.url.contains('_')).count(),
            },
        );
        let directive_noindex = internal
            .iter()
            .filter(|p| {
                p.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("noindex"))
                    || header_value(&p.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("noindex"))
            })
            .count();
        counts.insert(
            ResultTab::Directives,
            TabCounts {
                total: internal.len(),
                errors: directive_noindex,
                warnings: 0,
            },
        );
        let overview_issue_count = build_overview_rows(&self.all_pages).len();
        counts.insert(
            ResultTab::Overview,
            TabCounts {
                total: overview_issue_count,
                errors: overview_issue_count,
                warnings: 0,
            },
        );
        counts
    }

    fn rebuild_filter(&mut self) {
        self.occurrence_counts = build_occurrence_counts(self.active_tab, &self.all_pages);
        self.filtered_indices = filter_for_tab(
            self.active_tab,
            self.issue_filter,
            &self.all_pages,
            &self.occurrence_counts,
        );
        self.rebuild_flat_rows();
    }

    fn rebuild_flat_rows(&mut self) {
        if !tab_is_flattened(self.active_tab) {
            self.flat_rows.clear();
            return;
        }
        if self.active_tab == ResultTab::Overview {
            self.flat_rows = build_overview_rows(&self.all_pages);
            return;
        }
        let active_tab = self.active_tab;
        let all_pages = &self.all_pages;
        self.flat_rows = if active_tab == ResultTab::External {
            self.filtered_indices
                .iter()
                .flat_map(|&page_index| {
                    let Some(page) = all_pages.get(page_index) else {
                        return Vec::<FlatRow>::new();
                    };
                    page.outlinks
                        .iter()
                        .enumerate()
                        .filter(|(_, link)| !is_same_domain(&page.url, &link.dst_url))
                        .map(|(item_index, _)| FlatRow::Outlink {
                            page: page_index,
                            item: item_index,
                        })
                        .collect()
                })
                .collect()
        } else {
            self.filtered_indices
                .iter()
                .flat_map(|&page_index| {
                    let item_count = all_pages
                        .get(page_index)
                        .map(|page| flat_row_item_count(page, active_tab))
                        .unwrap_or(0);
                    (0..item_count)
                        .map(move |item_index| flat_row_variant(active_tab, page_index, item_index))
                })
                .collect()
        };
        self.filter_flat_rows();
    }

    fn filter_flat_rows(&mut self) {
        if self.issue_filter == IssueFilter::All {
            return;
        }
        self.flat_rows.retain(|row| {
            let page_index = match row {
                FlatRow::Image { page, .. }
                | FlatRow::Outlink { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. } => *page,
                FlatRow::OverviewIssue { .. } => return true,
            };
            let Some(page) = self.all_pages.get(page_index) else {
                return false;
            };
            flat_row_matches_filter(row, page, self.issue_filter)
        });
    }

    pub fn count_for_filter(&self, filter: IssueFilter) -> usize {
        let indices = filter_for_tab(
            self.active_tab,
            filter,
            &self.all_pages,
            &self.occurrence_counts,
        );
        if self.active_tab == ResultTab::Overview {
            return self.flat_rows.len();
        }
        if tab_is_flattened(self.active_tab) {
            if filter == IssueFilter::All {
                indices
                    .iter()
                    .map(|&page_ix| {
                        self.all_pages
                            .get(page_ix)
                            .map(|p| flat_row_item_count(p, self.active_tab))
                            .unwrap_or(0)
                    })
                    .sum::<usize>()
            } else {
                indices
                    .iter()
                    .map(|&page_ix| {
                        self.all_pages
                            .get(page_ix)
                            .map(|p| {
                                let item_count = flat_row_item_count(p, self.active_tab);
                                (0..item_count)
                                    .filter(|item| {
                                        let row = flat_row_variant(self.active_tab, page_ix, *item);
                                        flat_row_matches_filter(&row, p, filter)
                                    })
                                    .count()
                            })
                            .unwrap_or(0)
                    })
                    .sum::<usize>()
            }
        } else {
            indices.len()
        }
    }
}

fn columns_for_tab(tab: ResultTab) -> Vec<Column> {
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
            col("response_time", "Resp Time", 80., None),
            col("inlinks", "Inlinks", 70., None),
            col("outlinks_count", "Outlinks", 70., None),
            col("last_modified", "Last Modified", 130., None),
            col("redirect_url", "Redirect URI", 350., None),
            col("closest_similarity", "Closest Sim.", 90., None),
            col("near_duplicate_count", "Near Dups", 80., None),
        ],
        ResultTab::External => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("outlink_dst", "Destination", 380., None),
            col("outlink_anchor", "Anchor Text", 250., None),
            col("outlink_rel", "Rel", 120., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::ResponseCodes => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("content", "Content", 100., None),
            col("status_code", "Code", 70., None),
            col("status", "Status", 90., None),
            col("indexability", "Indexability", 110., None),
            col("redirect_url", "Redirect URI", 350., None),
            col("response_time", "Resp Time", 80., None),
        ],
        ResultTab::PageTitles => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("title", "Title", 350., None),
            col("title_length", "Title Len", 90., None),
            col("title_pixel_width", "Pixel Width", 90., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::MetaDesc => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("meta_desc", "Meta Desc", 350., None),
            col("meta_desc_length", "Meta Desc Len", 110., None),
            col("meta_desc_pixel_width", "Pixel Width", 90., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::H1 => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("h1", "H1", 300., None),
            col("h1_length", "H1 Len", 80., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::H2 => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("h2", "H2", 300., None),
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
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Images => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("image_src", "Src", 300., None),
            col("image_alt", "Alt Text", 200., None),
            col("image_width", "Width", 70., None),
            col("image_height", "Height", 70., None),
            col("image_has_alt", "Has Alt", 70., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Canonicals => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("canonical", "Canonical", 350., None),
            col("occurrences", "Occurrences", 100., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::Hreflang => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("hreflang_lang", "Language", 100., None),
            col("hreflang_url", "URL", 400., None),
            col("indexability", "Indexability", 110., None),
        ],
        ResultTab::StructuredData => vec![
            col("address", "Address", 380., Some(ColumnFixed::Left)),
            col("sd_format", "Format", 100., None),
            col("sd_type", "Type", 200., None),
            col("sd_raw", "JSON", 350., None),
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
            col("lcp", "LCP", 80., None),
            col("cls", "CLS", 80., None),
            col("inp", "INP", 80., None),
            col("response_time", "Resp Time", 80., None),
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
            col("issue", "Issue", 300., Some(ColumnFixed::Left)),
            col("count", "Count", 80., None),
        ],
    }
}

fn primary_field_key(tab: ResultTab) -> Option<&'static str> {
    match tab {
        ResultTab::PageTitles => Some("title"),
        ResultTab::MetaDesc => Some("meta_description"),
        ResultTab::H1 => Some("h1"),
        ResultTab::H2 => Some("h2"),
        ResultTab::Canonicals => Some("canonical"),
        _ => None,
    }
}

fn field_value<'a>(record: &'a PageRecord, field: &str) -> Option<&'a str> {
    match field {
        "title" => record.title.as_deref(),
        "meta_description" => record.meta_description.as_deref(),
        "h1" => record.h1.as_deref(),
        "h2" => record.h2.as_deref(),
        "canonical" => record.canonical.as_deref(),
        _ => None,
    }
}

fn field_count(record: &PageRecord, field: &str) -> u32 {
    match field {
        "title" => record.title_count,
        "h1" => record.h1_count,
        "h2" => record.h2_count,
        _ => 1,
    }
}

fn length_thresholds(tab: ResultTab) -> Option<(usize, usize)> {
    match tab {
        ResultTab::PageTitles => Some((30, 60)),
        ResultTab::MetaDesc => Some((50, 160)),
        ResultTab::H1 => Some((1, 70)),
        ResultTab::H2 => Some((1, 70)),
        _ => None,
    }
}

fn pixel_width_thresholds(tab: ResultTab) -> Option<(u32, u32)> {
    match tab {
        ResultTab::PageTitles => Some((200, 580)),
        ResultTab::MetaDesc => Some((200, 970)),
        _ => None,
    }
}

fn build_occurrence_counts(tab: ResultTab, pages: &[PageRecord]) -> HashMap<String, usize> {
    let Some(key) = primary_field_key(tab) else {
        return HashMap::new();
    };
    let mut counts: HashMap<String, usize> = HashMap::new();
    for page in pages {
        let val = field_value(page, key).unwrap_or("").to_string();
        *counts.entry(val).or_insert(0) += 1;
    }
    counts
}

fn filter_for_tab(
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
        | ResultTab::Directives => pages
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
                    IssueFilter::Multiple => count > 1,
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
            IssueFilter::AllGoodPerformance => indices.retain(|&idx| {
                let page = &pages[idx];
                page.lcp_ms.is_some_and(|ms| ms <= 2500)
                    && page.cls.is_some_and(|v| v <= 0.1)
                    && page.inp_ms.is_some_and(|ms| ms <= 200)
                    && page.ttfb_ms.is_some_and(|ms| ms <= 800)
            }),
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

fn flat_row_item_count(page: &PageRecord, tab: ResultTab) -> usize {
    match tab {
        ResultTab::Images => page.images.len(),
        ResultTab::External => page.outlinks.len(),
        ResultTab::Accessibility => page.a11y_issues.len(),
        ResultTab::Hreflang => page.hreflang_tags.len().max(1),
        ResultTab::StructuredData => page.sd_items.len().max(1),
        _ => 0,
    }
}

fn flat_row_variant(tab: ResultTab, page: usize, item: usize) -> FlatRow {
    match tab {
        ResultTab::Images => FlatRow::Image { page, item },
        ResultTab::External => FlatRow::Outlink { page, item },
        ResultTab::Accessibility => FlatRow::A11yIssue { page, item },
        ResultTab::Hreflang => FlatRow::Hreflang { page, item },
        ResultTab::StructuredData => FlatRow::SdItem { page, item },
        _ => FlatRow::Image { page, item },
    }
}

pub fn overview_issue_target(label: &str) -> Option<(ResultTab, IssueFilter)> {
    match label {
        "Missing Page Title" => Some((ResultTab::PageTitles, IssueFilter::Missing)),
        "Duplicate Page Title" => Some((ResultTab::PageTitles, IssueFilter::Duplicate)),
        "Missing Meta Description" => Some((ResultTab::MetaDesc, IssueFilter::Missing)),
        "Missing H1" => Some((ResultTab::H1, IssueFilter::Missing)),
        "Non-Indexable Pages" => Some((ResultTab::Internal, IssueFilter::NonIndexable)),
        "Missing Canonical" => Some((ResultTab::Canonicals, IssueFilter::MissingCanonical)),
        "Missing HTTPS" => Some((ResultTab::Security, IssueFilter::MissingHttps)),
        "Images Missing Alt" => Some((ResultTab::Images, IssueFilter::MissingAltText)),
        "Structured Data Errors" => Some((ResultTab::StructuredData, IssueFilter::SdErrors)),
        "Structured Data Warnings" => Some((ResultTab::StructuredData, IssueFilter::SdWarnings)),
        "Slow LCP" => Some((ResultTab::Performance, IssueFilter::SlowLcp)),
        "Slow CLS" => Some((ResultTab::Performance, IssueFilter::SlowCls)),
        "A11y Critical Issues" => Some((ResultTab::Accessibility, IssueFilter::All)),
        "A11y Warnings" => Some((ResultTab::Accessibility, IssueFilter::All)),
        "HTTP Errors (4xx/5xx)" => Some((ResultTab::ResponseCodes, IssueFilter::Status4xx)),
        "Near Duplicate Content" => Some((ResultTab::Content, IssueFilter::NearDuplicates)),
        "Low Content Pages" => Some((ResultTab::Content, IssueFilter::LowContent)),
        "Redirects" => Some((ResultTab::ResponseCodes, IssueFilter::Redirects)),
        "Missing HSTS" => Some((ResultTab::Security, IssueFilter::MissingHsts)),
        "Missing CSP" => Some((ResultTab::Security, IssueFilter::MissingCsp)),
        "Missing Frame Guard" => Some((ResultTab::Security, IssueFilter::MissingFrameGuard)),
        "Missing X-Content-Type" => {
            Some((ResultTab::Security, IssueFilter::MissingContentTypeOptions))
        }
        "Non-ASCII URLs" => Some((ResultTab::Url, IssueFilter::UrlNonAscii)),
        "Uppercase URLs" => Some((ResultTab::Url, IssueFilter::UrlUppercase)),
        "URLs with Underscores" => Some((ResultTab::Url, IssueFilter::UrlUnderscores)),
        "Long URLs" => Some((ResultTab::Url, IssueFilter::UrlOverLength)),
        _ => None,
    }
}

fn build_overview_rows(pages: &[PageRecord]) -> Vec<FlatRow> {
    let internal: Vec<&PageRecord> = pages.iter().filter(|p| p.is_internal).collect();
    let mut rows = Vec::new();

    let missing_title = internal
        .iter()
        .filter(|p| p.title.as_deref() == Some(""))
        .count();
    if missing_title > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing Page Title".into(),
            count: missing_title,
        });
    }

    let duplicate_title = {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for p in &internal {
            *counts.entry(p.title.as_deref().unwrap_or("")).or_insert(0) += 1;
        }
        internal
            .iter()
            .filter(|p| *counts.get(p.title.as_deref().unwrap_or("")).unwrap_or(&0) > 1)
            .count()
    };
    if duplicate_title > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Duplicate Page Title".into(),
            count: duplicate_title,
        });
    }

    let missing_desc = internal
        .iter()
        .filter(|p| p.meta_description.as_deref() == Some(""))
        .count();
    if missing_desc > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing Meta Description".into(),
            count: missing_desc,
        });
    }

    let missing_h1 = internal
        .iter()
        .filter(|p| p.h1.as_deref() == Some(""))
        .count();
    if missing_h1 > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing H1".into(),
            count: missing_h1,
        });
    }

    let non_indexable = internal
        .iter()
        .filter(|p| p.indexability.as_deref() == Some("Non-Indexable"))
        .count();
    if non_indexable > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Non-Indexable Pages".into(),
            count: non_indexable,
        });
    }

    let missing_canonical = internal
        .iter()
        .filter(|p| p.canonical.as_deref() == Some(""))
        .count();
    if missing_canonical > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing Canonical".into(),
            count: missing_canonical,
        });
    }

    let missing_https = internal
        .iter()
        .filter(|p| !p.url.starts_with("https://"))
        .count();
    if missing_https > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing HTTPS".into(),
            count: missing_https,
        });
    }

    let missing_alt = internal
        .iter()
        .flat_map(|p| p.images.iter())
        .filter(|img| !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty()))
        .count();
    if missing_alt > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Images Missing Alt".into(),
            count: missing_alt,
        });
    }

    let sd_errors = internal.iter().filter(|p| p.sd_errors > 0).count();
    if sd_errors > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Structured Data Errors".into(),
            count: sd_errors,
        });
    }

    let sd_warnings = internal.iter().filter(|p| p.sd_warnings > 0).count();
    if sd_warnings > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Structured Data Warnings".into(),
            count: sd_warnings,
        });
    }

    let slow_lcp = internal
        .iter()
        .filter(|p| p.lcp_ms.is_some_and(|ms| ms > 4000))
        .count();
    if slow_lcp > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Slow LCP".into(),
            count: slow_lcp,
        });
    }

    let slow_cls = internal
        .iter()
        .filter(|p| p.cls.is_some_and(|v| v > 0.25))
        .count();
    if slow_cls > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Slow CLS".into(),
            count: slow_cls,
        });
    }

    let a11y_critical = internal
        .iter()
        .flat_map(|p| p.a11y_issues.iter())
        .filter(|i| matches!(i.impact.as_str(), "critical" | "serious"))
        .count();
    if a11y_critical > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "A11y Critical Issues".into(),
            count: a11y_critical,
        });
    }

    let a11y_warnings = internal
        .iter()
        .flat_map(|p| p.a11y_issues.iter())
        .filter(|i| !matches!(i.impact.as_str(), "critical" | "serious"))
        .count();
    if a11y_warnings > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "A11y Warnings".into(),
            count: a11y_warnings,
        });
    }

    let status_errors = pages
        .iter()
        .filter(|p| p.status.is_some_and(|c| c >= 400))
        .count();
    if status_errors > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "HTTP Errors (4xx/5xx)".into(),
            count: status_errors,
        });
    }

    let near_dups = internal
        .iter()
        .filter(|p| p.near_duplicate_count.is_some_and(|c| c > 0))
        .count();
    if near_dups > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Near Duplicate Content".into(),
            count: near_dups,
        });
    }

    let low_content = internal
        .iter()
        .filter(|p| p.word_count.is_some_and(|w| w > 0 && w < 100))
        .count();
    if low_content > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Low Content Pages".into(),
            count: low_content,
        });
    }

    let redirects = pages.iter().filter(|p| p.redirect_url.is_some()).count();
    if redirects > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Redirects".into(),
            count: redirects,
        });
    }

    let missing_hsts = internal
        .iter()
        .filter(|p| !header_exists(&p.headers, "strict-transport-security"))
        .count();
    if missing_hsts > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing HSTS".into(),
            count: missing_hsts,
        });
    }

    let missing_csp = internal
        .iter()
        .filter(|p| !header_exists(&p.headers, "content-security-policy"))
        .count();
    if missing_csp > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing CSP".into(),
            count: missing_csp,
        });
    }

    let missing_frame_guard = internal
        .iter()
        .filter(|p| !header_exists(&p.headers, "x-frame-options"))
        .count();
    if missing_frame_guard > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing Frame Guard".into(),
            count: missing_frame_guard,
        });
    }

    let missing_content_type_opts = internal
        .iter()
        .filter(|p| !header_exists(&p.headers, "x-content-type-options"))
        .count();
    if missing_content_type_opts > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Missing X-Content-Type".into(),
            count: missing_content_type_opts,
        });
    }

    let url_non_ascii = internal.iter().filter(|p| !p.url.is_ascii()).count();
    if url_non_ascii > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Non-ASCII URLs".into(),
            count: url_non_ascii,
        });
    }

    let url_uppercase = internal
        .iter()
        .filter(|p| p.url.chars().any(|c| c.is_ascii_uppercase()))
        .count();
    if url_uppercase > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Uppercase URLs".into(),
            count: url_uppercase,
        });
    }

    let url_underscores = internal.iter().filter(|p| p.url.contains('_')).count();
    if url_underscores > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "URLs with Underscores".into(),
            count: url_underscores,
        });
    }

    let url_over_length = internal.iter().filter(|p| p.url.len() > 115).count();
    if url_over_length > 0 {
        rows.push(FlatRow::OverviewIssue {
            label: "Long URLs".into(),
            count: url_over_length,
        });
    }

    rows
}

fn flat_row_matches_filter(row: &FlatRow, page: &PageRecord, filter: IssueFilter) -> bool {
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
        FlatRow::Outlink { .. } | FlatRow::Hreflang { .. } | FlatRow::OverviewIssue { .. } => true,
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

fn header_exists(headers: &[(String, String)], name: &str) -> bool {
    headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case(name))
}

fn header_value<'a>(headers: &'a [(String, String)], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
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

fn compare_numeric(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parsed = a.parse::<f64>();
    let b_parsed = b.parse::<f64>();
    match (a_parsed, b_parsed) {
        (Ok(a_num), Ok(b_num)) => a_num
            .partial_cmp(&b_num)
            .unwrap_or(std::cmp::Ordering::Equal),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.cmp(b),
    }
}

fn is_tag_column(key: &str) -> bool {
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
            | "indexability_status"
            | "sec_https"
            | "sec_hsts"
            | "sec_csp"
            | "sec_frame_guard"
            | "sec_content_type_opts"
    )
}

fn is_mono_column(key: &str) -> bool {
    is_numeric_column(key)
        || is_tag_column(key)
        || matches!(
            key,
            "address"
                | "canonical"
                | "a11y_target"
                | "a11y_html"
                | "sd_raw"
                | "last_modified"
                | "redirect_url"
        )
}

fn is_numeric_column(key: &str) -> bool {
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
            | "response_time"
            | "inlinks"
            | "outlinks_count"
            | "closest_similarity"
            | "near_duplicate_count"
            | "occurrences"
            | "sd_errors"
            | "sd_warnings"
            | "ttfb"
            | "lcp"
            | "cls"
            | "inp"
            | "image_width"
            | "image_height"
            | "url_length"
    )
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

fn page_address(record: &PageRecord, root_origin: Option<&str>) -> SharedString {
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

fn flat_cell_text(
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
        FlatRow::OverviewIssue { .. } => SharedString::default(),
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

fn cell_text(
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

fn render_cell_tag(
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

impl TableDelegate for ResultsDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        if tab_is_flattened(self.active_tab) {
            self.flat_rows.len()
        } else {
            self.filtered_indices.len()
        }
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns.get(col_ix).cloned().unwrap_or_default()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let column = self.column(col_ix, cx);
        div()
            .size_full()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(column.name.clone())
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let key = self
            .columns
            .get(col_ix)
            .map(|c| c.key.clone())
            .unwrap_or_default();
        let mono = is_mono_column(&key);
        let mut cell = div().flex().items_center().text_xs();
        if mono {
            cell = cell.font_family(cx.theme().mono_font_family.clone());
        }

        if tab_is_flattened(self.active_tab) {
            let Some(row) = self.flat_rows.get(row_ix) else {
                return cell;
            };
            match row {
                FlatRow::OverviewIssue { label, count } => {
                    let text = match key.as_ref() {
                        "issue" => SharedString::from(label.clone()),
                        "count" => SharedString::from(count.to_string()),
                        _ => SharedString::default(),
                    };
                    if key.as_ref() == "count" {
                        let tone = if *count > 0 { Tone::Warn } else { Tone::Ok };
                        cell.child(tone_tag(tone).child(text))
                    } else {
                        cell.child(text)
                    }
                }
                _ => {
                    let page_index = match row {
                        FlatRow::Image { page, .. }
                        | FlatRow::Outlink { page, .. }
                        | FlatRow::A11yIssue { page, .. }
                        | FlatRow::Hreflang { page, .. }
                        | FlatRow::SdItem { page, .. } => *page,
                        FlatRow::OverviewIssue { .. } => unreachable!(),
                    };
                    let Some(record) = self.all_pages.get(page_index) else {
                        return cell;
                    };
                    let text = flat_cell_text(record, row, &key, self.root_origin.as_deref());
                    if let Some(tag) = render_cell_tag(record, &key, &text) {
                        cell.child(tag)
                    } else {
                        cell.child(text)
                    }
                }
            }
        } else {
            let Some(record) = self
                .filtered_indices
                .get(row_ix)
                .and_then(|&idx| self.all_pages.get(idx))
            else {
                return cell;
            };
            let text = cell_text(
                record,
                &key,
                &self.occurrence_counts,
                self.active_tab,
                self.root_origin.as_deref(),
            );
            if let Some(tag) = render_cell_tag(record, &key, &text) {
                cell.child(tag)
            } else {
                cell.child(text)
            }
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        let Some(col) = self.columns.get(col_ix) else {
            return;
        };
        let col_key = col.key.to_string();
        let numeric = is_numeric_column(&col_key);
        let root_origin = self.root_origin.clone();

        if tab_is_flattened(self.active_tab) {
            self.flat_rows.sort_by(|a, b| {
                if let (
                    FlatRow::OverviewIssue {
                        label: a_label,
                        count: a_count,
                    },
                    FlatRow::OverviewIssue {
                        label: b_label,
                        count: b_count,
                    },
                ) = (a, b)
                {
                    let ordering = match col_key.as_ref() {
                        "count" => a_count.cmp(b_count),
                        _ => a_label.cmp(b_label),
                    };
                    return match sort {
                        ColumnSort::Descending => ordering.reverse(),
                        _ => ordering,
                    };
                }
                let a_page = match a {
                    FlatRow::Image { page, .. }
                    | FlatRow::Outlink { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. } => *page,
                    FlatRow::OverviewIssue { .. } => 0,
                };
                let b_page = match b {
                    FlatRow::Image { page, .. }
                    | FlatRow::Outlink { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. } => *page,
                    FlatRow::OverviewIssue { .. } => 0,
                };
                let a_record = &self.all_pages[a_page];
                let b_record = &self.all_pages[b_page];
                let a_text = flat_cell_text(a_record, a, &col_key, root_origin.as_deref());
                let b_text = flat_cell_text(b_record, b, &col_key, root_origin.as_deref());

                let ordering = if numeric {
                    compare_numeric(&a_text, &b_text)
                } else {
                    a_text.cmp(&b_text)
                };

                match sort {
                    ColumnSort::Descending => ordering.reverse(),
                    _ => ordering,
                }
            });
        } else {
            self.filtered_indices.sort_by(|&a, &b| {
                let a_record = &self.all_pages[a];
                let b_record = &self.all_pages[b];
                let a_text = cell_text(
                    a_record,
                    &col_key,
                    &self.occurrence_counts,
                    self.active_tab,
                    self.root_origin.as_deref(),
                );
                let b_text = cell_text(
                    b_record,
                    &col_key,
                    &self.occurrence_counts,
                    self.active_tab,
                    self.root_origin.as_deref(),
                );

                let ordering = if numeric {
                    compare_numeric(&a_text, &b_text)
                } else {
                    a_text.cmp(&b_text)
                };

                match sort {
                    ColumnSort::Descending => ordering.reverse(),
                    _ => ordering,
                }
            });
        }
    }
}

pub struct ResultsGrid {
    state: Entity<TableState<ResultsDelegate>>,
    _subscription: Subscription,
}

impl ResultsGrid {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|cx| TableState::new(ResultsDelegate::new(), window, cx));
        let sub = cx.subscribe(&state, |this, _state, event: &TableEvent, cx| {
            if let TableEvent::SelectRow(row_ix) = event {
                let delegate = this.state.read(cx).delegate();
                if delegate.active_tab == ResultTab::Overview
                    && let Some(FlatRow::OverviewIssue { label, .. }) =
                        delegate.flat_rows().get(*row_ix)
                    && let Some((tab, filter)) = overview_issue_target(label)
                {
                    cx.emit(ResultsGridEvent::OverviewDrillDown { tab, filter });
                    return;
                }
                cx.emit(ResultsGridEvent::Selected(*row_ix))
            }
        });
        Self {
            state,
            _subscription: sub,
        }
    }

    pub fn push(&mut self, record: PageRecord, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().push(record);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().clear();
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab: ResultTab, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().switch_tab(tab);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn set_issue_filter(&mut self, filter: IssueFilter, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().set_issue_filter(filter);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn set_root_url(&mut self, root_url: &str, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().set_root_url(root_url);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn record_at(&self, index: usize, cx: &App) -> Option<PageRecord> {
        self.state.read(cx).delegate().record_at(index).cloned()
    }

    pub fn row_count(&self, cx: &App) -> usize {
        self.state.read(cx).delegate().filtered_count()
    }

    pub fn tab_counts(&self, cx: &App) -> HashMap<ResultTab, TabCounts> {
        self.state.read(cx).delegate().compute_tab_counts()
    }

    pub fn count_for_filter(&self, filter: IssueFilter, cx: &App) -> usize {
        self.state.read(cx).delegate().count_for_filter(filter)
    }

    #[allow(dead_code)]
    pub fn active_tab(&self, cx: &App) -> ResultTab {
        self.state.read(cx).delegate().active_tab()
    }
}

impl EventEmitter<ResultsGridEvent> for ResultsGrid {}

impl Render for ResultsGrid {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex_1()
            .size_full()
            .min_h_0()
            .bg(cx.theme().background)
            .child(DataTable::new(&self.state).bordered(false).stripe(true))
    }
}
