use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct PageRecord {
    pub url: String,
    pub status: Option<u16>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub response_time: Duration,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub h1: Option<String>,
    pub h2: Option<String>,
    pub canonical: Option<String>,
    pub robots: Option<String>,
    pub word_count: Option<u32>,
    pub depth: u32,
    pub is_internal: bool,
    /// True when this row came from spider's page callback, i.e. a navigated,
    /// parsed document. Document-derived tabs (Page Titles, Meta Desc, H1, ...)
    /// filter on this flag rather than on `content_type`, because spider can
    /// report a misleading `Content-Type` for the document (e.g. SPAs serving
    /// `application/javascript` for the request it actually issued).
    pub is_page: bool,
    /// True when this row is a subresource (CSS/JS/image/font/XHR) harvested
    /// from the page's resource timings rather than a navigated, parsed
    /// document. Its `content_type` is guessed from the URL/initiator and is
    /// unreliable, so document-derived tabs filter on this flag, not the type.
    pub is_resource: bool,
    /// The Resource Timing API `initiatorType` for harvested resources (e.g.
    /// "script", "css", "img", "fetch", "xmlhttprequest"). `None` for
    /// navigated documents. Used to single out Fetch/XHR requests, which have
    /// no usable content type.
    pub resource_initiator: Option<String>,
    pub indexability: Option<String>,
    pub h1_count: u32,
    pub h2_count: u32,
    pub title_count: u32,
    pub hreflang_tags: Vec<(String, String)>,
    pub sd_types: Vec<String>,
    pub sd_errors: u32,
    pub sd_warnings: u32,
    pub ttfb_ms: Option<u64>,
    pub lcp_ms: Option<u64>,
    pub cls: Option<f64>,
    pub fcp_ms: Option<u64>,
    pub sd_jsonld_count: u32,
    pub sd_microdata_count: u32,
    pub sd_items: Vec<SdItem>,
    pub sd_issues: Vec<SdIssue>,
    pub images: Vec<ImageRef>,
    pub content_hash: Option<String>,
    pub simhash: Option<u64>,
    pub closest_similarity: Option<u8>,
    pub near_duplicate_count: Option<u32>,
    pub near_duplicate_urls: Vec<String>,
    pub in_sitemap: Option<bool>,
    pub sitemap_url: Option<String>,
    pub og_type: Option<String>,
    pub ecommerce: Option<EcommerceAudit>,
    pub outlinks: Vec<Outlink>,
    pub inlinks_count: u32,
    /// Number of distinct source URLs linking here. A page linking to this one
    /// three times counts as three inlinks but one unique inlink.
    pub unique_inlinks_count: u32,
    pub csr_inlinks_count: u32,
    pub a11y_errors: u32,
    pub a11y_warnings: u32,
    pub a11y_issues: Vec<A11yIssue>,
    pub headers: Vec<(String, String)>,
    pub redirect_url: Option<String>,
    pub redirect_status: Option<u16>,
    pub link_score: Option<f32>,
    pub backlinks: Vec<Backlink>,
    pub title_2: Option<String>,
    pub meta_description_2: Option<String>,
    pub h1_2: Option<String>,
    pub h2_2: Option<String>,
    pub title_pixel_width: Option<u32>,
    pub meta_description_pixel_width: Option<u32>,
    pub hreflang_issues: Vec<HreflangIssue>,
    pub ssr_word_count: Option<u32>,
    pub ssr_h1: Option<String>,
    pub ssr_content_missing: Option<bool>,
    pub blocked_by_robots: Option<bool>,
    /// True when an HTTPS page loads at least one subresource (script, style,
    /// image, iframe, media, …) over plain HTTP. Browsers block or warn on such
    /// mixed content, so it's a security issue worth flagging.
    pub has_mixed_content: bool,
}

#[derive(Debug, Clone)]
pub struct SdItem {
    pub format: SdFormat,
    pub type_name: String,
    pub raw_json: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdFormat {
    JsonLd,
    Microdata,
}

#[derive(Debug, Clone)]
pub struct SdIssue {
    pub severity: SdSeverity,
    pub type_name: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SdSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HreflangIssue {
    MissingReturnTag { lang: String, target_url: String },
    InvalidLanguageCode { code: String },
    MissingXDefault,
    NonCanonicalUrl { hreflang_url: String },
}

#[derive(Debug, Clone)]
pub struct ImageRef {
    pub src: String,
    pub alt: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub has_alt_attr: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EcommerceAudit {
    pub price: Option<String>,
    pub currency: Option<String>,
    pub availability: Option<String>,
    pub sku: Option<String>,
    pub gtin: Option<String>,
    pub brand: Option<String>,
    pub has_image: bool,
    pub has_description: bool,
    pub has_review_or_rating: bool,
}

#[derive(Debug, Clone)]
pub struct Outlink {
    pub dst_url: String,
    pub anchor: Option<String>,
    pub rel: Option<String>,
    pub csr_only: bool,
}

#[derive(Debug, Clone)]
pub struct A11yIssue {
    pub rule: String,
    pub impact: String,
    pub target: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Backlink {
    pub source_url: String,
    pub anchor: Option<String>,
    pub rel: Option<String>,
}

impl PageRecord {
    pub fn compute_indexability(&mut self) {
        self.indexability = Some(match self.status {
            Some(code) if (200..300).contains(&code) => {
                if self
                    .robots
                    .as_deref()
                    .map(|r| r.to_ascii_lowercase().contains("noindex"))
                    .unwrap_or(false)
                {
                    "Non-Indexable".to_string()
                } else {
                    "Indexable".to_string()
                }
            }
            Some(_) => "Non-Indexable".to_string(),
            None => "N/A".to_string(),
        });
    }
}

#[derive(Debug, Clone)]
pub enum CrawlEvent {
    Started { crawl_id: i64, root_url: String },
    Page(Box<PageRecord>),
    Progress { crawled: u64, queued: u64 },
    Finished { crawl_id: i64, total: u64 },
    Error { url: String, message: String },
}
