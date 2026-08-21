use crate::ui::tag::Tone;
use crate::views::ResultTab;

/// One unique image source and everything the Images tab knows about it.
///
/// The flags are "any reference": a logo referenced from 125 pages is one row,
/// and it belongs in `Missing Alt Text` if any of those 125 `img` tags lacks
/// alt text. Boxed inside `FlatRow` so the other row variants, of which a large
/// crawl holds far more, stay small.
#[derive(Clone, Debug)]
pub(crate) struct ImageAggregateRow {
    pub src: String,
    /// The first alt text seen, which is what the column shows. References
    /// disagreeing about the alt are visible in the details panel.
    pub alt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub missing_alt_attr: bool,
    pub missing_alt_text: bool,
    pub alt_over_100: bool,
    pub missing_size_attrs: bool,
    /// What the resource pass found when it requested the image: its status
    /// code and size in bytes. `None`/0 when the image was never checked, which
    /// is what a crawl run with resource checks off looks like.
    pub status: Option<u16>,
    pub size_bytes: u64,
    /// How many `img` tags across the crawl point at this source.
    pub reference_count: usize,
    /// The pages referencing it, in crawl order, as indices into `all_pages`.
    pub pages: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) enum FlatRow {
    ImageAggregate(Box<ImageAggregateRow>),
    A11yIssue {
        page: usize,
        item: usize,
    },
    SdItem {
        page: usize,
        item: usize,
    },
    IssuesRow {
        index: usize,
    },
    ChangeRow {
        index: usize,
    },
    DirectoryAggregate {
        path: String,
        depth: u32,
        page_count: usize,
        avg_word_count: u64,
        total_size: u64,
        non_indexable: usize,
        indexable: usize,
    },
}

pub(crate) fn tab_is_flattened(tab: ResultTab) -> bool {
    matches!(
        tab,
        ResultTab::Images
            | ResultTab::Accessibility
            | ResultTab::StructuredData
            | ResultTab::Overview
            | ResultTab::SiteStructure
            | ResultTab::Changes
    )
}

#[derive(Clone, Debug, Default)]
pub struct TabCounts {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
}

/// Per-tab counting result shared by the tab badge and the sub-filter buttons,
/// so a tab's badge is always a pure function of the same per-filter data that
/// renders its sub-filter counts. `filter_counts` is in `filters_for_tab`
/// order; `badge` is the aggregate shown on the tab.
#[derive(Clone, Debug, Default)]
pub struct TabFilterCounts {
    pub filter_counts: Vec<(IssueFilter, usize)>,
    pub badge: TabCounts,
}

/// The page index a flattened row belongs to, or `None` for rows that are not
/// tied to a single page (issue/change entries and directory aggregates).
pub(super) fn flat_row_page_index(row: &FlatRow) -> Option<usize> {
    match row {
        FlatRow::A11yIssue { page, .. } | FlatRow::SdItem { page, .. } => Some(*page),
        FlatRow::IssuesRow { .. }
        | FlatRow::ChangeRow { .. }
        | FlatRow::ImageAggregate(_)
        | FlatRow::DirectoryAggregate { .. } => None,
    }
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
    NonSequential,
    NonIndexable,
    Html,
    #[allow(dead_code)]
    Images,
    Css,
    JavaScript,
    Pdf,
    FetchXhr,
    OtherResource,
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
    HreflangMissingReturnTag,
    HreflangInvalidLang,
    HreflangMissingXDefault,
    HreflangMissingSelfReference,
    HreflangNonCanonical,
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
    ImageOver100Kb,
    ImageBroken,
    UrlsInSitemap,
    UrlsNotInSitemap,
    SitemapOrphans,
    NonIndexableInSitemap,
    SitemapNon200,
    IsProductPage,
    MissingPrice,
    MissingAvailability,
    MissingSku,
    MissingGtin,
    InvalidGtin,
    OutOfStockIndexable,
    MissingBreadcrumbs,
    IndexableParameterUrl,
    RedirectChain,
    CanonicalTargetNotIndexable,
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
    SsrContentMissing,
    BlockedByRobots,
    SlowLcp,
    SlowCls,
    SlowFcp,
    SlowTtfb,
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
    MissingReferrerPolicy,
    MixedContent,
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
    IssueTypeError,
    IssueTypeOpportunity,
    IssueTypeWarning,
    PriorityHigh,
    PriorityMedium,
    PriorityLow,
    LinkBroken,
    LinkRedirected,
    LinkNofollow,
    LinkNoAnchorText,
    LinkExternal,
    MissingBodyTag,
    DepthShallow,
    DepthMedium,
    DepthDeep,
    ChangeAdded,
    ChangeRemoved,
    ChangeChanged,
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
            IssueFilter::NonSequential => "Non-Sequential",
            IssueFilter::NonIndexable => "Non-Indexable",
            IssueFilter::Html => "HTML",
            IssueFilter::Images => "Images",
            IssueFilter::Css => "CSS",
            IssueFilter::JavaScript => "JavaScript",
            IssueFilter::Pdf => "PDF",
            IssueFilter::FetchXhr => "Fetch/XHR",
            IssueFilter::OtherResource => "Other",
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
            IssueFilter::HreflangMissingReturnTag => "Missing Return Tags",
            IssueFilter::HreflangInvalidLang => "Invalid Language Code",
            IssueFilter::HreflangMissingXDefault => "Missing x-default",
            IssueFilter::HreflangMissingSelfReference => "Missing Self Reference",
            IssueFilter::HreflangNonCanonical => "Non-Canonical Target",
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
            IssueFilter::ImageOver100Kb => "Over 100 kB",
            IssueFilter::ImageBroken => "Broken",
            IssueFilter::UrlsInSitemap => "In Sitemap",
            IssueFilter::UrlsNotInSitemap => "Not in Sitemap",
            IssueFilter::SitemapOrphans => "Orphan URLs",
            IssueFilter::NonIndexableInSitemap => "Non-Indexable in Sitemap",
            IssueFilter::SitemapNon200 => "Non-200 in Sitemap",
            IssueFilter::IsProductPage => "Product Pages",
            IssueFilter::MissingPrice => "Missing Price",
            IssueFilter::MissingAvailability => "Missing Availability",
            IssueFilter::MissingSku => "Missing SKU",
            IssueFilter::MissingGtin => "Missing GTIN",
            IssueFilter::InvalidGtin => "Invalid GTIN",
            IssueFilter::OutOfStockIndexable => "Out of Stock",
            IssueFilter::MissingBreadcrumbs => "Missing Breadcrumbs",
            IssueFilter::IndexableParameterUrl => "Indexable Parameter URLs",
            IssueFilter::RedirectChain => "Redirect Chain",
            IssueFilter::CanonicalTargetNotIndexable => "Canonical to Non-Indexable",
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
            IssueFilter::SsrContentMissing => "SSR Content Missing",
            IssueFilter::BlockedByRobots => "Blocked by robots.txt",
            IssueFilter::SlowLcp => "Slow LCP",
            IssueFilter::SlowCls => "Slow CLS",
            IssueFilter::SlowFcp => "Slow FCP",
            IssueFilter::SlowTtfb => "Slow TTFB",
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
            IssueFilter::MissingReferrerPolicy => "Missing Referrer-Policy",
            IssueFilter::MixedContent => "Mixed Content",
            IssueFilter::UrlNonAscii => "Non-ASCII",
            IssueFilter::UrlUppercase => "Uppercase",
            IssueFilter::UrlUnderscores => "Underscores",
            IssueFilter::UrlMultipleSlashes => "Multiple Slashes",
            IssueFilter::UrlParameters => "Parameters",
            IssueFilter::UrlOverLength => "Over 115 Chars",
            IssueFilter::UrlSpaces => "Contains Space",
            IssueFilter::MissingBodyTag => "Missing <body>",
            IssueFilter::DirectiveNoindex => "Noindex",
            IssueFilter::DirectiveNofollow => "Nofollow",
            IssueFilter::DirectiveNoarchive => "Noarchive",
            IssueFilter::DirectiveNosnippet => "Nosnippet",
            IssueFilter::DirectiveNone => "None",
            IssueFilter::Redirects => "Redirects",
            IssueFilter::RedirectLoop => "Redirect Loop",
            IssueFilter::OverPixelWidth => "Over Pixel Width",
            IssueFilter::UnderPixelWidth => "Under Pixel Width",
            IssueFilter::IssueTypeError => "Errors",
            IssueFilter::IssueTypeOpportunity => "Opportunities",
            IssueFilter::IssueTypeWarning => "Warnings",
            IssueFilter::PriorityHigh => "High Priority",
            IssueFilter::PriorityMedium => "Medium Priority",
            IssueFilter::PriorityLow => "Low Priority",
            IssueFilter::LinkBroken => "Broken (4xx/5xx)",
            IssueFilter::LinkRedirected => "Redirected (3xx)",
            IssueFilter::LinkNofollow => "Nofollow",
            IssueFilter::LinkNoAnchorText => "No Anchor Text",
            IssueFilter::LinkExternal => "External",
            IssueFilter::DepthShallow => "Depth 0-1",
            IssueFilter::DepthMedium => "Depth 2-3",
            IssueFilter::DepthDeep => "Depth 4+",
            IssueFilter::ChangeAdded => "Added",
            IssueFilter::ChangeRemoved => "Removed",
            IssueFilter::ChangeChanged => "Changed",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::All
            | Self::Html
            | Self::Images
            | Self::Css
            | Self::JavaScript
            | Self::Pdf
            | Self::FetchXhr
            | Self::OtherResource
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
            | Self::SdTypeArticle
            | Self::SdTypeProduct
            | Self::SdTypeFaq
            | Self::SdTypeHowTo
            | Self::SdTypeRecipe
            | Self::SdTypeVideo
            | Self::SdTypeBreadcrumb
            | Self::SdTypeOrganization
            | Self::DirectiveNone
            | Self::LinkExternal
            | Self::DepthShallow
            | Self::DepthMedium => Tone::Neutral,

            // Overview and Changes filters mirror the grid cells: delegate to the
            // same tone the rendered chips use so the tab badge, the sub-filter
            // button, and the cell can never disagree.
            Self::IssueTypeError => IssueType::Issue.tone(),
            Self::IssueTypeOpportunity => IssueType::Opportunity.tone(),
            Self::IssueTypeWarning => IssueType::Warning.tone(),
            Self::PriorityHigh => IssuePriority::High.tone(),
            Self::PriorityMedium => IssuePriority::Medium.tone(),
            Self::PriorityLow => IssuePriority::Low.tone(),
            Self::ChangeAdded => ChangeKind::Added.tone(),
            Self::ChangeRemoved => ChangeKind::Removed.tone(),
            Self::ChangeChanged => ChangeKind::Changed.tone(),

            Self::Status4xx
            | Self::Status5xx
            | Self::SdErrors
            | Self::ParseErrors
            | Self::SitemapOrphans
            | Self::SitemapNon200
            | Self::RedirectLoop
            | Self::RedirectChain
            | Self::InvalidGtin
            | Self::CanonicalTargetNotIndexable
            | Self::MissingHttps
            | Self::MixedContent
            | Self::ExactDuplicates
            | Self::SsrContentMissing
            | Self::BlockedByRobots
            | Self::DirectiveNoindex
            | Self::ImageBroken
            | Self::MissingBodyTag
            | Self::LinkBroken => Tone::Err,

            Self::NonIndexable
            | Self::Missing
            | Self::Duplicate
            | Self::OverLength
            | Self::UnderLength
            | Self::Multiple
            | Self::SameAsH1
            | Self::NonSequential
            | Self::Canonicalised
            | Self::MissingCanonical
            | Self::MissingHreflang
            | Self::HreflangMissingReturnTag
            | Self::HreflangInvalidLang
            | Self::HreflangMissingXDefault
            | Self::HreflangMissingSelfReference
            | Self::HreflangNonCanonical
            | Self::MissingStructuredData
            | Self::SdWarnings
            | Self::A11yImageAlt
            | Self::A11yLabel
            | Self::A11yLinkName
            | Self::A11yButtonName
            | Self::A11yColorContrast
            | Self::A11yHtmlHasLang
            | Self::A11yHeadingOrder
            | Self::MissingAltText
            | Self::MissingAltAttribute
            | Self::AltOver100
            | Self::MissingSizeAttributes
            | Self::ImageOver100Kb
            | Self::NonIndexableInSitemap
            | Self::UrlsNotInSitemap
            | Self::MissingPrice
            | Self::MissingAvailability
            | Self::MissingSku
            | Self::MissingGtin
            | Self::OutOfStockIndexable
            | Self::MissingBreadcrumbs
            | Self::IndexableParameterUrl
            | Self::MissingBrand
            | Self::MissingReviewRating
            | Self::MissingProductImage
            | Self::NearDuplicates
            | Self::LowContent
            | Self::SlowLcp
            | Self::SlowCls
            | Self::SlowFcp
            | Self::SlowTtfb
            | Self::Redirects
            | Self::MissingHsts
            | Self::MissingCsp
            | Self::MissingFrameGuard
            | Self::MissingContentTypeOptions
            | Self::MissingReferrerPolicy
            | Self::UrlNonAscii
            | Self::UrlUppercase
            | Self::UrlUnderscores
            | Self::UrlMultipleSlashes
            | Self::UrlParameters
            | Self::UrlOverLength
            | Self::UrlSpaces
            | Self::DirectiveNofollow
            | Self::DirectiveNoarchive
            | Self::DirectiveNosnippet
            | Self::OverPixelWidth
            | Self::UnderPixelWidth
            | Self::LinkRedirected
            | Self::LinkNofollow
            | Self::LinkNoAnchorText
            | Self::DepthDeep => Tone::Warn,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueType {
    Issue,
    Opportunity,
    Warning,
}

impl IssueType {
    pub fn label(self) -> &'static str {
        match self {
            Self::Issue => "Issue",
            Self::Opportunity => "Opportunity",
            Self::Warning => "Warning",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::Issue => Tone::Err,
            Self::Opportunity => Tone::Warn,
            Self::Warning => Tone::Neutral,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssuePriority {
    High,
    Medium,
    Low,
}

impl IssuePriority {
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::High => Tone::Err,
            Self::Medium => Tone::Warn,
            Self::Low => Tone::Neutral,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IssueEntry {
    pub name: String,
    pub issue_type: IssueType,
    pub priority: IssuePriority,
    pub count: usize,
    pub pct: f32,
    pub description: String,
    pub hint: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
}

impl ChangeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Removed => "Removed",
            Self::Changed => "Changed",
        }
    }

    pub fn tone(self) -> Tone {
        match self {
            Self::Added => Tone::Ok,
            Self::Removed => Tone::Err,
            Self::Changed => Tone::Warn,
        }
    }
}

/// A single per-URL difference between the loaded crawl and its baseline.
#[derive(Clone, Debug)]
pub struct ChangeEntry {
    pub url: String,
    pub kind: ChangeKind,
    pub status_before: Option<u16>,
    pub status_after: Option<u16>,
    pub changes: Vec<String>,
}

impl ChangeEntry {
    pub fn status_text(&self) -> String {
        let before = self
            .status_before
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into());
        let after = self
            .status_after
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".into());
        match self.kind {
            ChangeKind::Added => format!("→ {after}"),
            ChangeKind::Removed => format!("{before} →"),
            ChangeKind::Changed => {
                if self.status_before == self.status_after {
                    after
                } else {
                    format!("{before} → {after}")
                }
            }
        }
    }

    pub fn detail_text(&self) -> String {
        match self.kind {
            ChangeKind::Added => "New page".into(),
            ChangeKind::Removed => "Page removed".into(),
            ChangeKind::Changed => self.changes.join(", "),
        }
    }
}
