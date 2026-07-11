pub mod crawl_bar;
pub mod crawls_sidebar;
pub mod details_panel;
pub mod results_grid;
pub mod status_bar;
pub mod tabs_bar;

pub use crawl_bar::CrawlBar;
pub use crawls_sidebar::CrawlsSidebar;
pub use details_panel::DetailsPanel;
pub use results_grid::ResultsGrid;
pub use status_bar::StatusBar;
pub use tabs_bar::ResultTab;

/// Renders a Unix timestamp as a short relative label ("just now", "5m ago",
/// "3h ago", "2d ago", or a "Mon 5" date once it is older than a week). Shared
/// by the crawl sidebar and the baseline comparison label so both stay in step.
pub fn relative_time(now: i64, ts: i64) -> String {
    let delta = now.saturating_sub(ts);
    if delta < 60 {
        return "just now".into();
    }
    if delta < 3600 {
        return format!("{}m ago", delta / 60);
    }
    if delta < 86_400 {
        return format!("{}h ago", delta / 3600);
    }
    if delta < 7 * 86_400 {
        return format!("{}d ago", delta / 86_400);
    }
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%b %-d").to_string())
        .unwrap_or_else(|| "unknown".into())
}
