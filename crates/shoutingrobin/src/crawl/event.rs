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
    /// Click depth from the start page. `None` for URLs the link graph never
    /// reaches, such as sitemap-only orphans and robots.txt-blocked URLs, which
    /// are not zero clicks away from the start page but an unknown number.
    pub depth: Option<u32>,
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
    /// Whether an H2 opens the document before its first H1. `None` for a row
    /// with no parsed document to read an outline from: a subresource, or a URL
    /// nothing was fetched for.
    pub h2_non_sequential: Option<bool>,
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
    /// Where this page's hreflang tags were found, in the order the sources are
    /// consulted. Search engines accept all three and treat them as one set, so
    /// the tags themselves are merged; this records which sources contributed.
    pub hreflang_sources: Vec<HreflangSource>,
    pub in_sitemap: Option<bool>,
    pub sitemap_url: Option<String>,
    /// The `<lastmod>` the sitemap claims for this URL, verbatim. Sitemaps are
    /// free to give a date or a full timestamp, so it is not parsed.
    pub sitemap_lastmod: Option<String>,
    pub og_type: Option<String>,
    pub ecommerce: Option<EcommerceAudit>,
    pub outlinks: Vec<Outlink>,
    /// Stylesheet and script URLs the page pulls in, absolute and resolved.
    ///
    /// Not persisted: these exist so the post-crawl resource pass knows what to
    /// status-check, and the resources it finds are recorded as rows of their
    /// own. A record loaded back from storage carries an empty list.
    pub subresources: Vec<Subresource>,
    pub inlinks_count: u32,
    /// Number of distinct source URLs linking here. A page linking to this one
    /// three times counts as three inlinks but one unique inlink.
    pub unique_inlinks_count: u32,
    pub csr_inlinks_count: u32,
    /// Pages linking here only after rendering. The count above is links; this
    /// is how many distinct pages they come from.
    pub unique_csr_inlinks_count: u32,
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
    /// Whether the markup as served carries a `<body>` start tag. `None` for a
    /// row that is not a parsed document: a subresource, or a URL nothing was
    /// fetched for, neither of which has markup to be missing anything.
    pub has_body_tag: Option<bool>,
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
    MissingReturnTag {
        lang: String,
        target_url: String,
    },
    InvalidLanguageCode {
        code: String,
    },
    MissingXDefault,
    /// The page's own hreflang set doesn't list the page itself. Search engines
    /// treat a cluster whose members don't each point at themselves as
    /// incomplete and may ignore the whole set.
    MissingSelfReference,
    NonCanonicalUrl {
        hreflang_url: String,
    },
}

/// Where an hreflang annotation was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HreflangSource {
    Html,
    HttpHeader,
    Sitemap,
}

impl HreflangSource {
    pub fn label(self) -> &'static str {
        match self {
            Self::Html => "HTML",
            Self::HttpHeader => "HTTP",
            Self::Sitemap => "Sitemap",
        }
    }
}

/// A non-anchor resource a page pulls in, with what pulled it in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subresource {
    pub url: String,
    pub kind: SubresourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubresourceKind {
    Stylesheet,
    Script,
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

/// The lowercase directive tokens of one robots value, with any leading
/// crawler name (`googlebot: noindex, nofollow`) removed.
pub fn robots_directives(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = match value.split_once(':') {
        Some((agent, rest))
            if !agent.trim().is_empty()
                && agent
                    .trim()
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '*') =>
        {
            rest
        }
        _ => value,
    };
    value
        .split(',')
        .map(|token| token.trim().to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

/// Availability values that mean the product cannot be bought right now.
pub fn availability_is_out_of_stock(availability: &str) -> bool {
    matches!(
        availability.trim().to_ascii_lowercase().as_str(),
        "outofstock" | "soldout" | "discontinued"
    )
}

/// True when `gtin` is a GTIN-8/12/13/14 (or ISBN-13) whose check digit is
/// right. A GTIN that fails this is a typo, not an identifier, and Merchant
/// Center rejects the product for it.
pub fn gtin_checksum_valid(gtin: &str) -> bool {
    let digits: Vec<u32> = gtin
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_digit(10))
        .collect::<Option<Vec<u32>>>()
        .unwrap_or_default();
    if !matches!(digits.len(), 8 | 12 | 13 | 14) {
        return false;
    }
    let Some((&check, payload)) = digits.split_last() else {
        return false;
    };
    // Weights alternate 3,1,3,1... counted from the digit nearest the check.
    let sum: u32 = payload
        .iter()
        .rev()
        .enumerate()
        .map(|(position, digit)| if position % 2 == 0 { digit * 3 } else { *digit })
        .sum();
    (10 - sum % 10) % 10 == check
}

impl PageRecord {
    /// True when nothing keeps this page out of the index: a 2xx document
    /// that is neither a redirect, noindex, nor canonicalised elsewhere.
    pub fn is_indexable(&self) -> bool {
        self.status
            .is_some_and(|status| (200..300).contains(&status))
            && !self.is_redirect()
            && !self.is_noindex()
            && !self.is_canonicalised()
    }

    /// True when the body is (or may be) an HTML document worth auditing for
    /// titles, headings and content. An unknown content type counts as HTML,
    /// because a server that omits the header is still most likely serving a
    /// page; a declared PDF, XML feed or JSON endpoint is not.
    pub fn is_html_document(&self) -> bool {
        match self.content_type.as_deref() {
            None => true,
            Some(content_type) => {
                let media_type = content_type
                    .split(';')
                    .next()
                    .unwrap_or(content_type)
                    .trim()
                    .to_ascii_lowercase();
                media_type.is_empty()
                    || media_type == "text/html"
                    || media_type == "application/xhtml+xml"
            }
        }
    }

    /// True when the page carries a `noindex` directive, from either the robots
    /// meta tag or the `X-Robots-Tag` response header. Search engines honour
    /// both, so a page is out of the index either way.
    pub fn is_noindex(&self) -> bool {
        self.has_robots_directive("noindex")
    }

    /// True when `directive` (`noindex`, `nofollow`, `noarchive`, ...) is set
    /// by the robots meta tag or by any `X-Robots-Tag` header. Servers may send
    /// several such headers, and each may be scoped to a crawler
    /// (`googlebot: noindex`); `none` stands for `noindex, nofollow`.
    pub fn has_robots_directive(&self, directive: &str) -> bool {
        let header_values = self
            .headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case("x-robots-tag"))
            .map(|(_, value)| value.as_str());
        self.robots
            .as_deref()
            .into_iter()
            .chain(header_values)
            .flat_map(robots_directives)
            .any(|token| {
                token == directive
                    || (token == "none" && matches!(directive, "noindex" | "nofollow"))
            })
    }

    /// True when the page declares a canonical pointing at a *different* URL,
    /// i.e. it asks search engines to index that other page instead of this one.
    ///
    /// The href is resolved against the page URL and both sides are normalised
    /// first. A relative canonical (`/a`) and a canonical written without the
    /// trailing slash (`https://example.com` for the page served at
    /// `https://example.com/`) are both self-referencing, but a raw string
    /// comparison reports every such page as canonicalised elsewhere.
    pub fn is_canonicalised(&self) -> bool {
        self.canonical.as_deref().is_some_and(|canonical| {
            if canonical.trim().is_empty() {
                return false;
            }
            let resolved = crate::crawl::url_norm::resolve_url(&self.url, canonical)
                .unwrap_or_else(|| canonical.to_string());
            !crate::crawl::url_norm::urls_equivalent(&resolved, &self.url)
        })
    }

    /// True when this row is a redirect rather than a document of its own.
    ///
    /// Both signals are needed. A followed redirect can surface as the target's
    /// 2xx with the final URL recorded, and a redirect the crawler didn't follow
    /// surfaces as a 3xx with no final URL at all: on the crawled site two of
    /// three redirects came back that way, and keying only on `redirect_url`
    /// counted one redirect instead of three and let the other two be audited as
    /// ordinary pages, reporting a missing title, H1 and description for a
    /// response that has no body to carry them.
    pub fn is_redirect(&self) -> bool {
        self.redirect_url.is_some() || self.status.is_some_and(|c| (300..400).contains(&c))
    }

    /// Why the page is or isn't eligible for the index, as the `Index. Status`
    /// column shows it: `Indexable`, or a comma-separated list of every reason
    /// it isn't. A page can be excluded for more than one reason at once (a
    /// canonicalised page that also carries `noindex`), and reporting only the
    /// first would hide the rest.
    ///
    /// `indexability` is derived from this, so the two columns can never
    /// disagree the way they did when each computed its own answer.
    pub fn indexability_status(&self) -> String {
        let Some(status) = self.status else {
            return "N/A".to_string();
        };
        let mut reasons: Vec<String> = Vec::new();
        if self.is_redirect() {
            reasons.push("Redirected".to_string());
        } else if !(200..300).contains(&status) {
            reasons.push(format!("Non-Indexable ({status})"));
        }
        if self.is_noindex() {
            reasons.push("Noindex".to_string());
        }
        // A redirect response has no document, so any canonical on this row
        // belongs to the target and says nothing about this URL.
        if !self.is_redirect() && self.is_canonicalised() {
            reasons.push("Canonicalised".to_string());
        }
        if reasons.is_empty() {
            "Indexable".to_string()
        } else {
            reasons.join(", ")
        }
    }

    pub fn compute_indexability(&mut self) {
        self.indexability = Some(match self.indexability_status().as_str() {
            "Indexable" => "Indexable".to_string(),
            "N/A" => "N/A".to_string(),
            _ => "Non-Indexable".to_string(),
        });
    }
}

#[derive(Debug, Clone)]
pub enum CrawlEvent {
    Started { crawl_id: i64, root_url: String },
    Page(Box<PageRecord>),
    Finished { crawl_id: i64, total: u64 },
    Error { url: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(url: &str) -> PageRecord {
        PageRecord {
            url: url.to_string(),
            status: Some(200),
            ..Default::default()
        }
    }

    fn status_of(record: &mut PageRecord) -> (String, String) {
        record.compute_indexability();
        (
            record.indexability_status(),
            record.indexability.clone().unwrap_or_default(),
        )
    }

    #[test]
    fn a_plain_page_is_indexable() {
        let mut record = page("https://example.com/a");
        assert_eq!(
            status_of(&mut record),
            ("Indexable".into(), "Indexable".into())
        );
    }

    #[test]
    fn a_canonical_without_the_trailing_slash_is_self_referencing() {
        // The live false positive: the home page's canonical omits the slash.
        let mut record = page("https://example.com/");
        record.canonical = Some("https://example.com".into());
        assert!(!record.is_canonicalised());
        assert_eq!(
            status_of(&mut record),
            ("Indexable".into(), "Indexable".into())
        );
    }

    #[test]
    fn a_relative_canonical_is_resolved_before_comparison() {
        let mut record = page("https://example.com/a");
        record.canonical = Some("/a".into());
        assert!(!record.is_canonicalised());
        assert_eq!(record.indexability_status(), "Indexable");
    }

    #[test]
    fn a_canonical_pointing_elsewhere_is_non_indexable() {
        let mut record = page("https://example.com/a");
        record.canonical = Some("https://example.com/b".into());
        assert!(record.is_canonicalised());
        assert_eq!(
            status_of(&mut record),
            ("Canonicalised".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn an_empty_canonical_is_not_canonicalisation() {
        let mut record = page("https://example.com/a");
        record.canonical = Some("  ".into());
        assert!(!record.is_canonicalised());
        assert_eq!(record.indexability_status(), "Indexable");
    }

    #[test]
    fn every_exclusion_reason_is_reported_at_once() {
        let mut record = page("https://example.com/a");
        record.robots = Some("noindex, follow".into());
        record.canonical = Some("https://example.com/b".into());
        assert_eq!(
            status_of(&mut record),
            ("Noindex, Canonicalised".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn an_x_robots_tag_header_counts_as_noindex() {
        let mut record = page("https://example.com/a");
        record.headers = vec![("X-Robots-Tag".into(), "NOINDEX".into())];
        assert!(record.is_noindex());
        assert_eq!(
            status_of(&mut record),
            ("Noindex".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn gtin_check_digits() {
        assert!(gtin_checksum_valid("0123456789012"));
        assert!(gtin_checksum_valid("4006381333931"));
        assert!(gtin_checksum_valid("96385074"));
        assert!(gtin_checksum_valid("036000291452"));
        assert!(gtin_checksum_valid("9780306406157"));
        assert!(!gtin_checksum_valid("0123456789013"));
        assert!(!gtin_checksum_valid("12345"));
        assert!(!gtin_checksum_valid("SKU-123"));
        assert!(!gtin_checksum_valid(""));
    }

    #[test]
    fn none_means_noindex_and_nofollow() {
        let mut record = page("https://example.com/a");
        record.robots = Some("none".into());
        assert!(record.is_noindex());
        assert!(record.has_robots_directive("nofollow"));
        assert!(!record.has_robots_directive("noarchive"));
    }

    #[test]
    fn every_x_robots_tag_header_is_read() {
        let mut record = page("https://example.com/a");
        record.headers = vec![
            ("x-robots-tag".into(), "googlebot: nosnippet".into()),
            ("x-robots-tag".into(), "otherbot: noindex, nofollow".into()),
        ];
        assert!(record.is_noindex());
        assert!(record.has_robots_directive("nosnippet"));
        assert!(record.has_robots_directive("nofollow"));
    }

    #[test]
    fn directive_substrings_do_not_match() {
        let mut record = page("https://example.com/a");
        record.robots = Some("index, follow, max-snippet:-1".into());
        assert!(!record.is_noindex());
        assert!(!record.has_robots_directive("nofollow"));
    }

    #[test]
    fn a_redirect_is_reported_as_redirected() {
        let mut record = page("https://example.com/a");
        record.redirect_url = Some("https://example.com/b".into());
        // The row carries the target's canonical, which says nothing about this
        // URL, so it must not add a second reason.
        record.canonical = Some("https://example.com/b".into());
        assert_eq!(
            status_of(&mut record),
            ("Redirected".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn an_unfollowed_redirect_is_still_a_redirect() {
        // Two of three redirects on the crawled site arrived this way: a 3xx
        // with no final URL, so no `redirect_url` to key off.
        let mut record = page("https://example.com/a");
        record.status = Some(301);
        assert!(record.is_redirect());
        assert_eq!(
            status_of(&mut record),
            ("Redirected".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn a_followed_redirect_carrying_the_targets_status_is_still_a_redirect() {
        let mut record = page("https://example.com/a");
        record.status = Some(200);
        record.redirect_url = Some("https://example.com/b".into());
        assert!(record.is_redirect());
    }

    #[test]
    fn an_ordinary_page_is_not_a_redirect() {
        let mut record = page("https://example.com/a");
        assert!(!record.is_redirect());
        record.status = Some(404);
        assert!(!record.is_redirect());
    }

    #[test]
    fn an_error_status_reports_its_code() {
        let mut record = page("https://example.com/a");
        record.status = Some(404);
        assert_eq!(
            status_of(&mut record),
            ("Non-Indexable (404)".into(), "Non-Indexable".into())
        );
    }

    #[test]
    fn a_row_without_a_status_is_not_assessed() {
        let mut record = page("https://example.com/a");
        record.status = None;
        assert_eq!(status_of(&mut record), ("N/A".into(), "N/A".into()));
    }
}
