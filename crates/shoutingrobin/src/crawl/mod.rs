pub mod analyzers;
pub mod engine;
pub mod event;
pub mod render_mode;
pub mod similarity;
pub mod sitemap;

pub use engine::CrawlEngine;
pub use event::CrawlEvent;
pub use render_mode::RenderMode;

pub struct CrawlConfig {
    pub max_pages: u32,
    pub max_concurrent: u32,
    pub delay_ms: u64,
    pub timeout_seconds: u32,
    pub respect_robots_txt: bool,
    pub near_duplicate_threshold: u8,
    pub content_selector: String,
}
