pub mod analyzers;
pub mod engine;
pub mod event;
pub mod font_metrics;
pub mod layout;
pub mod layout_groups;
pub mod render_mode;
pub mod resources;
pub mod similarity;
pub mod site_icon;
pub mod site_settings;
pub mod sitemap;
pub mod url_norm;

pub use engine::CrawlEngine;
pub use event::CrawlEvent;
pub use render_mode::RenderMode;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CrawlConfig {
    pub max_pages: u32,
    pub max_concurrent: u32,
    pub delay_ms: u64,
    pub timeout_seconds: u32,
    pub respect_robots_txt: bool,
    #[serde(default)]
    pub follow_sitemaps: bool,
    /// In Chrome mode, block images, media, fonts, stylesheets and analytics
    /// during render (first-party assets are still allowed). Speeds up crawls.
    #[serde(default)]
    pub block_images: bool,
    pub near_duplicate_threshold: u8,
    pub content_selector: String,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub extra_headers: Vec<(String, String)>,
    #[serde(default)]
    pub include_patterns: Vec<String>,
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
    #[serde(default)]
    pub crawl_subdomains: bool,
    #[serde(default)]
    pub list_mode: bool,
    #[serde(default)]
    pub seed_urls: Vec<String>,
    /// Request every discovered image, stylesheet, script and external link
    /// once after the page crawl, to record its status, type and size.
    ///
    /// On by default: without it a broken image or a dead external link is
    /// invisible. Off is one request per page and nothing else, which is what
    /// the test suites want so they never reach a third-party host.
    #[serde(default = "default_true")]
    pub check_resources: bool,
    /// Whether a Chrome crawl records the boxes each page renders, for layout
    /// grouping and layout changes between crawls. Always off for HTTP.
    #[serde(default)]
    pub layout_capture: LayoutCapture,
}

/// How a Chrome crawl records page layout.
///
/// Two crawls are only comparable when they captured the same way: content
/// that loads on scroll is in one and missing from the other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutCapture {
    #[default]
    Off,
    /// The page as it is once loaded, without scrolling. Content that loads
    /// on scroll is left out, but every crawl leaves out the same content.
    Loaded,
    /// Scrolls through the page first, so content that loads on scroll is
    /// recorded too. Slower, by seconds a page.
    Scrolled,
}

impl LayoutCapture {
    /// The value the two crawl switches describe. Scrolling means nothing
    /// while capture is off.
    pub fn from_switches(capture: bool, scroll: bool) -> Self {
        match (capture, scroll) {
            (false, _) => Self::Off,
            (true, false) => Self::Loaded,
            (true, true) => Self::Scrolled,
        }
    }

    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }

    pub fn scrolls(self) -> bool {
        self == Self::Scrolled
    }

    pub fn describe(self) -> &'static str {
        match self {
            Self::Off => "Layout not captured",
            Self::Loaded => "Layout captured, without scrolling",
            Self::Scrolled => "Layout captured, with scrolling",
        }
    }
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::LayoutCapture;

    #[test]
    fn scrolling_only_counts_while_capture_is_on() {
        assert_eq!(
            LayoutCapture::from_switches(false, false),
            LayoutCapture::Off
        );
        assert_eq!(
            LayoutCapture::from_switches(false, true),
            LayoutCapture::Off
        );
        assert_eq!(
            LayoutCapture::from_switches(true, false),
            LayoutCapture::Loaded
        );
        assert_eq!(
            LayoutCapture::from_switches(true, true),
            LayoutCapture::Scrolled
        );
    }
}
