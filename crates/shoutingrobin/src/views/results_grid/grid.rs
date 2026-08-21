use std::collections::HashMap;

use gpui::{
    AnyElement, App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render,
    Styled, Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme, Sizable as _, Size,
    spinner::Spinner,
    table::{DataTable, TableEvent, TableState},
};

use crate::crawl::event::PageRecord;
use crate::views::ResultTab;
use crate::views::details_panel::DetailsSelection;

use super::data_build::{build_change_entries, overview_issue_target};
use super::delegate::{
    CrawlSnapshot, ResultsDelegate, baseline_issue_counts, compute_all_tab_filter_counts,
    root_origin_of,
};
use super::types::{FlatRow, IssueFilter, ResultsGridEvent, TabCounts, TabFilterCounts};

/// A crawl with its aggregates already built. Opening a crawl from history used
/// to do this work on the foreground thread, where it froze the window for
/// seconds on a few thousand pages: every tab's filter counts is a pass over
/// the page set per filter, and the baseline's issue counts another.
pub struct PreparedCrawl {
    pages: Vec<PageRecord>,
    baseline: Option<(Vec<PageRecord>, i64)>,
    baseline_issue_counts: HashMap<String, (usize, f32)>,
    tab_filter_counts: HashMap<ResultTab, TabFilterCounts>,
}

impl PreparedCrawl {
    /// Builds the aggregates. Pure and self-contained, so it belongs on a
    /// background thread; the result goes to [`ResultsGrid::load_prepared`].
    pub fn prepare(
        pages: Vec<PageRecord>,
        baseline: Option<(Vec<PageRecord>, i64)>,
        root_url: &str,
    ) -> Self {
        let change_entries = match &baseline {
            Some((baseline_pages, _)) => build_change_entries(&pages, baseline_pages),
            None => Vec::new(),
        };
        let tab_filter_counts = compute_all_tab_filter_counts(
            &pages,
            &change_entries,
            root_origin_of(root_url).as_deref(),
        );
        let baseline_issue_counts = match &baseline {
            Some((baseline_pages, _)) => baseline_issue_counts(baseline_pages),
            None => HashMap::new(),
        };
        Self {
            pages,
            baseline,
            baseline_issue_counts,
            tab_filter_counts,
        }
    }
}

pub struct ResultsGrid {
    state: Entity<TableState<ResultsDelegate>>,
    loading: bool,
    _subscription: Subscription,
}

impl ResultsGrid {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|cx| TableState::new(ResultsDelegate::new(), window, cx));
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

    pub fn push(&mut self, record: PageRecord, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().push(record);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn push_many(&mut self, records: Vec<PageRecord>, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().push_many(records);
            state.refresh(cx);
        });
        cx.notify();
    }

    /// Swaps a prepared crawl in, and leaves the loading state. The primed
    /// counts go in last: every other step invalidates them.
    pub fn load_prepared(
        &mut self,
        prepared: PreparedCrawl,
        root_url: &str,
        cx: &mut Context<Self>,
    ) {
        self.state.update(cx, |state, cx| {
            let delegate = state.delegate_mut();
            delegate.clear();
            delegate.set_root_url(root_url);
            delegate.apply_loaded_crawl(
                prepared.pages,
                prepared.baseline,
                prepared.baseline_issue_counts,
            );
            delegate.prime_counts(prepared.tab_filter_counts);
            state.refresh(cx);
        });
        self.loading = false;
        cx.notify();
    }

    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().clear();
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn switch_tab(&mut self, tab: ResultTab, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().switch_tab(tab);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn set_issue_filter(&mut self, filter: IssueFilter, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().set_issue_filter(filter);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn set_root_url(&mut self, root_url: &str, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().set_root_url(root_url);
            state.refresh(cx);
        });
        cx.notify();
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

    /// Per-tab badge counts. Goes through `state.update` so the delegate's lazy
    /// counts cache can fill on first access.
    pub fn tab_badges(&self, cx: &mut App) -> HashMap<ResultTab, TabCounts> {
        self.state.update(cx, |state, _cx| {
            state
                .delegate_mut()
                .tab_filter_counts()
                .iter()
                .map(|(&tab, counts)| (tab, counts.badge.clone()))
                .collect()
        })
    }

    /// The sub-filter counts for the active tab, in `filters_for_tab` order.
    pub fn active_filter_counts(&self, cx: &mut App) -> Vec<(IssueFilter, usize)> {
        self.state.update(cx, |state, _cx| {
            let tab = state.delegate().active_tab;
            state
                .delegate_mut()
                .tab_filter_counts()
                .get(&tab)
                .map(|counts| counts.filter_counts.clone())
                .unwrap_or_default()
        })
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

        assert_eq!(prepared.tab_filter_counts.len(), expected.len());
        for (tab, counts) in &prepared.tab_filter_counts {
            let want = expected.get(tab).expect("every tab is counted");
            assert_eq!(counts.filter_counts, want.filter_counts, "{tab:?}");
            assert_eq!(counts.badge.total, want.badge.total, "{tab:?}");
            assert_eq!(counts.badge.errors, want.badge.errors, "{tab:?}");
            assert_eq!(counts.badge.warnings, want.badge.warnings, "{tab:?}");
        }
    }
}
