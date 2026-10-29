//! The bar under a tab's sub-filters that narrows its rows with a search and
//! column conditions. Each tab has its own, kept while other tabs are open.

use std::time::Duration;

use gpui_kit::component::{
    ActiveTheme as _, IconName, IndexPath, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::{
    App, AppContext as _, Context, Entity, EventEmitter, InteractiveElement as _, IntoElement,
    KeyBinding, ParentElement as _, Render, SharedString, Styled as _, Subscription, Task, Window,
    actions, div, px,
};

use crate::ui::icon::Icon;
use crate::ui::tag::group_digits;
use crate::views::ResultsGrid;
use crate::views::results_grid::{FilterCondition, FilterJoin, FilterOperator, RowFilter};

const CONTEXT: &str = "RowFilterBar";

/// Typing waits this long before narrowing the rows. A search over the
/// largest tabs of a 400k row crawl takes about a tenth of a second, which
/// would otherwise be paid on every key.
const TYPING_DELAY: Duration = Duration::from_millis(150);

actions!(
    row_filter,
    [
        /// Open the filter bar of the open tab and focus its search.
        FindInResults,
        /// Clear and close the filter bar.
        CloseRowFilter,
    ]
);

/// Narrow the open tab to the rows whose cell in `column` shows `value`, or
/// to every other row when `exclude` is set.
#[derive(Clone, PartialEq, gpui_kit::Action)]
#[action(namespace = row_filter, no_json)]
pub struct FilterByValue {
    pub column: SharedString,
    pub value: String,
    pub exclude: bool,
}

pub fn key_bindings() -> Vec<KeyBinding> {
    vec![
        #[cfg(target_os = "macos")]
        KeyBinding::new("cmd-f", FindInResults, None),
        #[cfg(not(target_os = "macos"))]
        KeyBinding::new("ctrl-f", FindInResults, None),
        KeyBinding::new("escape", CloseRowFilter, Some(CONTEXT)),
    ]
}

pub enum FilterBarEvent {
    Changed(RowFilter),
    Close,
}

#[derive(Clone, Debug, PartialEq)]
struct FilterColumn {
    key: SharedString,
    name: SharedString,
}

struct ConditionRow {
    column: Entity<SelectState<SearchableVec<SharedString>>>,
    operator: Entity<SelectState<Vec<SharedString>>>,
    value: Entity<InputState>,
}

pub struct FilterBar {
    grid: Entity<ResultsGrid>,
    columns: Vec<FilterColumn>,
    search: Entity<InputState>,
    join: FilterJoin,
    rows: Vec<ConditionRow>,
    /// Dropping it cancels a change still waiting out [`TYPING_DELAY`].
    pending_change: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<FilterBarEvent> for FilterBar {}

impl FilterBar {
    pub fn new(grid: Entity<ResultsGrid>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search the address, title and every text column")
        });
        let columns = column_list(&grid, cx);
        let subscriptions = vec![
            cx.subscribe_in(&search, window, Self::on_input_event),
            // The row counts the bar shows change with the grid's rows.
            cx.observe(&grid, |_, _, cx| cx.notify()),
        ];
        Self {
            grid,
            columns,
            search,
            join: FilterJoin::All,
            rows: Vec::new(),
            pending_change: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.focus(window, cx));
    }

    /// Picks up columns the tab gained or lost, such as the comparison
    /// columns a baseline adds. A condition keeps its column by key.
    pub fn sync_columns(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let columns = column_list(&self.grid, cx);
        if columns == self.columns {
            return;
        }
        let keys: Vec<Option<SharedString>> = self
            .rows
            .iter()
            .map(|row| self.row_column(row, cx))
            .collect();
        self.columns = columns;
        let names = self.column_names();
        for (row, key) in self.rows.iter().zip(keys) {
            let index = key.and_then(|key| self.column_index(&key));
            row.column.update(cx, |select, cx| {
                select.set_items(names.clone(), window, cx);
                select.set_selected_index(index.map(|ix| IndexPath::default().row(ix)), window, cx);
            });
        }
    }

    /// Adds a condition row, and applies it at once when it is complete.
    pub fn add_condition(
        &mut self,
        column: Option<&str>,
        operator: FilterOperator,
        value: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let column_index =
            column
                .and_then(|key| self.column_index(key))
                .or(if self.columns.is_empty() {
                    None
                } else {
                    Some(0)
                });
        let column = cx.new(|cx| {
            SelectState::new(
                self.column_names(),
                column_index.map(|ix| IndexPath::default().row(ix)),
                window,
                cx,
            )
            .searchable(true)
        });
        let operators: Vec<SharedString> = FilterOperator::ALL
            .iter()
            .map(|operator| SharedString::from(operator.label()))
            .collect();
        let operator_index = FilterOperator::ALL
            .iter()
            .position(|candidate| *candidate == operator);
        let operator = cx.new(|cx| {
            SelectState::new(
                operators,
                operator_index.map(|ix| IndexPath::default().row(ix)),
                window,
                cx,
            )
        });
        let value = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Value")
                .default_value(value.to_string())
        });
        self._subscriptions.extend([
            cx.subscribe_in(&value, window, Self::on_input_event),
            cx.subscribe(&column, |bar, _, _: &SelectEvent<_>, cx| {
                bar.emit_change(cx)
            }),
            cx.subscribe(&operator, |bar, _, _: &SelectEvent<_>, cx| {
                bar.emit_change(cx)
            }),
        ]);
        self.rows.push(ConditionRow {
            column,
            operator,
            value,
        });
        self.emit_change(cx);
    }

    fn remove_condition(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.rows.len() {
            self.rows.remove(index);
            self.emit_change(cx);
        }
    }

    pub fn filter(&self, cx: &App) -> RowFilter {
        RowFilter {
            search: self.search.read(cx).value().to_string(),
            join: self.join,
            conditions: self
                .rows
                .iter()
                .filter_map(|row| {
                    Some(FilterCondition {
                        column: self.row_column(row, cx)?,
                        operator: row_operator(row, cx)?,
                        value: row.value.read(cx).value().to_string(),
                    })
                })
                .collect(),
        }
    }

    fn row_column(&self, row: &ConditionRow, cx: &App) -> Option<SharedString> {
        let name = row.column.read(cx).selected_value()?;
        self.columns
            .iter()
            .find(|column| &column.name == name)
            .map(|column| column.key.clone())
    }

    fn column_index(&self, key: &str) -> Option<usize> {
        self.columns.iter().position(|column| column.key == key)
    }

    fn column_names(&self) -> SearchableVec<SharedString> {
        SearchableVec::new(
            self.columns
                .iter()
                .map(|column| column.name.clone())
                .collect::<Vec<_>>(),
        )
    }

    fn on_input_event(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            InputEvent::Change => {
                self.pending_change = Some(cx.spawn(async move |bar, cx| {
                    cx.background_executor().timer(TYPING_DELAY).await;
                    if let Err(error) = bar.update(cx, |bar, cx| bar.emit_change(cx)) {
                        tracing::debug!(%error, "the filter bar closed before applying");
                    }
                }));
            }
            InputEvent::PressEnter { .. } => self.emit_change(cx),
            InputEvent::Focus | InputEvent::Blur => {}
        }
    }

    fn emit_change(&mut self, cx: &mut Context<Self>) {
        self.pending_change = None;
        cx.emit(FilterBarEvent::Changed(self.filter(cx)));
        cx.notify();
    }

    fn on_close(&mut self, _: &CloseRowFilter, _: &mut Window, cx: &mut Context<Self>) {
        cx.emit(FilterBarEvent::Close);
    }

    fn render_condition_row(
        &self,
        index: usize,
        row: &ConditionRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let takes_value = row_operator(row, cx).is_none_or(FilterOperator::takes_value);
        let join = match (index, self.join) {
            (0, _) => "Where",
            (_, FilterJoin::All) => "and",
            (_, FilterJoin::Any) => "or",
        };
        h_flex()
            .gap_1()
            .items_center()
            .child(
                div()
                    .w(px(44.))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(join),
            )
            .child(
                div().w(px(200.)).child(
                    Select::new(&row.column)
                        .xsmall()
                        .search_placeholder("Search columns")
                        .menu_width(px(260.)),
                ),
            )
            .child(
                div()
                    .w(px(140.))
                    .child(Select::new(&row.operator).xsmall().menu_width(px(180.))),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(120.))
                    .child(Input::new(&row.value).xsmall().disabled(!takes_value)),
            )
            .child(
                Button::new(("remove-row-filter-condition", index))
                    .ghost()
                    .xsmall()
                    .icon(IconName::Minus)
                    .tooltip("Remove Condition")
                    .on_click(cx.listener(move |bar, _, _, cx| bar.remove_condition(index, cx))),
            )
    }
}

fn column_list(grid: &Entity<ResultsGrid>, cx: &App) -> Vec<FilterColumn> {
    grid.read(cx)
        .columns(cx)
        .into_iter()
        .map(|(key, name)| FilterColumn { key, name })
        .collect()
}

fn row_operator(row: &ConditionRow, cx: &App) -> Option<FilterOperator> {
    let label = row.operator.read(cx).selected_value()?;
    FilterOperator::ALL
        .into_iter()
        .find(|operator| operator.label() == label.as_ref())
}

impl Render for FilterBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let grid = self.grid.read(cx);
        let shown = grid.row_count(cx);
        let total = grid.rows_before_row_filter(cx);
        let muted = cx.theme().muted_foreground;
        let join = self.join;
        let rows: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .map(|(index, row)| self.render_condition_row(index, row, cx))
            .collect();

        v_flex()
            .id("row-filter-bar")
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::on_close))
            .gap_1()
            .px(crate::app::PANEL_GAP)
            .pb(crate::app::PANEL_GAP)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div().flex_1().min_w_0().child(
                            Input::new(&self.search).xsmall().cleanable(true).prefix(
                                gpui_kit::component::Icon::from(Icon::Search)
                                    .xsmall()
                                    .text_color(muted),
                            ),
                        ),
                    )
                    .when(self.rows.len() > 1, |this| {
                        this.child(
                            ButtonGroup::new("row-filter-join")
                                .outline()
                                .xsmall()
                                .child(
                                    Button::new("row-filter-join-all")
                                        .label("Match All")
                                        .selected(join == FilterJoin::All),
                                )
                                .child(
                                    Button::new("row-filter-join-any")
                                        .label("Match Any")
                                        .selected(join == FilterJoin::Any),
                                )
                                .on_click(cx.listener(|bar, selected: &Vec<usize>, _, cx| {
                                    bar.join = match selected.first() {
                                        Some(1) => FilterJoin::Any,
                                        _ => FilterJoin::All,
                                    };
                                    bar.emit_change(cx);
                                })),
                        )
                    })
                    .child(
                        Button::new("add-row-filter-condition")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Plus)
                            .label("Add Condition")
                            .on_click(cx.listener(|bar, _, window, cx| {
                                bar.add_condition(None, FilterOperator::Contains, "", window, cx);
                            })),
                    )
                    .child(div().flex_shrink_0().text_xs().text_color(muted).child(
                        if shown == total {
                            format!("{} rows", group_digits(total))
                        } else {
                            format!("{} of {} rows", group_digits(shown), group_digits(total))
                        },
                    ))
                    .child(
                        Button::new("close-row-filter")
                            .ghost()
                            .xsmall()
                            .icon(IconName::Close)
                            .tooltip("Clear and Close the Filter")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(FilterBarEvent::Close))),
                    ),
            )
            .children(rows)
    }
}
