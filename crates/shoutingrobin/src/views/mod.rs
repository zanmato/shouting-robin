pub mod crawl_bar;
pub mod crawls_sidebar;
pub mod details_panel;
pub mod filter_bar;
pub mod layout_view;
pub mod results_grid;
pub mod status_bar;
pub mod tabs_bar;

pub use crawl_bar::CrawlBar;
pub use crawls_sidebar::CrawlsSidebar;
pub use details_panel::DetailsPanel;
pub use results_grid::ResultsGrid;
pub use status_bar::StatusBar;
pub use tabs_bar::ResultTab;

use chrono::Datelike as _;

/// Renders a Unix timestamp as a short relative label ("just now", "5m ago",
/// "3h ago", "2d ago", or a "Mon 5" date once it is older than a week). Shared
/// by the crawl sidebar and the baseline comparison label so both stay in step.
pub fn relative_time(now: i64, ts: i64) -> String {
    let delta = now.saturating_sub(ts);
    if delta < 60 {
        return t!("results_grid.relative_time.just_now").into_owned();
    }
    if delta < 3600 {
        return t!("results_grid.relative_time.minutes_ago", count = delta / 60).into_owned();
    }
    if delta < 86_400 {
        return t!("results_grid.relative_time.hours_ago", count = delta / 3600).into_owned();
    }
    if delta < 7 * 86_400 {
        return t!(
            "results_grid.relative_time.days_ago",
            count = delta / 86_400
        )
        .into_owned();
    }
    let Some(date) = chrono::DateTime::from_timestamp(ts, 0) else {
        return t!("results_grid.relative_time.unknown").into_owned();
    };
    t!(
        "results_grid.relative_time.date",
        month = month_abbreviation(date.month()),
        day = date.day()
    )
    .into_owned()
}

fn month_abbreviation(month: u32) -> std::borrow::Cow<'static, str> {
    match month {
        1 => t!("results_grid.month.january"),
        2 => t!("results_grid.month.february"),
        3 => t!("results_grid.month.march"),
        4 => t!("results_grid.month.april"),
        5 => t!("results_grid.month.may"),
        6 => t!("results_grid.month.june"),
        7 => t!("results_grid.month.july"),
        8 => t!("results_grid.month.august"),
        9 => t!("results_grid.month.september"),
        10 => t!("results_grid.month.october"),
        11 => t!("results_grid.month.november"),
        _ => t!("results_grid.month.december"),
    }
}
