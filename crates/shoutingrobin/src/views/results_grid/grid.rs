use std::collections::HashMap;

use gpui::{
    App, AppContext, Context, Entity, EventEmitter, IntoElement, ParentElement, Render, Styled,
    Subscription, Window, div,
};
use gpui_component::{
    ActiveTheme,
    table::{DataTable, TableEvent, TableState},
};

use crate::crawl::event::PageRecord;
use crate::views::ResultTab;

use super::data_build::{build_issues_entries, overview_issue_target};
use super::delegate::ResultsDelegate;
use super::types::{FlatRow, IssueFilter, ResultsGridEvent, TabCounts};

pub struct ResultsGrid {
    state: Entity<TableState<ResultsDelegate>>,
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
                {
                    let entries = build_issues_entries(delegate.all_pages());
                    if let Some(entry) = entries.get(*index)
                        && let Some((tab, filter)) = overview_issue_target(&entry.name)
                    {
                        cx.emit(ResultsGridEvent::OverviewDrillDown { tab, filter });
                        return;
                    }
                }
                cx.emit(ResultsGridEvent::Selected(*row_ix))
            }
        });
        Self {
            state,
            _subscription: sub,
        }
    }

    pub fn push(&mut self, record: PageRecord, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().push(record);
            state.refresh(cx);
        });
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

    pub fn set_baseline(
        &mut self,
        pages: Vec<PageRecord>,
        started_at: i64,
        cx: &mut Context<Self>,
    ) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().set_baseline(pages, started_at);
            state.refresh(cx);
        });
        cx.notify();
    }

    pub fn clear_baseline(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.delegate_mut().clear_baseline();
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

    pub fn record_at(&self, index: usize, cx: &App) -> Option<PageRecord> {
        self.state.read(cx).delegate().record_at(index).cloned()
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
        div()
            .flex_1()
            .size_full()
            .min_h_0()
            .bg(cx.theme().background)
            .child(DataTable::new(&self.state).bordered(false).stripe(true))
    }
}
