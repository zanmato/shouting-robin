//! Cost of the results grid on a large crawl read from a copy of the app
//! database. Ignored by default: both tests want a release build and a real
//! crawl, which is not part of the repository.
//!
//! Take a consistent copy of a live database first (the app keeps it in WAL
//! mode, so copying the file alone can miss committed pages):
//!
//! ```sh
//! sqlite3 -readonly ~/.local/share/shoutingrobin/shoutingrobin.db ".backup snapshot.db"
//! GRID_BENCH_DB=snapshot.db GRID_BENCH_CRAWL=95 \
//!     cargo test --release -p shoutingrobin --bin shoutingrobin results_grid_ \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `results_grid_costs` times the steps a streamed batch triggers, tab by tab.
//! `results_grid_frames` measures whole frames with GPUI's bench harness;
//! `GRID_BENCH=<substring>` runs only the scenarios whose name contains it.

use std::str::FromStr as _;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::{
    AppContext as _, BenchAppContext, BenchMeasurement, BenchReport, Context, Entity, IntoElement,
    ParentElement as _, Render, Styled as _, Window, bench_platform, div,
};

use super::data_build::build_issues_entries;
use super::delegate::{ResultsDelegate, root_origin_of};
use super::filter::{CountsProgress, compute_tab_filter_counts, count_every_tab};
use super::grid::ResultsGrid;
use super::types::shared;
use crate::crawl::event::PageRecord;
use crate::views::ResultTab;

/// How many records the event pump typically folds into one push while a
/// crawl with resource checks is running.
const BATCH: usize = 20;

struct Crawl {
    pages: Vec<PageRecord>,
    root_url: String,
}

fn load_crawl() -> Option<Crawl> {
    let Ok(path) = std::env::var("GRID_BENCH_DB") else {
        eprintln!("GRID_BENCH_DB is not set, skipping");
        return None;
    };
    let crawl_id: i64 = std::env::var("GRID_BENCH_CRAWL")
        .ok()
        .and_then(|value| value.parse().ok())
        .expect("GRID_BENCH_CRAWL should name a crawl id");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let options = sqlx::sqlite::SqliteConnectOptions::from_str(&format!("sqlite://{path}"))
            .expect("database path")
            .read_only(true);
        let pool = sqlx::SqlitePool::connect_with(options)
            .await
            .expect("open the database copy");
        let (root_url,): (String,) = sqlx::query_as("SELECT root_url FROM crawls WHERE id = ?")
            .bind(crawl_id)
            .fetch_one(&pool)
            .await
            .expect("the crawl exists");
        let started = Instant::now();
        let mut step_started = started;
        let mut previous_step = None;
        let pages =
            crate::storage::load_pages_for_crawl_reporting(&pool, crawl_id, &root_url, |step| {
                if let Some(previous) = previous_step.replace(step) {
                    eprintln!(
                        "  {previous:?}: {:.0} ms",
                        milliseconds(step_started.elapsed())
                    );
                }
                step_started = Instant::now();
            })
            .await
            .expect("load the crawl");
        if let Some(last) = previous_step {
            eprintln!("  {last:?}: {:.0} ms", milliseconds(step_started.elapsed()));
        }
        let resources = pages.iter().filter(|page| !page.is_page).count();
        eprintln!(
            "load_pages_for_crawl: {} rows ({} pages, {resources} resources) in {:.0} ms",
            pages.len(),
            pages.len() - resources,
            started.elapsed().as_secs_f64() * 1000.
        );
        Some(Crawl { pages, root_url })
    })
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.
}

/// Where a streamed batch's time goes. On the foreground thread: the push, which
/// rebuilds the open tab's rows. On a background thread: the counting pass,
/// first over the whole crawl as it stood a batch ago, then over that one more
/// batch, which is what each pass costs while a crawl streams in. The streamed
/// counts are checked against a full count on the way.
#[test]
#[ignore = "needs GRID_BENCH_DB and a release build"]
fn results_grid_costs() {
    let Some(crawl) = load_crawl() else {
        return;
    };
    let origin = root_origin_of(&crawl.root_url);
    let pages = shared(crawl.pages.iter().cloned());
    let split = pages.len().saturating_sub(BATCH);

    let mut per_tab_counts: Vec<(ResultTab, Duration)> = ResultTab::ALL
        .iter()
        .map(|&tab| {
            let started = Instant::now();
            std::hint::black_box(compute_tab_filter_counts(
                tab,
                &pages,
                &[],
                origin.as_deref(),
            ));
            (tab, started.elapsed())
        })
        .collect();
    let all_counts: Duration = per_tab_counts.iter().map(|(_, duration)| *duration).sum();
    per_tab_counts.sort_by_key(|(_, duration)| std::cmp::Reverse(*duration));
    eprintln!(
        "\nevery tab counted from scratch: {:.0} ms",
        milliseconds(all_counts)
    );
    for (tab, duration) in per_tab_counts.iter().take(8) {
        eprintln!(
            "  {:<16} {:>8.1} ms",
            format!("{tab:?}"),
            milliseconds(*duration)
        );
    }

    let mut progress = CountsProgress::default();
    let pass = |pages: &[Arc<PageRecord>], progress: &mut CountsProgress| {
        let started = Instant::now();
        let issue_entries = build_issues_entries(pages);
        let entries = started.elapsed();
        let counts = count_every_tab(pages, &[], &issue_entries, origin.as_deref(), progress);
        (counts, entries, started.elapsed())
    };
    let (_, entries, first) = pass(&pages[..split], &mut progress);
    eprintln!(
        "\nbackground pass over {split} rows: {:.0} ms ({:.0} ms of it the Overview's issue entries)",
        milliseconds(first),
        milliseconds(entries)
    );
    let (streamed, entries, next) = pass(&pages, &mut progress);
    eprintln!(
        "background pass after {BATCH} more rows: {:.0} ms ({:.0} ms of it the Overview's issue entries)",
        milliseconds(next),
        milliseconds(entries)
    );
    for &tab in ResultTab::ALL {
        let full = compute_tab_filter_counts(tab, &pages, &[], origin.as_deref());
        let counted = streamed.get(&tab).expect("every tab is counted");
        assert_eq!(counted.filter_counts, full.filter_counts, "{tab:?}");
        assert_eq!(counted.badge.errors, full.badge.errors, "{tab:?}");
        assert_eq!(counted.badge.warnings, full.badge.warnings, "{tab:?}");
    }

    let mut delegate = ResultsDelegate::new();
    delegate.switch_tab(ResultTab::ResponseCodes);
    delegate.push_many(crawl.pages.clone());
    let broken_resource = (0..delegate.filtered_count()).find(|&row| {
        delegate
            .record_at(row)
            .is_some_and(|record| record.is_resource && record.status.is_some_and(|s| s >= 400))
    });
    if let Some(row) = broken_resource {
        let started = Instant::now();
        let selection = delegate.selection_at(row);
        let elapsed = started.elapsed();
        let references = match &selection {
            Some(super::super::details_panel::DetailsSelection::Page { references, .. }) => {
                references.len()
            }
            _ => 0,
        };
        eprintln!(
            "\nselecting a broken resource: {:.1} ms, used as an image {references} times",
            milliseconds(elapsed)
        );
    }

    let (initial, batch) = crawl.pages.split_at(split);
    eprintln!(
        "\nforeground cost of one batch of {BATCH} rows pushed onto {split}, per open tab:\n  {:<16} {:>10} {:>12}",
        "tab", "push", "has_layouts"
    );
    for &tab in ResultTab::ALL {
        let mut delegate = ResultsDelegate::new().count_in_background();
        delegate.set_root_url(&crawl.root_url);
        delegate.switch_tab(tab);
        delegate.push_many(initial.to_vec());

        let records = batch.to_vec();
        let started = Instant::now();
        delegate.push_many(records);
        let push = started.elapsed();

        let started = Instant::now();
        std::hint::black_box(delegate.has_layouts());
        let has_layouts = started.elapsed();

        eprintln!(
            "  {:<16} {:>7.1} ms {:>9.3} ms",
            format!("{tab:?}"),
            milliseconds(push),
            milliseconds(has_layouts)
        );
    }
}

/// The grid as the app hosts it: every render of the app root asks the grid
/// for its tab badges, the active tab's sub-filter counts, whether there are
/// layouts and the row count, so a frame pays for those too.
struct Host {
    grid: Entity<ResultsGrid>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = self.grid.read(cx);
        std::hint::black_box(grid.tab_badges(cx));
        std::hint::black_box(grid.active_filter_counts(cx));
        std::hint::black_box(grid.has_layouts(cx));
        std::hint::black_box(grid.row_count(cx));
        div().size_full().child(self.grid.clone())
    }
}

#[derive(Clone, Copy)]
enum Frame {
    /// The app re-renders with nothing changed, as when the status bar ticks.
    Rerender,
    /// The table scrolls by a few rows.
    Scroll,
    /// A batch of streamed records arrives, as during a crawl.
    Stream,
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    tab: ResultTab,
    frame: Frame,
}

const SCENARIOS: [Scenario; 8] = [
    Scenario {
        name: "internal/rerender",
        tab: ResultTab::Internal,
        frame: Frame::Rerender,
    },
    Scenario {
        name: "internal/scroll",
        tab: ResultTab::Internal,
        frame: Frame::Scroll,
    },
    Scenario {
        name: "internal/stream",
        tab: ResultTab::Internal,
        frame: Frame::Stream,
    },
    Scenario {
        name: "response-codes/stream",
        tab: ResultTab::ResponseCodes,
        frame: Frame::Stream,
    },
    Scenario {
        name: "overview/rerender",
        tab: ResultTab::Overview,
        frame: Frame::Rerender,
    },
    Scenario {
        name: "overview/stream",
        tab: ResultTab::Overview,
        frame: Frame::Stream,
    },
    Scenario {
        name: "images/stream",
        tab: ResultTab::Images,
        frame: Frame::Stream,
    },
    Scenario {
        name: "site-structure/stream",
        tab: ResultTab::SiteStructure,
        frame: Frame::Stream,
    },
];

fn run_scenario(scenario: Scenario, crawl: &Crawl, cx: &mut BenchAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::ui::fonts::register(cx);
    });
    let mut window = cx.add_empty_window();
    let grid: Entity<ResultsGrid> = window.update(|window, cx| {
        let grid = cx.new(|cx| ResultsGrid::new(window, cx));
        let host = cx.new(|_| Host { grid: grid.clone() });
        window.replace_root(cx, |window, cx| {
            gpui_kit::component::Root::new(host, window, cx)
        });
        grid
    });
    let split = crawl.pages.len().saturating_sub(BATCH * 50);
    let (initial, streamed) = crawl.pages.split_at(split);
    window.update(|_, cx| {
        grid.update(cx, |grid, cx| {
            grid.set_root_url(&crawl.root_url, cx);
            grid.switch_tab(scenario.tab, cx);
            grid.push_many(initial.to_vec(), cx);
        });
    });
    window.run_until_idle();
    // The tabs show counts only once a background pass lands, which proves
    // the grid's counting runs at all outside the delegate's own tests.
    let counted = window.update(|_, cx| grid.read(cx).tab_badges(cx));
    assert!(!counted.is_empty(), "no counting pass landed");

    let streamed = streamed.to_vec();
    let app = window.app_context();
    let mut frame_index = 0usize;
    app.bench_renderer(grid, move |grid, _window, cx| {
        frame_index += 1;
        match scenario.frame {
            Frame::Rerender => {}
            Frame::Scroll => {
                let rows = grid.row_count(cx).max(1);
                grid.scroll_to_row_for_bench(frame_index * 3 % rows, cx);
            }
            Frame::Stream => {
                let chunks = streamed.len().div_ceil(BATCH).max(1);
                let start = frame_index % chunks * BATCH;
                let end = (start + BATCH).min(streamed.len());
                if let Some(records) = streamed.get(start..end) {
                    grid.push_many(records.to_vec(), cx);
                }
            }
        }
        cx.notify();
    });
}

#[test]
#[ignore = "needs GRID_BENCH_DB and a release build"]
fn results_grid_frames() {
    let Some(crawl) = load_crawl() else {
        return;
    };
    // See `details_panel_frames`: without a session bus the reduce motion
    // watcher gives up before outliving the app Criterion builds per pass.
    // SAFETY: set before any app or thread of this test exists, and the test
    // runs with --test-threads=1.
    unsafe { std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "disabled:") };
    let filter = std::env::var("GRID_BENCH").unwrap_or_default();
    let mut criterion = criterion::Criterion::default()
        .with_measurement(BenchMeasurement::from_env_or_exit())
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .without_plots();
    let crawl = std::sync::Arc::new(crawl);
    for scenario in SCENARIOS
        .into_iter()
        .filter(|scenario| scenario.name.contains(filter.as_str()))
    {
        let report = BenchReport::with_fps(120);
        criterion.bench_function(scenario.name, {
            let report = report.clone();
            let crawl = crawl.clone();
            move |bencher| {
                let mut cx = BenchAppContext::new_with_platform_and_report(
                    bench_platform(
                        Some(Box::new(gpui_kit::platform::current_headless_renderer)),
                        gpui_kit::platform::bench_text_system(),
                    ),
                    Some("results_grid_frames"),
                    bencher,
                    report.clone(),
                );
                run_scenario(scenario, &crawl, &mut cx);
                cx.teardown();
            }
        });
        report.print(scenario.name);
    }
}
