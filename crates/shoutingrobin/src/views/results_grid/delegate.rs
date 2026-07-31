use std::collections::HashMap;

use gpui::{
    App, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, Icon as UiIcon, Sizable, StyledExt as _,
    table::{ColumnSort, TableDelegate, TableState},
    tooltip::Tooltip,
};

use crate::a11y_rules::rule_description;
use crate::crawl::engine::is_same_domain;
use crate::crawl::event::PageRecord;
use crate::ui::icon::Icon;
use crate::ui::tag::{Tone, tone_tag, tone_text_color};
use crate::views::ResultTab;

use super::cell::{cell_text, flat_cell_text, render_cell_tag, url_to_path};
use super::columns::{
    build_occurrence_counts, columns_for_tab_with_baseline, compare_numeric, is_mono_column,
    is_numeric_column,
};
use super::data_build::{
    build_change_entries, build_issues_entries, build_rows_for_tab, change_entry_matches,
    dir_format_size, issue_entry_matches,
};
use super::filter::{compute_tab_filter_counts, filter_for_tab, flat_row_matches_filter};
use super::types::{
    ChangeEntry, ChangeKind, FlatRow, IssueFilter, TabFilterCounts, tab_is_flattened,
};

fn format_delta(delta: i64) -> String {
    match delta.cmp(&0) {
        std::cmp::Ordering::Greater => format!("+{delta}"),
        _ => delta.to_string(),
    }
}

/// Colors an issue-count delta: a rising issue count is bad (red), a falling one
/// is good (green), unchanged is neutral.
fn delta_tone(delta: i64) -> Tone {
    match delta.cmp(&0) {
        std::cmp::Ordering::Greater => Tone::Err,
        std::cmp::Ordering::Less => Tone::Ok,
        std::cmp::Ordering::Equal => Tone::Neutral,
    }
}

pub struct ResultsDelegate {
    pub(super) all_pages: Vec<PageRecord>,
    filtered_indices: Vec<usize>,
    flat_rows: Vec<FlatRow>,
    occurrence_counts: HashMap<String, usize>,
    columns: Vec<gpui_component::table::Column>,
    pub(super) active_tab: ResultTab,
    issue_filter: IssueFilter,
    pub(super) root_origin: Option<String>,
    root_url: Option<String>,
    baseline_pages: Option<Vec<PageRecord>>,
    baseline_started_at: Option<i64>,
    baseline_issue_counts: HashMap<String, (usize, f32)>,
    /// Lazily computed per-tab badge and sub-filter counts, keyed by tab.
    /// Invalidated whenever the underlying data changes (pages, baseline, root),
    /// but not on tab/filter switches since the counts span all tabs.
    counts_cache: Option<HashMap<ResultTab, TabFilterCounts>>,
}

impl ResultsDelegate {
    pub fn new() -> Self {
        let tab = ResultTab::Internal;
        Self {
            all_pages: Vec::new(),
            filtered_indices: Vec::new(),
            flat_rows: Vec::new(),
            occurrence_counts: HashMap::new(),
            columns: columns_for_tab_with_baseline(tab, false),
            active_tab: tab,
            issue_filter: IssueFilter::All,
            root_origin: None,
            root_url: None,
            baseline_pages: None,
            baseline_started_at: None,
            baseline_issue_counts: HashMap::new(),
            counts_cache: None,
        }
    }

    /// Drops the cached counts so the next `tab_filter_counts` call recomputes
    /// them. Called from every mutation that changes the counted data.
    fn invalidate_counts(&mut self) {
        self.counts_cache = None;
    }

    /// Lazily computes and caches per-tab badge and sub-filter counts for every
    /// tab. The badge and the sub-filter buttons both read from here so they can
    /// never disagree.
    pub fn tab_filter_counts(&mut self) -> &HashMap<ResultTab, TabFilterCounts> {
        if self.counts_cache.is_none() {
            let change_entries = self.change_entries();
            let root_origin = self.root_origin.clone();
            let mut counts = HashMap::with_capacity(ResultTab::ALL.len());
            for &tab in ResultTab::ALL {
                counts.insert(
                    tab,
                    compute_tab_filter_counts(
                        tab,
                        &self.all_pages,
                        &change_entries,
                        root_origin.as_deref(),
                    ),
                );
            }
            self.counts_cache = Some(counts);
        }
        self.counts_cache
            .as_ref()
            .expect("counts_cache populated above")
    }

    fn rebuild_columns(&mut self) {
        self.columns =
            columns_for_tab_with_baseline(self.active_tab, self.baseline_pages.is_some());
    }

    pub fn switch_tab(&mut self, tab: ResultTab) {
        self.active_tab = tab;
        self.issue_filter = IssueFilter::All;
        self.rebuild_columns();
        self.rebuild_filter();
    }

    pub fn set_baseline(&mut self, pages: Vec<PageRecord>, started_at: i64) {
        self.baseline_issue_counts = build_issues_entries(&pages)
            .into_iter()
            .map(|entry| (entry.name, (entry.count, entry.pct)))
            .collect();
        self.baseline_pages = Some(pages);
        self.baseline_started_at = Some(started_at);
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    pub fn clear_baseline(&mut self) {
        self.baseline_pages = None;
        self.baseline_started_at = None;
        self.baseline_issue_counts.clear();
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    pub fn has_baseline(&self) -> bool {
        self.baseline_pages.is_some()
    }

    pub fn baseline_started_at(&self) -> Option<i64> {
        self.baseline_started_at
    }

    fn change_entries(&self) -> Vec<ChangeEntry> {
        match &self.baseline_pages {
            Some(baseline) => build_change_entries(&self.all_pages, baseline),
            None => Vec::new(),
        }
    }

    pub fn set_issue_filter(&mut self, filter: IssueFilter) {
        self.issue_filter = filter;
        self.rebuild_filter();
    }

    pub fn set_root_url(&mut self, root_url: &str) {
        self.root_url = Some(root_url.to_owned());
        self.root_origin = url::Url::parse(root_url)
            .ok()
            .map(|u| u.origin().ascii_serialization());
        // Directory aggregates on the Site Structure tab key off the origin.
        self.invalidate_counts();
    }

    pub fn root_url(&self) -> Option<&str> {
        self.root_url.as_deref()
    }

    pub fn push(&mut self, record: PageRecord) {
        self.all_pages.push(record);
        self.invalidate_counts();
        self.rebuild_filter();
    }

    pub fn clear(&mut self) {
        self.all_pages.clear();
        self.filtered_indices.clear();
        self.flat_rows.clear();
        self.occurrence_counts.clear();
        self.root_origin = None;
        self.root_url = None;
        self.baseline_pages = None;
        self.baseline_started_at = None;
        self.baseline_issue_counts.clear();
        self.invalidate_counts();
        self.rebuild_columns();
    }

    pub fn record_at(&self, index: usize) -> Option<&PageRecord> {
        if tab_is_flattened(self.active_tab) {
            match self.flat_rows.get(index)? {
                FlatRow::Image { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. }
                | FlatRow::LinkRow { page, .. } => self.all_pages.get(*page),
                FlatRow::ChangeRow { index } => {
                    let entries = self.change_entries();
                    let entry = entries.get(*index)?;
                    let url = entry.url.clone();
                    match entry.kind {
                        ChangeKind::Removed => self
                            .baseline_pages
                            .as_ref()
                            .and_then(|baseline| baseline.iter().find(|p| p.url == url)),
                        _ => self.all_pages.iter().find(|p| p.url == url),
                    }
                }
                FlatRow::IssuesRow { .. } | FlatRow::DirectoryAggregate { .. } => None,
            }
        } else {
            self.filtered_indices
                .get(index)
                .and_then(|&idx| self.all_pages.get(idx))
        }
    }

    pub fn filtered_count(&self) -> usize {
        if tab_is_flattened(self.active_tab) {
            self.flat_rows.len()
        } else {
            self.filtered_indices.len()
        }
    }

    #[allow(dead_code)]
    pub fn active_tab(&self) -> ResultTab {
        self.active_tab
    }

    pub fn flat_rows(&self) -> &[FlatRow] {
        &self.flat_rows
    }

    pub fn all_pages(&self) -> &[PageRecord] {
        &self.all_pages
    }

    fn rebuild_filter(&mut self) {
        self.occurrence_counts = build_occurrence_counts(self.active_tab, &self.all_pages);
        self.filtered_indices = filter_for_tab(
            self.active_tab,
            self.issue_filter,
            &self.all_pages,
            &self.occurrence_counts,
        );
        self.rebuild_flat_rows();
    }

    fn rebuild_flat_rows(&mut self) {
        if !tab_is_flattened(self.active_tab) {
            self.flat_rows.clear();
            return;
        }
        let change_entries = self.change_entries();
        self.flat_rows = build_rows_for_tab(
            self.active_tab,
            &self.filtered_indices,
            &self.all_pages,
            &change_entries,
            self.root_origin.as_deref(),
        );
        self.filter_flat_rows();
    }

    fn filter_flat_rows(&mut self) {
        if self.issue_filter == IssueFilter::All {
            return;
        }
        if self.active_tab == ResultTab::Overview {
            let entries = build_issues_entries(&self.all_pages);
            let filter = self.issue_filter;
            self.flat_rows.retain(|row| {
                let FlatRow::IssuesRow { index } = row else {
                    return true;
                };
                let Some(entry) = entries.get(*index) else {
                    return false;
                };
                issue_entry_matches(entry, filter)
            });
            return;
        }
        if self.active_tab == ResultTab::Changes {
            let entries = self.change_entries();
            let filter = self.issue_filter;
            self.flat_rows.retain(|row| {
                let FlatRow::ChangeRow { index } = row else {
                    return true;
                };
                let Some(entry) = entries.get(*index) else {
                    return false;
                };
                change_entry_matches(entry, filter)
            });
            return;
        }
        if self.active_tab == ResultTab::SiteStructure {
            self.flat_rows.retain(|row| {
                let FlatRow::DirectoryAggregate { depth, .. } = row else {
                    return true;
                };
                match self.issue_filter {
                    IssueFilter::DepthShallow => *depth <= 1,
                    IssueFilter::DepthMedium => *depth >= 2 && *depth <= 3,
                    IssueFilter::DepthDeep => *depth >= 4,
                    _ => true,
                }
            });
            return;
        }
        self.flat_rows.retain(|row| {
            let page_index = match row {
                FlatRow::Image { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. }
                | FlatRow::LinkRow { page, .. } => *page,
                FlatRow::IssuesRow { .. }
                | FlatRow::ChangeRow { .. }
                | FlatRow::DirectoryAggregate { .. } => return true,
            };
            let Some(page) = self.all_pages.get(page_index) else {
                return false;
            };
            flat_row_matches_filter(row, page, self.issue_filter, &self.all_pages)
        });
    }

    pub fn export_csv(&self) -> Result<String, csv::Error> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        let headers: Vec<&str> = self.columns.iter().map(|c| c.name.as_ref()).collect();
        wtr.write_record(&headers)?;

        let row_count = if tab_is_flattened(self.active_tab) {
            self.flat_rows.len()
        } else {
            self.filtered_indices.len()
        };

        for row_ix in 0..row_count {
            let mut cells: Vec<String> = Vec::with_capacity(self.columns.len());
            for col in &self.columns {
                let key = col.key.as_ref();
                let text = if tab_is_flattened(self.active_tab) {
                    let Some(row) = self.flat_rows.get(row_ix) else {
                        cells.push(String::new());
                        continue;
                    };
                    self.flat_row_cell_text(row, key)
                } else {
                    let Some(record) = self
                        .filtered_indices
                        .get(row_ix)
                        .and_then(|&idx| self.all_pages.get(idx))
                    else {
                        cells.push(String::new());
                        continue;
                    };
                    cell_text(
                        record,
                        key,
                        &self.occurrence_counts,
                        self.active_tab,
                        self.root_origin.as_deref(),
                    )
                    .into()
                };
                cells.push(text);
            }
            wtr.write_record(&cells)?;
        }

        let bytes = wtr
            .into_inner()
            .map_err(|e| csv::Error::from(std::io::Error::other(e)))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    fn flat_row_cell_text(&self, row: &FlatRow, col_key: &str) -> String {
        match row {
            FlatRow::IssuesRow { index } => {
                let entries = build_issues_entries(&self.all_pages);
                let Some(entry) = entries.get(*index) else {
                    return String::new();
                };
                match col_key {
                    "issue_name" => entry.name.clone(),
                    "issue_type" => entry.issue_type.label().to_string(),
                    "priority" => entry.priority.label().to_string(),
                    "count" => entry.count.to_string(),
                    "pct" => format!("{:.1}%", entry.pct),
                    "count_prev" => self
                        .baseline_issue_counts
                        .get(&entry.name)
                        .map(|(count, _)| count.to_string())
                        .unwrap_or_else(|| "-".into()),
                    "count_delta" => {
                        let previous = self
                            .baseline_issue_counts
                            .get(&entry.name)
                            .map(|(count, _)| *count)
                            .unwrap_or(0);
                        format_delta(entry.count as i64 - previous as i64)
                    }
                    _ => String::new(),
                }
            }
            FlatRow::ChangeRow { index } => {
                let entries = self.change_entries();
                let Some(entry) = entries.get(*index) else {
                    return String::new();
                };
                match col_key {
                    "change_url" => url_to_path(&entry.url, self.root_origin.as_deref()).into(),
                    "change_kind" => entry.kind.label().to_string(),
                    "change_status" => entry.status_text(),
                    "change_detail" => entry.detail_text(),
                    _ => String::new(),
                }
            }
            FlatRow::DirectoryAggregate {
                path,
                depth,
                page_count,
                avg_word_count,
                total_size,
                non_indexable,
                indexable,
                ..
            } => match col_key {
                "dir_path" => path.clone(),
                "dir_page_count" => page_count.to_string(),
                "dir_depth" => depth.to_string(),
                "dir_avg_words" => avg_word_count.to_string(),
                "dir_total_size" => dir_format_size(*total_size),
                "dir_indexable" => indexable.to_string(),
                "dir_non_indexable" => non_indexable.to_string(),
                _ => String::new(),
            },
            FlatRow::LinkRow { page, item } => {
                let Some(record) = self.all_pages.get(*page) else {
                    return String::new();
                };
                let Some(link) = record.outlinks.get(*item) else {
                    return String::new();
                };
                match col_key {
                    "source" => url_to_path(&record.url, self.root_origin.as_deref()).into(),
                    "destination" => url_to_path(&link.dst_url, self.root_origin.as_deref()).into(),
                    "anchor" => link.anchor.clone().unwrap_or_default(),
                    "rel" => link.rel.clone().unwrap_or_default(),
                    "status_code" => "-".to_string(),
                    "link_type" => {
                        if is_same_domain(&record.url, &link.dst_url) {
                            "Internal".to_string()
                        } else {
                            "External".to_string()
                        }
                    }
                    _ => String::new(),
                }
            }
            _ => {
                let page_index = match row {
                    FlatRow::Image { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. } => *page,
                    _ => return String::new(),
                };
                let Some(record) = self.all_pages.get(page_index) else {
                    return String::new();
                };
                flat_cell_text(record, row, col_key, self.root_origin.as_deref()).into()
            }
        }
    }
}

impl TableDelegate for ResultsDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        if tab_is_flattened(self.active_tab) {
            self.flat_rows.len()
        } else {
            self.filtered_indices.len()
        }
    }

    fn column(&self, col_ix: usize, _: &App) -> gpui_component::table::Column {
        self.columns.get(col_ix).cloned().unwrap_or_default()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let column = self.column(col_ix, cx);
        div()
            .flex()
            .size_full()
            .items_center()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(column.name)
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let key = self
            .columns
            .get(col_ix)
            .map(|c| c.key.clone())
            .unwrap_or_default();
        let mono = is_mono_column(&key);
        // `h_full` is what makes `items_center` do anything: the table's own td
        // wrapper is the element that carries the row height, so without it this
        // div shrinks to its content and everything sits at the top of the cell.
        let mut cell = div().flex().h_full().items_center().text_xs();
        if mono {
            cell = cell.font_family(cx.theme().mono_font_family.clone());
        }

        if tab_is_flattened(self.active_tab) {
            let Some(row) = self.flat_rows.get(row_ix) else {
                return cell;
            };
            match row {
                FlatRow::IssuesRow { index } => {
                    let entries = build_issues_entries(&self.all_pages);
                    let Some(entry) = entries.get(*index) else {
                        return cell;
                    };
                    let previous = self.baseline_issue_counts.get(&entry.name).map(|(c, _)| *c);
                    let delta = entry.count as i64 - previous.unwrap_or(0) as i64;
                    let text = match key.as_ref() {
                        "issue_name" => SharedString::from(entry.name.clone()),
                        "issue_type" => SharedString::from(entry.issue_type.label()),
                        "priority" => SharedString::from(entry.priority.label()),
                        "count" => SharedString::from(entry.count.to_string()),
                        "pct" => SharedString::from(format!("{:.1}%", entry.pct)),
                        "count_prev" => SharedString::from(
                            previous
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ),
                        "count_delta" => SharedString::from(format_delta(delta)),
                        "description" => SharedString::from(entry.description.clone()),
                        "hint" => SharedString::from(entry.hint.clone()),
                        _ => SharedString::default(),
                    };
                    match key.as_ref() {
                        "issue_type" => {
                            cell.child(tone_tag(entry.issue_type.tone(), cx).child(text))
                        }
                        "priority" => cell.child(tone_tag(entry.priority.tone(), cx).child(text)),
                        "count" => {
                            let tone = if entry.count > 0 {
                                Tone::Warn
                            } else {
                                Tone::Ok
                            };
                            cell.child(tone_tag(tone, cx).child(text))
                        }
                        "count_delta" => cell
                            .text_color(tone_text_color(delta_tone(delta), cx))
                            .font_semibold()
                            .child(text),
                        _ => cell.child(text),
                    }
                }
                FlatRow::ChangeRow { index } => {
                    let entries = self.change_entries();
                    let Some(entry) = entries.get(*index) else {
                        return cell;
                    };
                    let text = match key.as_ref() {
                        "change_url" => url_to_path(&entry.url, self.root_origin.as_deref()),
                        "change_kind" => SharedString::from(entry.kind.label()),
                        "change_status" => SharedString::from(entry.status_text()),
                        "change_detail" => SharedString::from(entry.detail_text()),
                        _ => SharedString::default(),
                    };
                    match key.as_ref() {
                        "change_kind" => cell.child(tone_tag(entry.kind.tone(), cx).child(text)),
                        _ => cell.child(text),
                    }
                }
                FlatRow::LinkRow { page, item } => {
                    let Some(record) = self.all_pages.get(*page) else {
                        return cell;
                    };
                    let Some(link) = record.outlinks.get(*item) else {
                        return cell;
                    };
                    let text = match key.as_ref() {
                        "source" => url_to_path(&record.url, self.root_origin.as_deref()),
                        "destination" => url_to_path(&link.dst_url, self.root_origin.as_deref()),
                        "anchor" => SharedString::from(link.anchor.clone().unwrap_or_default()),
                        "rel" => SharedString::from(link.rel.clone().unwrap_or_default()),
                        "status_code" => SharedString::from("-"),
                        "link_type" => {
                            if is_same_domain(&record.url, &link.dst_url) {
                                SharedString::from("Internal")
                            } else {
                                SharedString::from("External")
                            }
                        }
                        _ => SharedString::default(),
                    };
                    match key.as_ref() {
                        "link_type" => {
                            let tone = if is_same_domain(&record.url, &link.dst_url) {
                                Tone::Neutral
                            } else {
                                Tone::Warn
                            };
                            cell.child(tone_tag(tone, cx).child(text))
                        }
                        "source" => cell.child(text),
                        "destination" => {
                            let url = link.dst_url.clone();
                            cell.child(
                                div()
                                    .group("url-cell-dst")
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(text)
                                    .child(
                                        div()
                                            .invisible()
                                            .group_hover("url-cell-dst", |s| s.visible()),
                                    )
                                    .child(
                                        div()
                                            .id(("open-dst", row_ix))
                                            .cursor_pointer()
                                            .child(UiIcon::from(Icon::ExternalLink).xsmall())
                                            .on_click(move |_, _window, cx| {
                                                cx.open_url(&url);
                                            }),
                                    ),
                            )
                        }
                        _ => cell.child(text),
                    }
                }
                FlatRow::DirectoryAggregate {
                    path,
                    depth,
                    page_count,
                    avg_word_count,
                    total_size,
                    non_indexable,
                    indexable,
                    ..
                } => {
                    let text = match key.as_ref() {
                        "dir_path" => SharedString::from(path.clone()),
                        "dir_page_count" => SharedString::from(page_count.to_string()),
                        "dir_depth" => SharedString::from(depth.to_string()),
                        "dir_avg_words" => SharedString::from(avg_word_count.to_string()),
                        "dir_total_size" => SharedString::from(dir_format_size(*total_size)),
                        "dir_indexable" => SharedString::from(indexable.to_string()),
                        "dir_non_indexable" => SharedString::from(non_indexable.to_string()),
                        _ => SharedString::default(),
                    };
                    match key.as_ref() {
                        "dir_non_indexable" if *non_indexable > 0 => {
                            cell.child(tone_tag(Tone::Warn, cx).child(text))
                        }
                        _ => cell.child(text),
                    }
                }
                FlatRow::A11yIssue { page, item } => {
                    let Some(record) = self.all_pages.get(*page) else {
                        return cell;
                    };
                    let Some(issue) = record.a11y_issues.get(*item) else {
                        return cell;
                    };
                    let text = flat_cell_text(record, row, &key, self.root_origin.as_deref());
                    if key.as_ref() == "a11y_rule" {
                        if let Some(desc) = rule_description(&issue.rule) {
                            let desc = SharedString::from(desc.to_string());
                            cell.child(div().id(("a11y-rule-tip", row_ix)).child(text).tooltip(
                                move |window, cx| Tooltip::new(desc.clone()).build(window, cx),
                            ))
                        } else {
                            cell.child(text)
                        }
                    } else if let Some(tag) = render_cell_tag(record, &key, &text, cx) {
                        cell.child(tag)
                    } else {
                        cell.child(text)
                    }
                }
                _ => {
                    let page_index = match row {
                        FlatRow::Image { page, .. }
                        | FlatRow::Hreflang { page, .. }
                        | FlatRow::SdItem { page, .. } => *page,
                        FlatRow::IssuesRow { .. }
                        | FlatRow::ChangeRow { .. }
                        | FlatRow::LinkRow { .. }
                        | FlatRow::DirectoryAggregate { .. }
                        | FlatRow::A11yIssue { .. } => unreachable!(),
                    };
                    let Some(record) = self.all_pages.get(page_index) else {
                        return cell;
                    };
                    let text = flat_cell_text(record, row, &key, self.root_origin.as_deref());
                    if let Some(tag) = render_cell_tag(record, &key, &text, cx) {
                        cell.child(tag)
                    } else {
                        cell.child(text)
                    }
                }
            }
        } else {
            let Some(record) = self
                .filtered_indices
                .get(row_ix)
                .and_then(|&idx| self.all_pages.get(idx))
            else {
                return cell;
            };
            let text = cell_text(
                record,
                &key,
                &self.occurrence_counts,
                self.active_tab,
                self.root_origin.as_deref(),
            );
            if let Some(tag) = render_cell_tag(record, &key, &text, cx) {
                cell.child(tag)
            } else {
                cell.child(text)
            }
        }
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
        let Some(col) = self.columns.get(col_ix) else {
            return;
        };
        // Default means "no sort": restore the tab's natural row order rather
        // than re-sorting ascending, which would differ from what a fresh
        // filter rebuild produces.
        if sort == ColumnSort::Default {
            self.rebuild_filter();
            return;
        }
        let col_key = col.key.to_string();
        let numeric = is_numeric_column(&col_key);
        let root_origin = self.root_origin.clone();

        if tab_is_flattened(self.active_tab) {
            // Precompute outside the closure to avoid borrowing all of `self`
            // (a method call) while `self.flat_rows` is borrowed mutably.
            let change_entries = if self.active_tab == ResultTab::Changes {
                self.change_entries()
            } else {
                Vec::new()
            };
            self.flat_rows.sort_by(|a, b| {
                if let (FlatRow::IssuesRow { index: a_idx }, FlatRow::IssuesRow { index: b_idx }) =
                    (a, b)
                {
                    let entries = build_issues_entries(&self.all_pages);
                    let a_entry = entries.get(*a_idx);
                    let b_entry = entries.get(*b_idx);
                    let ordering = match (a_entry, b_entry) {
                        (Some(ae), Some(be)) => match col_key.as_ref() {
                            "count" => ae.count.cmp(&be.count),
                            "pct" => ae
                                .pct
                                .partial_cmp(&be.pct)
                                .unwrap_or(std::cmp::Ordering::Equal),
                            "priority" => ae.priority.cmp(&be.priority),
                            "issue_type" => ae.issue_type.cmp(&be.issue_type),
                            "description" => ae.description.cmp(&be.description),
                            "hint" => ae.hint.cmp(&be.hint),
                            "count_prev" => {
                                let ap = self
                                    .baseline_issue_counts
                                    .get(&ae.name)
                                    .map(|(c, _)| *c)
                                    .unwrap_or(0);
                                let bp = self
                                    .baseline_issue_counts
                                    .get(&be.name)
                                    .map(|(c, _)| *c)
                                    .unwrap_or(0);
                                ap.cmp(&bp)
                            }
                            "count_delta" => {
                                let ad = ae.count as i64
                                    - self
                                        .baseline_issue_counts
                                        .get(&ae.name)
                                        .map(|(c, _)| *c as i64)
                                        .unwrap_or(0);
                                let bd = be.count as i64
                                    - self
                                        .baseline_issue_counts
                                        .get(&be.name)
                                        .map(|(c, _)| *c as i64)
                                        .unwrap_or(0);
                                ad.cmp(&bd)
                            }
                            _ => ae.name.cmp(&be.name),
                        },
                        _ => std::cmp::Ordering::Equal,
                    };
                    return match sort {
                        ColumnSort::Descending => ordering.reverse(),
                        _ => ordering,
                    };
                }
                if let (FlatRow::ChangeRow { index: a_idx }, FlatRow::ChangeRow { index: b_idx }) =
                    (a, b)
                {
                    let a_entry = change_entries.get(*a_idx);
                    let b_entry = change_entries.get(*b_idx);
                    let ordering = match (a_entry, b_entry) {
                        (Some(ae), Some(be)) => match col_key.as_ref() {
                            "change_kind" => ae.kind.cmp(&be.kind),
                            "change_status" => ae.status_after.cmp(&be.status_after),
                            "change_detail" => ae.changes.len().cmp(&be.changes.len()),
                            _ => ae.url.cmp(&be.url),
                        },
                        _ => std::cmp::Ordering::Equal,
                    };
                    return match sort {
                        ColumnSort::Descending => ordering.reverse(),
                        _ => ordering,
                    };
                }
                if let (
                    FlatRow::DirectoryAggregate {
                        path: a_path,
                        depth: a_depth,
                        page_count: a_pc,
                        avg_word_count: a_aw,
                        total_size: a_ts,
                        ..
                    },
                    FlatRow::DirectoryAggregate {
                        path: b_path,
                        depth: b_depth,
                        page_count: b_pc,
                        avg_word_count: b_aw,
                        total_size: b_ts,
                        ..
                    },
                ) = (a, b)
                {
                    let ordering = match col_key.as_ref() {
                        "dir_page_count" => a_pc.cmp(b_pc),
                        "dir_depth" => a_depth.cmp(b_depth),
                        "dir_avg_words" => a_aw.cmp(b_aw),
                        "dir_total_size" => a_ts.cmp(b_ts),
                        _ => a_path.cmp(b_path),
                    };
                    return match sort {
                        ColumnSort::Descending => ordering.reverse(),
                        _ => ordering,
                    };
                }
                let a_page = match a {
                    FlatRow::Image { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. }
                    | FlatRow::LinkRow { page, .. } => *page,
                    FlatRow::IssuesRow { .. }
                    | FlatRow::ChangeRow { .. }
                    | FlatRow::DirectoryAggregate { .. } => 0,
                };

                let b_page = match b {
                    FlatRow::Image { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. }
                    | FlatRow::LinkRow { page, .. } => *page,
                    FlatRow::IssuesRow { .. }
                    | FlatRow::ChangeRow { .. }
                    | FlatRow::DirectoryAggregate { .. } => 0,
                };
                let a_record = &self.all_pages[a_page];
                let b_record = &self.all_pages[b_page];
                let a_text = flat_cell_text(a_record, a, &col_key, root_origin.as_deref());
                let b_text = flat_cell_text(b_record, b, &col_key, root_origin.as_deref());

                let ordering = if numeric {
                    compare_numeric(&a_text, &b_text)
                } else {
                    a_text.cmp(&b_text)
                };

                match sort {
                    ColumnSort::Descending => ordering.reverse(),
                    _ => ordering,
                }
            });
        } else {
            self.filtered_indices.sort_by(|&a, &b| {
                let a_record = &self.all_pages[a];
                let b_record = &self.all_pages[b];
                let a_text = cell_text(
                    a_record,
                    &col_key,
                    &self.occurrence_counts,
                    self.active_tab,
                    self.root_origin.as_deref(),
                );
                let b_text = cell_text(
                    b_record,
                    &col_key,
                    &self.occurrence_counts,
                    self.active_tab,
                    self.root_origin.as_deref(),
                );

                let ordering = if numeric {
                    compare_numeric(&a_text, &b_text)
                } else {
                    a_text.cmp(&b_text)
                };

                match sort {
                    ColumnSort::Descending => ordering.reverse(),
                    _ => ordering,
                }
            });
        }
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use crate::crawl::event::PageRecord;

    fn internal_page(url: &str) -> PageRecord {
        PageRecord {
            url: url.into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            ..Default::default()
        }
    }

    fn internal_total(delegate: &mut ResultsDelegate) -> usize {
        delegate
            .tab_filter_counts()
            .get(&ResultTab::Internal)
            .map(|counts| counts.badge.total)
            .unwrap_or_default()
    }

    #[test]
    fn counts_cache_invalidates_on_push_and_clear() {
        let mut delegate = ResultsDelegate::new();
        assert_eq!(internal_total(&mut delegate), 0);

        // A first push must be reflected even though the cache was already
        // populated by the read above.
        delegate.push(internal_page("https://a.test/one"));
        assert_eq!(internal_total(&mut delegate), 1);

        delegate.push(internal_page("https://a.test/two"));
        assert_eq!(internal_total(&mut delegate), 2);

        delegate.clear();
        assert_eq!(internal_total(&mut delegate), 0);
    }

    #[test]
    fn counts_cache_invalidates_on_baseline_changes() {
        let mut delegate = ResultsDelegate::new();
        delegate.push(internal_page("https://a.test/current"));

        let baseline = vec![
            internal_page("https://a.test/current"),
            internal_page("https://a.test/removed"),
        ];
        delegate.set_baseline(baseline, 0);
        let changes_total = delegate
            .tab_filter_counts()
            .get(&ResultTab::Changes)
            .map(|counts| counts.badge.total)
            .unwrap_or_default();
        // The removed page is a change entry, so the Changes tab is non-empty.
        assert!(
            changes_total > 0,
            "baseline diff should produce change rows"
        );

        delegate.clear_baseline();
        let after = delegate
            .tab_filter_counts()
            .get(&ResultTab::Changes)
            .map(|counts| counts.badge.total)
            .unwrap_or_default();
        assert_eq!(after, 0, "clearing the baseline must drop change rows");
    }
}
