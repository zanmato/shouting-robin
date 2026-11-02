use gpui_kit::SharedString;

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

/// One layout template of the crawl and how its pages fared, for the Layout
/// tab. Boxed inside `FlatRow` like the image rows.
#[derive(Clone, Debug)]
pub(crate) struct LayoutTemplateRow {
    pub name: String,
    /// The structured data type naming the template, `None` for a template
    /// found by layout alone.
    pub type_key: Option<String>,
    /// Indices into `all_pages`, in crawl order.
    pub pages: Vec<usize>,
    /// Pages that joined by layout rather than by their structured data.
    pub joined_by_layout: usize,
    /// Pages whose layout fits a typed template while they lack its markup.
    pub missing_markup: Vec<usize>,
    /// Pages whose layout changed since the baseline crawl.
    pub changed: Vec<usize>,
    /// The layout change most of the changed pages share, and how many do.
    pub common_change: Option<(String, usize)>,
}

impl LayoutTemplateRow {
    /// The page the details panel shows for the row: a changed page when
    /// there is one, since that is what the row is most often opened for.
    pub fn example_page(&self) -> Option<usize> {
        self.changed
            .first()
            .or_else(|| self.missing_markup.first())
            .or_else(|| self.pages.first())
            .copied()
    }
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
    LayoutTemplate(Box<LayoutTemplateRow>),
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
            | ResultTab::Layout
    )
}

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TabCounts {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
}

/// Per-tab counting result shared by the tab badge and the sub-filter buttons,
/// so a tab's badge is always a pure function of the same per-filter data that
/// renders its sub-filter counts. `filter_counts` is in `filters_for_tab`
/// order; `badge` is the aggregate shown on the tab.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
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
        | FlatRow::LayoutTemplate(_)
        | FlatRow::DirectoryAggregate { .. } => None,
    }
}

#[derive(Clone, Debug)]
pub enum ResultsGridEvent {
    Selected(usize),
    OverviewDrillDown { tab: ResultTab, filter: IssueFilter },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IssueFilter {
    All,
    Missing,
    Duplicate,
    OverLength,
    UnderLength,
    Multiple,
    SameAsH1,
    HtmlEntity,
    RepeatsSiteName,
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
    ChangeLayout,
    LayoutMissingMarkup,
    LayoutChanged,
    ProductLayoutWithoutMarkup,
    RedirectTemporary,
    RedirectToError,
    LinkToHttp,
    SingleInlink,
    NoInternalOutlinks,
    CanonicalToRedirect,
    CanonicalToError,
    CanonicalToHttp,
    CanonicalChain,
    CanonicalisedInSitemap,
    SlowResponse,
    LargeHtml,
    /// An asset of the site Chrome fetched over the network more than once
    /// during the crawl: the server's caching headers sent the browser back
    /// for a file it already had.
    NotCached,
}

impl IssueFilter {
    pub fn label(self) -> SharedString {
        match self {
            IssueFilter::All => t!("results_grid.filter.all").into(),
            IssueFilter::Missing => t!("results_grid.filter.missing").into(),
            IssueFilter::Duplicate => t!("results_grid.filter.duplicate").into(),
            IssueFilter::OverLength => t!("results_grid.filter.over_length").into(),
            IssueFilter::UnderLength => t!("results_grid.filter.under_length").into(),
            IssueFilter::Multiple => t!("results_grid.filter.multiple").into(),
            IssueFilter::SameAsH1 => t!("results_grid.filter.same_as_h1").into(),
            IssueFilter::HtmlEntity => t!("results_grid.filter.html_entity").into(),
            IssueFilter::RepeatsSiteName => t!("results_grid.filter.repeats_site_name").into(),
            IssueFilter::NonSequential => t!("results_grid.filter.non_sequential").into(),
            IssueFilter::NonIndexable => t!("results_grid.filter.non_indexable").into(),
            IssueFilter::Html => "HTML".into(),
            IssueFilter::Images => t!("results_grid.filter.images").into(),
            IssueFilter::Css => "CSS".into(),
            IssueFilter::JavaScript => "JavaScript".into(),
            IssueFilter::Pdf => "PDF".into(),
            IssueFilter::FetchXhr => "Fetch/XHR".into(),
            IssueFilter::OtherResource => t!("results_grid.filter.other_resource").into(),
            IssueFilter::Status2xx => "2xx".into(),
            IssueFilter::Status3xx => "3xx".into(),
            IssueFilter::Status4xx => "4xx".into(),
            IssueFilter::Status5xx => "5xx".into(),
            IssueFilter::ContainsCanonical => t!("results_grid.filter.contains_canonical").into(),
            IssueFilter::SelfReferencing => t!("results_grid.filter.self_referencing").into(),
            IssueFilter::Canonicalised => t!("results_grid.filter.canonicalised").into(),
            IssueFilter::MissingCanonical => t!("results_grid.filter.missing_canonical").into(),
            IssueFilter::ContainsHreflang => t!("results_grid.filter.contains_hreflang").into(),
            IssueFilter::MissingHreflang => t!("results_grid.filter.missing_hreflang").into(),
            IssueFilter::HreflangMissingReturnTag => {
                t!("results_grid.filter.hreflang_missing_return_tag").into()
            }
            IssueFilter::HreflangInvalidLang => {
                t!("results_grid.filter.hreflang_invalid_lang").into()
            }
            IssueFilter::HreflangMissingXDefault => {
                t!("results_grid.filter.hreflang_missing_x_default").into()
            }
            IssueFilter::HreflangMissingSelfReference => {
                t!("results_grid.filter.hreflang_missing_self_reference").into()
            }
            IssueFilter::HreflangNonCanonical => {
                t!("results_grid.filter.hreflang_non_canonical").into()
            }
            IssueFilter::HasStructuredData => t!("results_grid.filter.has_structured_data").into(),
            IssueFilter::MissingStructuredData => {
                t!("results_grid.filter.missing_structured_data").into()
            }
            IssueFilter::SdErrors => t!("results_grid.filter.sd_errors").into(),
            IssueFilter::SdWarnings => t!("results_grid.filter.sd_warnings").into(),
            IssueFilter::JsonLdUrls => "JSON-LD".into(),
            IssueFilter::MicrodataUrls => t!("results_grid.filter.microdata_urls").into(),
            IssueFilter::ParseErrors => t!("results_grid.filter.parse_errors").into(),
            IssueFilter::MissingAltText => t!("results_grid.filter.missing_alt_text").into(),
            IssueFilter::MissingAltAttribute => {
                t!("results_grid.filter.missing_alt_attribute").into()
            }
            IssueFilter::AltOver100 => t!("results_grid.filter.alt_over_100").into(),
            IssueFilter::MissingSizeAttributes => {
                t!("results_grid.filter.missing_size_attributes").into()
            }
            IssueFilter::ImageOver100Kb => t!("results_grid.filter.image_over_100_kb").into(),
            IssueFilter::ImageBroken => t!("results_grid.filter.image_broken").into(),
            IssueFilter::UrlsInSitemap => t!("results_grid.filter.urls_in_sitemap").into(),
            IssueFilter::UrlsNotInSitemap => t!("results_grid.filter.urls_not_in_sitemap").into(),
            IssueFilter::SitemapOrphans => t!("results_grid.filter.sitemap_orphans").into(),
            IssueFilter::NonIndexableInSitemap => {
                t!("results_grid.filter.non_indexable_in_sitemap").into()
            }
            IssueFilter::SitemapNon200 => t!("results_grid.filter.sitemap_non_200").into(),
            IssueFilter::IsProductPage => t!("results_grid.filter.is_product_page").into(),
            IssueFilter::MissingPrice => t!("results_grid.filter.missing_price").into(),
            IssueFilter::MissingAvailability => {
                t!("results_grid.filter.missing_availability").into()
            }
            IssueFilter::MissingSku => t!("results_grid.filter.missing_sku").into(),
            IssueFilter::MissingGtin => t!("results_grid.filter.missing_gtin").into(),
            IssueFilter::InvalidGtin => t!("results_grid.filter.invalid_gtin").into(),
            IssueFilter::OutOfStockIndexable => {
                t!("results_grid.filter.out_of_stock_indexable").into()
            }
            IssueFilter::MissingBreadcrumbs => t!("results_grid.filter.missing_breadcrumbs").into(),
            IssueFilter::IndexableParameterUrl => {
                t!("results_grid.filter.indexable_parameter_url").into()
            }
            IssueFilter::RedirectChain => t!("results_grid.filter.redirect_chain").into(),
            IssueFilter::CanonicalTargetNotIndexable => {
                t!("results_grid.filter.canonical_target_not_indexable").into()
            }
            IssueFilter::MissingBrand => t!("results_grid.filter.missing_brand").into(),
            IssueFilter::MissingReviewRating => {
                t!("results_grid.filter.missing_review_rating").into()
            }
            IssueFilter::MissingProductImage => {
                t!("results_grid.filter.missing_product_image").into()
            }
            IssueFilter::A11yImageAlt => "image-alt".into(),
            IssueFilter::A11yLabel => "label".into(),
            IssueFilter::A11yLinkName => "link-name".into(),
            IssueFilter::A11yButtonName => "button-name".into(),
            IssueFilter::A11yColorContrast => "color-contrast".into(),
            IssueFilter::A11yHtmlHasLang => "html-has-lang".into(),
            IssueFilter::A11yHeadingOrder => "heading-order".into(),
            IssueFilter::ExactDuplicates => t!("results_grid.filter.exact_duplicates").into(),
            IssueFilter::NearDuplicates => t!("results_grid.filter.near_duplicates").into(),
            IssueFilter::LowContent => t!("results_grid.filter.low_content").into(),
            IssueFilter::SsrContentMissing => t!("results_grid.filter.ssr_content_missing").into(),
            IssueFilter::BlockedByRobots => t!("results_grid.filter.blocked_by_robots").into(),
            IssueFilter::SlowLcp => t!("results_grid.filter.slow_lcp").into(),
            IssueFilter::SlowCls => t!("results_grid.filter.slow_cls").into(),
            IssueFilter::SlowFcp => t!("results_grid.filter.slow_fcp").into(),
            IssueFilter::SlowTtfb => t!("results_grid.filter.slow_ttfb").into(),
            IssueFilter::SdTypeArticle => t!("results_grid.filter.sd_type_article").into(),
            IssueFilter::SdTypeProduct => t!("results_grid.filter.sd_type_product").into(),
            IssueFilter::SdTypeFaq => "FAQ".into(),
            IssueFilter::SdTypeHowTo => "HowTo".into(),
            IssueFilter::SdTypeRecipe => t!("results_grid.filter.sd_type_recipe").into(),
            IssueFilter::SdTypeVideo => t!("results_grid.filter.sd_type_video").into(),
            IssueFilter::SdTypeBreadcrumb => t!("results_grid.filter.sd_type_breadcrumb").into(),
            IssueFilter::SdTypeOrganization => {
                t!("results_grid.filter.sd_type_organization").into()
            }
            IssueFilter::MissingHttps => t!("results_grid.filter.missing_https").into(),
            IssueFilter::MissingHsts => t!("results_grid.filter.missing_hsts").into(),
            IssueFilter::MissingCsp => t!("results_grid.filter.missing_csp").into(),
            IssueFilter::MissingFrameGuard => t!("results_grid.filter.missing_frame_guard").into(),
            IssueFilter::MissingContentTypeOptions => {
                t!("results_grid.filter.missing_content_type_options").into()
            }
            IssueFilter::MissingReferrerPolicy => {
                t!("results_grid.filter.missing_referrer_policy").into()
            }
            IssueFilter::MixedContent => t!("results_grid.filter.mixed_content").into(),
            IssueFilter::UrlNonAscii => t!("results_grid.filter.url_non_ascii").into(),
            IssueFilter::UrlUppercase => t!("results_grid.filter.url_uppercase").into(),
            IssueFilter::UrlUnderscores => t!("results_grid.filter.url_underscores").into(),
            IssueFilter::UrlMultipleSlashes => {
                t!("results_grid.filter.url_multiple_slashes").into()
            }
            IssueFilter::UrlParameters => t!("results_grid.filter.url_parameters").into(),
            IssueFilter::UrlOverLength => t!("results_grid.filter.url_over_length").into(),
            IssueFilter::UrlSpaces => t!("results_grid.filter.url_spaces").into(),
            IssueFilter::MissingBodyTag => t!("results_grid.filter.missing_body_tag").into(),
            IssueFilter::DirectiveNoindex => "Noindex".into(),
            IssueFilter::DirectiveNofollow => "Nofollow".into(),
            IssueFilter::DirectiveNoarchive => "Noarchive".into(),
            IssueFilter::DirectiveNosnippet => "Nosnippet".into(),
            IssueFilter::DirectiveNone => t!("results_grid.filter.directive_none").into(),
            IssueFilter::Redirects => t!("results_grid.filter.redirects").into(),
            IssueFilter::RedirectLoop => t!("results_grid.filter.redirect_loop").into(),
            IssueFilter::OverPixelWidth => t!("results_grid.filter.over_pixel_width").into(),
            IssueFilter::UnderPixelWidth => t!("results_grid.filter.under_pixel_width").into(),
            IssueFilter::IssueTypeError => t!("results_grid.filter.issue_type_error").into(),
            IssueFilter::IssueTypeOpportunity => {
                t!("results_grid.filter.issue_type_opportunity").into()
            }
            IssueFilter::IssueTypeWarning => t!("results_grid.filter.issue_type_warning").into(),
            IssueFilter::PriorityHigh => t!("results_grid.filter.priority_high").into(),
            IssueFilter::PriorityMedium => t!("results_grid.filter.priority_medium").into(),
            IssueFilter::PriorityLow => t!("results_grid.filter.priority_low").into(),
            IssueFilter::LinkBroken => t!("results_grid.filter.link_broken").into(),
            IssueFilter::LinkRedirected => t!("results_grid.filter.link_redirected").into(),
            IssueFilter::LinkNofollow => "Nofollow".into(),
            IssueFilter::LinkNoAnchorText => t!("results_grid.filter.link_no_anchor_text").into(),
            IssueFilter::LinkExternal => t!("results_grid.filter.link_external").into(),
            IssueFilter::DepthShallow => t!("results_grid.filter.depth_shallow").into(),
            IssueFilter::DepthMedium => t!("results_grid.filter.depth_medium").into(),
            IssueFilter::DepthDeep => t!("results_grid.filter.depth_deep").into(),
            IssueFilter::ChangeAdded => t!("results_grid.filter.change_added").into(),
            IssueFilter::ChangeRemoved => t!("results_grid.filter.change_removed").into(),
            IssueFilter::ChangeChanged => t!("results_grid.filter.change_changed").into(),
            IssueFilter::ChangeLayout => t!("results_grid.filter.change_layout").into(),
            IssueFilter::LayoutMissingMarkup => {
                t!("results_grid.filter.layout_missing_markup").into()
            }
            IssueFilter::LayoutChanged => t!("results_grid.filter.layout_changed").into(),
            IssueFilter::ProductLayoutWithoutMarkup => {
                t!("results_grid.filter.product_layout_without_markup").into()
            }
            IssueFilter::RedirectTemporary => t!("results_grid.filter.redirect_temporary").into(),
            IssueFilter::RedirectToError => t!("results_grid.filter.redirect_to_error").into(),
            IssueFilter::LinkToHttp => t!("results_grid.filter.link_to_http").into(),
            IssueFilter::SingleInlink => t!("results_grid.filter.single_inlink").into(),
            IssueFilter::NoInternalOutlinks => {
                t!("results_grid.filter.no_internal_outlinks").into()
            }
            IssueFilter::CanonicalToRedirect => {
                t!("results_grid.filter.canonical_to_redirect").into()
            }
            IssueFilter::CanonicalToError => t!("results_grid.filter.canonical_to_error").into(),
            IssueFilter::CanonicalToHttp => t!("results_grid.filter.canonical_to_http").into(),
            IssueFilter::CanonicalChain => t!("results_grid.filter.canonical_chain").into(),
            IssueFilter::CanonicalisedInSitemap => {
                t!("results_grid.filter.canonicalised_in_sitemap").into()
            }
            IssueFilter::SlowResponse => t!("results_grid.filter.slow_response").into(),
            IssueFilter::LargeHtml => t!("results_grid.filter.large_html").into(),
            IssueFilter::NotCached => t!("results_grid.filter.not_cached").into(),
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
            Self::ChangeChanged | Self::ChangeLayout | Self::LayoutChanged => {
                ChangeKind::Changed.tone()
            }

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
            | Self::LinkBroken
            | Self::RedirectToError
            | Self::CanonicalToError => Tone::Err,

            Self::NonIndexable
            | Self::LayoutMissingMarkup
            | Self::Missing
            | Self::Duplicate
            | Self::OverLength
            | Self::UnderLength
            | Self::Multiple
            | Self::SameAsH1
            | Self::HtmlEntity
            | Self::RepeatsSiteName
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
            | Self::ProductLayoutWithoutMarkup
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
            | Self::DepthDeep
            | Self::RedirectTemporary
            | Self::LinkToHttp
            | Self::SingleInlink
            | Self::NoInternalOutlinks
            | Self::CanonicalToRedirect
            | Self::CanonicalToHttp
            | Self::CanonicalChain
            | Self::CanonicalisedInSitemap
            | Self::SlowResponse
            | Self::LargeHtml
            | Self::NotCached => Tone::Warn,
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum IssueType {
    Issue,
    Opportunity,
    Warning,
}

impl IssueType {
    pub fn label(self) -> SharedString {
        match self {
            Self::Issue => t!("results_grid.issue_type.issue").into(),
            Self::Opportunity => t!("results_grid.issue_type.opportunity").into(),
            Self::Warning => t!("results_grid.issue_type.warning").into(),
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

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum IssuePriority {
    High,
    Medium,
    Low,
}

impl IssuePriority {
    pub fn label(self) -> SharedString {
        match self {
            Self::High => t!("results_grid.priority.high").into(),
            Self::Medium => t!("results_grid.priority.medium").into(),
            Self::Low => t!("results_grid.priority.low").into(),
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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
    pub fn label(self) -> SharedString {
        match self {
            Self::Added => t!("results_grid.change_kind.added").into(),
            Self::Removed => t!("results_grid.change_kind.removed").into(),
            Self::Changed => t!("results_grid.change_kind.changed").into(),
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
    /// Set when both crawls captured the page and its layout changed.
    pub layout: Option<crate::crawl::layout::PageLayoutChange>,
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
            ChangeKind::Added => t!("results_grid.change.new_page").into(),
            ChangeKind::Removed => t!("results_grid.change.page_removed").into(),
            ChangeKind::Changed => self.changes.join(", "),
        }
    }
}

/// Wraps records the way the grid stores them, for tests that build their own.
#[cfg(test)]
pub(super) fn shared(
    pages: impl IntoIterator<Item = crate::crawl::event::PageRecord>,
) -> Vec<std::sync::Arc<crate::crawl::event::PageRecord>> {
    pages.into_iter().map(std::sync::Arc::new).collect()
}
