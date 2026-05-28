use crate::ui::tag::Tone;
use crate::views::ResultTab;

#[derive(Clone, Debug)]
pub(crate) enum FlatRow {
    Image {
        page: usize,
        item: usize,
    },
    Outlink {
        page: usize,
        item: usize,
    },
    A11yIssue {
        page: usize,
        item: usize,
    },
    Hreflang {
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
    LinkRow {
        page: usize,
        item: usize,
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
            | ResultTab::External
            | ResultTab::Accessibility
            | ResultTab::Hreflang
            | ResultTab::StructuredData
            | ResultTab::Overview
            | ResultTab::Links
            | ResultTab::SiteStructure
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
    Css,
    JavaScript,
    Pdf,
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
    IssueTypeError,
    IssueTypeOpportunity,
    IssueTypeWarning,
    PriorityHigh,
    PriorityMedium,
    PriorityLow,
    LinkBroken,
    LinkRedirected,
    LinkNofollow,
    LinkExternal,
    DepthShallow,
    DepthMedium,
    DepthDeep,
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
            IssueFilter::Css => "CSS",
            IssueFilter::JavaScript => "JavaScript",
            IssueFilter::Pdf => "PDF",
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
            IssueFilter::IssueTypeError => "Errors",
            IssueFilter::IssueTypeOpportunity => "Opportunities",
            IssueFilter::IssueTypeWarning => "Warnings",
            IssueFilter::PriorityHigh => "High Priority",
            IssueFilter::PriorityMedium => "Medium Priority",
            IssueFilter::PriorityLow => "Low Priority",
            IssueFilter::LinkBroken => "Broken (4xx/5xx)",
            IssueFilter::LinkRedirected => "Redirected (3xx)",
            IssueFilter::LinkNofollow => "Nofollow",
            IssueFilter::LinkExternal => "External",
            IssueFilter::DepthShallow => "Depth 0-1",
            IssueFilter::DepthMedium => "Depth 2-3",
            IssueFilter::DepthDeep => "Depth 4+",
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
            | Self::AllGoodPerformance
            | Self::SdTypeArticle
            | Self::SdTypeProduct
            | Self::SdTypeFaq
            | Self::SdTypeHowTo
            | Self::SdTypeRecipe
            | Self::SdTypeVideo
            | Self::SdTypeBreadcrumb
            | Self::SdTypeOrganization
            | Self::DirectiveNone
            | Self::IssueTypeWarning
            | Self::PriorityLow
            | Self::LinkExternal
            | Self::DepthShallow
            | Self::DepthMedium => Tone::Neutral,

            Self::Status4xx
            | Self::Status5xx
            | Self::SdErrors
            | Self::ParseErrors
            | Self::SitemapOrphans
            | Self::RedirectLoop
            | Self::MissingHttps
            | Self::ExactDuplicates
            | Self::DirectiveNoindex
            | Self::LinkBroken
            | Self::IssueTypeError
            | Self::PriorityHigh => Tone::Err,

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
            | Self::HreflangMissingReturnTag
            | Self::HreflangInvalidLang
            | Self::HreflangMissingXDefault
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
            | Self::IssueTypeOpportunity
            | Self::PriorityMedium
            | Self::LinkRedirected
            | Self::LinkNofollow
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
