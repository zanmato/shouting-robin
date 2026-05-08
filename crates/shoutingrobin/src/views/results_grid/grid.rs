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
                    && let Some(FlatRow::OverviewIssue { label, .. }) =
                        delegate.flat_rows().get(*row_ix)
                    && let Some((tab, filter)) = overview_issue_target(label)
                {
                    cx.emit(ResultsGridEvent::OverviewDrillDown { tab, filter });
                    return;
                }
                if delegate.active_tab == ResultTab::Issues
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

    pub fn record_at(&self, index: usize, cx: &App) -> Option<PageRecord> {
        self.state.read(cx).delegate().record_at(index).cloned()
    }

    pub fn row_count(&self, cx: &App) -> usize {
        self.state.read(cx).delegate().filtered_count()
    }

    pub fn tab_counts(&self, cx: &App) -> HashMap<ResultTab, TabCounts> {
        self.state.read(cx).delegate().compute_tab_counts()
    }

    pub fn count_for_filter(&self, filter: IssueFilter, cx: &App) -> usize {
        self.state.read(cx).delegate().count_for_filter(filter)
    }

    #[allow(dead_code)]
    pub fn active_tab(&self, cx: &App) -> ResultTab {
        self.state.read(cx).delegate().active_tab()
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
