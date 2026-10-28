use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::component::{
    ActiveTheme, Sizable as _, Size,
    spinner::Spinner,
    table::{DataTable, TableEvent, TableState},
};
use gpui_kit::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render,
    SharedString, Styled, Subscription, Task, Window, div,
};

use crate::crawl::event::PageRecord;
use crate::views::ResultTab;
use crate::views::details_panel::DetailsSelection;

use super::data_build::overview_issue_target;
use super::delegate::{
    CountedCrawl, CountsJob, CrawlSnapshot, ResultsDelegate, baseline_issue_counts,
};
use super::types::{FlatRow, IssueFilter, ResultsGridEvent, TabCounts};

/// The least time between the starts of two counting passes. A pass over a
/// large crawl takes about this long anyway; on a small one, pages arriving
/// every few hundred milliseconds would otherwise keep a thread counting.
const COUNTS_INTERVAL: Duration = Duration::from_secs(1);

/// A crawl with its aggregates already built. Opening a crawl from history used
/// to do this work on the foreground thread, where it froze the window for
/// seconds on a few thousand pages: every tab's filter counts is a pass over
/// the page set per filter, and the baseline's issue counts another.
pub struct PreparedCrawl {
    pages: Vec<Arc<PageRecord>>,
    baseline: Option<(Vec<Arc<PageRecord>>, i64)>,
    baseline_issue_counts: HashMap<String, (usize, f32)>,
    counted: CountedCrawl,
}

impl PreparedCrawl {
    /// Builds the aggregates. Pure and self-contained, so it belongs on a
    /// background thread; the result goes to [`ResultsGrid::load_prepared`].
    pub fn prepare(
        pages: Vec<PageRecord>,
        baseline: Option<(Vec<PageRecord>, i64)>,
        root_url: &str,
    ) -> Self {
        let pages: Vec<Arc<PageRecord>> = pages.into_iter().map(Arc::new).collect();
        let baseline = baseline.map(|(baseline_pages, started_at)| {
            (
                baseline_pages.into_iter().map(Arc::new).collect::<Vec<_>>(),
                started_at,
            )
        });
        let counted = CountsJob::for_crawl(
            pages.clone(),
            baseline
                .as_ref()
                .map(|(baseline_pages, _)| baseline_pages.clone()),
            root_url,
        )
        .run();
        let baseline_issue_counts = match &baseline {
            Some((baseline_pages, _)) => baseline_issue_counts(baseline_pages),
            None => HashMap::new(),
        };
        Self {
            pages,
            baseline,
            baseline_issue_counts,
            counted,
        }
    }
}

pub struct ResultsGrid {
    state: Entity<TableState<ResultsDelegate>>,
    loading: bool,
    layout: LayoutContext,
    /// The counting pass in flight, or waiting out `COUNTS_INTERVAL`.
    counts_task: Option<Task<()>>,
    counts_started_at: Option<Instant>,
    _subscription: Subscription,
}

/// What the layout views need to know about the crawl on screen.
#[derive(Clone, Debug, Default)]
pub struct LayoutContext {
    pub crawl_id: Option<i64>,
    /// The baseline crawl whose layouts are compared with this one's. Unset
    /// when there is none, or when it captured layout with other settings.
    pub baseline_crawl_id: Option<i64>,
    /// Shown next to the baseline note: a comparison in progress, or why
    /// layouts are not compared.
    pub note: Option<SharedString>,
}

impl ResultsGrid {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state =
            cx.new(|cx| TableState::new(ResultsDelegate::new().count_in_background(), window, cx));
        let sub = cx.subscribe(&state, |this, _state, event: &TableEvent, cx| {
            if let TableEvent::SelectRow(row_ix) = event {
                let delegate = this.state.read(cx).delegate();
                if delegate.active_tab == ResultTab::Overview
                    && let Some(FlatRow::IssuesRow { index }) = delegate.flat_rows().get(*row_ix)
                    && let Some(entry) = delegate.issue_entries().get(*index)
                    && let Some((tab, filter)) = overview_issue_target(&entry.name)
                {
                    cx.emit(ResultsGridEvent::OverviewDrillDown { tab, filter });
                    return;
                }
                cx.emit(ResultsGridEvent::Selected(*row_ix))
            }
        });
        Self {
            state,
            loading: false,
            layout: LayoutContext::default(),
            counts_task: None,
            counts_started_at: None,
            _subscription: sub,
        }
    }

    /// Puts the grid in its loading state, which replaces the table with a
    /// spinner. Opening a crawl from history reads two crawls out of the
    /// database and then rebuilds every aggregate, and the table would
    /// otherwise sit there showing the previous crawl until that finishes.
    pub fn set_loading(&mut self, loading: bool, cx: &mut Context<Self>) {
        if self.loading == loading {
            return;
        }
        self.loading = loading;
        cx.notify();
    }

    /// Applies a change to the rows, then brings the counts up to date in the
    /// background.
    fn update_rows(&mut self, cx: &mut Context<Self>, change: impl FnOnce(&mut ResultsDelegate)) {
        self.state.update(cx, |state, cx| {
            change(state.delegate_mut());
            state.refresh(cx);
        });
        cx.notify();
        self.refresh_counts(cx);
    }

    /// Starts a counting pass when the counts are behind and none is running.
    /// A running pass starts the next one when it lands, which then counts the
    /// pages streamed in meanwhile. Until a pass lands, the tabs keep showing
    /// the counts before it.
    fn refresh_counts(&mut self, cx: &mut Context<Self>) {
        if self.counts_task.is_some() || self.state.read(cx).delegate().counts_are_current() {
            return;
        }
        let wait = self
            .counts_started_at
            .map(|started_at| COUNTS_INTERVAL.saturating_sub(started_at.elapsed()))
            .unwrap_or_default();
        self.counts_task = Some(cx.spawn(async move |this, cx| {
            if !wait.is_zero() {
                cx.background_executor().timer(wait).await;
            }
            // Taken after the wait, so it counts the pages that arrived during it.
            let job = this.update(cx, |grid, cx| {
                grid.counts_started_at = Some(Instant::now());
                let job = grid
                    .state
                    .update(cx, |state, _| state.delegate_mut().counts_job());
                if job.is_none() {
                    grid.counts_task = None;
                }
                job
            });
            let job = match job {
                Ok(Some(job)) => job,
                Ok(None) => return,
                Err(error) => {
                    tracing::error!(%error, "the results grid went away before counting");
                    return;
                }
            };
            let counted = cx.background_spawn(async move { job.run() }).await;
            if let Err(error) = this.update(cx, |grid, cx| {
                grid.counts_task = None;
                grid.state.update(cx, |state, cx| {
                    state.delegate_mut().finish_counts(counted);
                    state.refresh(cx);
                });
                cx.notify();
                grid.refresh_counts(cx);
            }) {
                tracing::error!(%error, "the results grid went away while counting");
            }
        }));
    }

    pub fn push_many(&mut self, records: Vec<PageRecord>, cx: &mut Context<Self>) {
        self.update_rows(cx, |delegate| delegate.push_many(records));
    }

    /// Adds the streamed records the grid does not hold yet. A crawl read back
    /// from the database while it runs may already hold pages that were still
    /// on their way to the grid when the read began.
    pub fn push_unseen(&mut self, records: Vec<PageRecord>, cx: &mut Context<Self>) {
        let unseen = self.state.read(cx).delegate().without_held(records);
        self.push_many(unseen, cx);
    }

    /// Swaps a prepared crawl in, and leaves the loading state.
    pub fn load_prepared(
        &mut self,
        prepared: PreparedCrawl,
        root_url: &str,
        cx: &mut Context<Self>,
    ) {
        self.loading = false;
        self.update_rows(cx, |delegate| {
            delegate.clear();
            delegate.set_root_url(root_url);
            delegate.apply_loaded_crawl(
                prepared.pages,
                prepared.baseline,
                prepared.baseline_issue_counts,
                prepared.counted,
            );
        });
    }

    pub fn layout_context(&self) -> &LayoutContext {
        &self.layout
    }

    pub fn set_layout_context(&mut self, layout: LayoutContext, cx: &mut Context<Self>) {
        self.layout = layout;
        cx.notify();
    }

    /// Installs the pages whose layout changed since the baseline, for the
    /// crawl `crawl_id`. A result for a crawl that is no longer on screen is
    /// dropped.
    pub fn set_layout_changes(
        &mut self,
        crawl_id: i64,
        layout_changes: HashMap<String, crate::crawl::layout::PageLayoutChange>,
        cx: &mut Context<Self>,
    ) {
        if self.layout.crawl_id != Some(crawl_id) {
            return;
        }
        self.layout.note = None;
        self.update_rows(cx, |delegate| delegate.set_layout_changes(layout_changes));
    }

    pub fn has_layouts(&self, cx: &App) -> bool {
        self.state.read(cx).delegate().has_layouts()
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.layout = LayoutContext::default();
        self.update_rows(cx, |delegate| delegate.clear());
    }

    pub fn switch_tab(&mut self, tab: ResultTab, cx: &mut Context<Self>) {
        self.update_rows(cx, |delegate| delegate.switch_tab(tab));
    }

    pub fn set_issue_filter(&mut self, filter: IssueFilter, cx: &mut Context<Self>) {
        self.update_rows(cx, |delegate| delegate.set_issue_filter(filter));
    }

    pub fn set_root_url(&mut self, root_url: &str, cx: &mut Context<Self>) {
        self.update_rows(cx, |delegate| delegate.set_root_url(root_url));
    }

    #[cfg(test)]
    pub(super) fn scroll_to_row_for_bench(&mut self, row_ix: usize, cx: &mut Context<Self>) {
        self.state
            .update(cx, |state, cx| state.scroll_to_row(row_ix, cx));
    }

    pub fn has_baseline(&self, cx: &App) -> bool {
        self.state.read(cx).delegate().has_baseline()
    }

    pub fn baseline_started_at(&self, cx: &App) -> Option<i64> {
        self.state.read(cx).delegate().baseline_started_at()
    }

    pub fn selection_at(&self, index: usize, cx: &App) -> Option<DetailsSelection> {
        self.state.read(cx).delegate().selection_at(index)
    }

    pub fn row_count(&self, cx: &App) -> usize {
        self.state.read(cx).delegate().filtered_count()
    }

    /// Per-tab badge counts, from the latest counting pass.
    pub fn tab_badges(&self, cx: &App) -> HashMap<ResultTab, TabCounts> {
        self.state
            .read(cx)
            .delegate()
            .displayed_counts()
            .map(|counts| {
                counts
                    .iter()
                    .map(|(&tab, counts)| (tab, counts.badge.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The sub-filter counts for the active tab, in `filters_for_tab` order,
    /// from the latest counting pass.
    pub fn active_filter_counts(&self, cx: &App) -> Vec<(IssueFilter, usize)> {
        let delegate = self.state.read(cx).delegate();
        delegate
            .displayed_counts()
            .and_then(|counts| counts.get(&delegate.active_tab))
            .map(|counts| counts.filter_counts.clone())
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn active_tab(&self, cx: &App) -> ResultTab {
        self.state.read(cx).delegate().active_tab()
    }

    pub fn export_csv(&self, cx: &App) -> Result<String, csv::Error> {
        self.state.read(cx).delegate().export_csv()
    }

    pub fn snapshot(&self, cx: &App) -> CrawlSnapshot {
        self.state.read(cx).delegate().snapshot()
    }

    /// The loaded crawl, in the form the PDF report renders.
    pub fn build_report(&self, render_mode: &str, cx: &App) -> crate::report::Report {
        self.state.read(cx).delegate().build_report(render_mode)
    }

    pub fn root_url(&self, cx: &App) -> Option<String> {
        self.state
            .read(cx)
            .delegate()
            .root_url()
            .map(|s| s.to_owned())
    }

    pub fn has_results(&self, cx: &App) -> bool {
        self.state.read(cx).delegate().filtered_count() > 0
    }

    #[cfg(feature = "screenshots")]
    pub(crate) fn is_loading(&self) -> bool {
        self.loading
    }

    /// The loaded crawl's documents and resources, for the status bar.
    #[cfg(feature = "screenshots")]
    pub(crate) fn page_and_resource_counts(&self, cx: &App) -> (u64, u64) {
        let pages = &self.state.read(cx).delegate().all_pages;
        let documents = pages.iter().filter(|page| page.is_page).count() as u64;
        (documents, pages.len() as u64 - documents)
    }

    /// Selects the first row of the active tab whose page URL `matches`, as
    /// clicking it does, which shows it in the details panel.
    #[cfg(feature = "screenshots")]
    pub(crate) fn select_row_where(
        &mut self,
        matches: impl Fn(&PageRecord) -> bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let delegate = self.state.read(cx).delegate();
        let Some(row_ix) =
            (0..delegate.filtered_count()).find(|&ix| delegate.record_at(ix).is_some_and(&matches))
        else {
            return false;
        };
        self.state
            .update(cx, |state, cx| state.set_selected_row(row_ix, cx));
        true
    }
}

impl EventEmitter<ResultsGridEvent> for ResultsGrid {}

impl Render for ResultsGrid {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content: AnyElement = if self.loading {
            div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_3()
                .text_color(cx.theme().muted_foreground)
                .child(Spinner::new().with_size(Size::Large))
                .child(div().text_sm().child("Loading crawl…"))
                .into_any_element()
        } else {
            DataTable::new(&self.state)
                .bordered(false)
                .stripe(true)
                .into_any_element()
        };
        div()
            .flex_1()
            .size_full()
            .min_h_0()
            .rounded_bl(crate::app::PANEL_RADIUS)
            .child(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "https://a.test/";

    fn pages(count: usize, prefix: &str) -> Vec<PageRecord> {
        (0..count)
            .map(|index| PageRecord {
                url: format!("{ROOT}{prefix}-{index}"),
                is_internal: true,
                is_page: true,
                status: Some(200),
                title: Some(format!("Title {index}")),
                meta_description: Some(format!("Meta description {index}")),
                h1: Some(format!("H1 {index}")),
                word_count: Some(500),
                ..Default::default()
            })
            .collect()
    }

    /// The point of preparing a crawl on a background thread is that the grid
    /// then skips the lazy pass, so what is primed has to be what that pass
    /// would have produced. A drift here shows up as tab badges that disagree
    /// with the rows behind them.
    #[test]
    fn a_prepared_crawl_carries_the_counts_the_lazy_pass_would_compute() {
        let current = pages(12, "page");
        let mut baseline_pages = pages(10, "page");
        baseline_pages.push(PageRecord {
            url: format!("{ROOT}gone"),
            is_internal: true,
            is_page: true,
            status: Some(200),
            ..Default::default()
        });
        let started_at = 1_700_000_000;

        let prepared = PreparedCrawl::prepare(
            current.clone(),
            Some((baseline_pages.clone(), started_at)),
            ROOT,
        );

        let mut lazy = ResultsDelegate::new();
        lazy.set_root_url(ROOT);
        lazy.replace_records(current);
        lazy.set_baseline(baseline_pages, started_at);
        let expected = lazy.tab_filter_counts().clone();

        assert_eq!(prepared.counted.counts().len(), expected.len());
        for (tab, counts) in prepared.counted.counts() {
            let want = expected.get(tab).expect("every tab is counted");
            assert_eq!(counts.filter_counts, want.filter_counts, "{tab:?}");
            assert_eq!(counts.badge.total, want.badge.total, "{tab:?}");
            assert_eq!(counts.badge.errors, want.badge.errors, "{tab:?}");
            assert_eq!(counts.badge.warnings, want.badge.warnings, "{tab:?}");
        }
    }
}
