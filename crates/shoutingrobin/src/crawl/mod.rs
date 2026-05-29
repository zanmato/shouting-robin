pub mod analyzers;
pub mod engine;
pub mod event;
pub mod render_mode;
pub mod similarity;
pub mod sitemap;

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
}
