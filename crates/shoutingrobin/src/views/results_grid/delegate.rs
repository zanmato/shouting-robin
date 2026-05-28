use std::collections::{HashMap, HashSet};

use gpui::{App, Context, IntoElement, ParentElement, SharedString, Styled, Window, div};
use gpui_component::{
    ActiveTheme,
    table::{ColumnSort, TableDelegate, TableState},
};

use crate::crawl::engine::is_same_domain;
use crate::crawl::event::PageRecord;
use crate::ui::tag::{Tone, tone_tag};
use crate::views::ResultTab;

use super::cell::{cell_text, flat_cell_text, render_cell_tag, url_to_path};
use super::columns::{
    build_occurrence_counts, columns_for_tab, compare_numeric, is_mono_column, is_numeric_column,
};
use super::data_build::{
    build_directory_aggregates, build_issues_entries, build_issues_rows, dir_format_size,
    flat_row_item_count, flat_row_variant,
};
use super::filter::{filter_for_tab, filters_for_tab, flat_row_matches_filter};
use super::types::{FlatRow, IssueFilter, IssuePriority, IssueType, TabCounts, tab_is_flattened};

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
}

impl ResultsDelegate {
    pub fn new() -> Self {
        let tab = ResultTab::Internal;
        Self {
            all_pages: Vec::new(),
            filtered_indices: Vec::new(),
            flat_rows: Vec::new(),
            occurrence_counts: HashMap::new(),
            columns: columns_for_tab(tab),
            active_tab: tab,
            issue_filter: IssueFilter::All,
            root_origin: None,
            root_url: None,
        }
    }

    pub fn switch_tab(&mut self, tab: ResultTab) {
        self.active_tab = tab;
        self.issue_filter = IssueFilter::All;
        self.columns = columns_for_tab(tab);
        self.rebuild_filter();
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
    }

    pub fn root_url(&self) -> Option<&str> {
        self.root_url.as_deref()
    }

    pub fn push(&mut self, record: PageRecord) {
        self.all_pages.push(record);
        self.rebuild_filter();
    }

    pub fn clear(&mut self) {
        self.all_pages.clear();
        self.filtered_indices.clear();
        self.flat_rows.clear();
        self.occurrence_counts.clear();
        self.root_origin = None;
        self.root_url = None;
    }

    pub fn record_at(&self, index: usize) -> Option<&PageRecord> {
        if tab_is_flattened(self.active_tab) {
            let page_index = match self.flat_rows.get(index)? {
                FlatRow::Image { page, .. }
                | FlatRow::Outlink { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. }
                | FlatRow::LinkRow { page, .. } => *page,
                FlatRow::IssuesRow { .. }
                | FlatRow::DirectoryAggregate { .. } => return None,
            };
            self.all_pages.get(page_index)
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

    pub fn compute_tab_counts(&self) -> HashMap<ResultTab, TabCounts> {
        let mut counts = HashMap::new();

        for &tab in ResultTab::ALL {
            let filters = filters_for_tab(tab);
            let occ_counts = build_occurrence_counts(tab, &self.all_pages);
            let total = self.count_filter_for_tab(tab, IssueFilter::All, &occ_counts);

            if tab_is_flattened(tab) {
                let mut error_count = 0usize;
                let mut warn_count = 0usize;

                for &filter in filters.iter().skip(1) {
                    let count = self.count_filter_for_tab(tab, filter, &occ_counts);
                    if count > 0 {
                        match filter.tone() {
                            Tone::Err => error_count += count,
                            Tone::Warn => warn_count += count,
                            _ => {}
                        }
                    }
                }

                counts.insert(
                    tab,
                    TabCounts {
                        total,
                        errors: error_count,
                        warnings: warn_count,
                    },
                );
            } else {
                let mut error_indices = HashSet::new();
                let mut warn_indices = HashSet::new();

                for &filter in filters.iter().skip(1) {
                    let matching = filter_for_tab(tab, filter, &self.all_pages, &occ_counts);
                    match filter.tone() {
                        Tone::Err => error_indices.extend(matching),
                        Tone::Warn => warn_indices.extend(matching),
                        _ => {}
                    }
                }

                warn_indices.retain(|ix| !error_indices.contains(ix));

                counts.insert(
                    tab,
                    TabCounts {
                        total,
                        errors: error_indices.len(),
                        warnings: warn_indices.len(),
                    },
                );
            }
        }

        counts
    }

    fn count_filter_for_tab(
        &self,
        tab: ResultTab,
        filter: IssueFilter,
        occ_counts: &HashMap<String, usize>,
    ) -> usize {
        if tab == ResultTab::Overview {
            let entries = build_issues_entries(&self.all_pages);
            return entries
                .iter()
                .filter(|entry| match filter {
                    IssueFilter::All => true,
                    IssueFilter::IssueTypeError => entry.issue_type == IssueType::Issue,
                    IssueFilter::IssueTypeOpportunity => entry.issue_type == IssueType::Opportunity,
                    IssueFilter::IssueTypeWarning => entry.issue_type == IssueType::Warning,
                    IssueFilter::PriorityHigh => entry.priority == IssuePriority::High,
                    IssueFilter::PriorityMedium => entry.priority == IssuePriority::Medium,
                    IssueFilter::PriorityLow => entry.priority == IssuePriority::Low,
                    _ => true,
                })
                .count();
        }

        let indices = filter_for_tab(tab, filter, &self.all_pages, occ_counts);

        if tab_is_flattened(tab) {
            if filter == IssueFilter::All {
                indices
                    .iter()
                    .map(|&page_ix| {
                        self.all_pages
                            .get(page_ix)
                            .map(|p| flat_row_item_count(p, tab))
                            .unwrap_or(0)
                    })
                    .sum::<usize>()
            } else {
                indices
                    .iter()
                    .map(|&page_ix| {
                        self.all_pages
                            .get(page_ix)
                            .map(|p| {
                                let item_count = flat_row_item_count(p, tab);
                                (0..item_count)
                                    .filter(|item| {
                                        let row = flat_row_variant(tab, page_ix, *item);
                                        flat_row_matches_filter(&row, p, filter)
                                    })
                                    .count()
                            })
                            .unwrap_or(0)
                    })
                    .sum::<usize>()
            }
        } else {
            indices.len()
        }
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
        if self.active_tab == ResultTab::Overview {
            self.flat_rows = build_issues_rows(&self.all_pages);
            return;
        }
        if self.active_tab == ResultTab::SiteStructure {
            self.flat_rows =
                build_directory_aggregates(&self.all_pages, self.root_origin.as_deref());
            return;
        }
        if self.active_tab == ResultTab::Links {
            self.flat_rows = self
                .filtered_indices
                .iter()
                .flat_map(|&page_index| {
                    let Some(page) = self.all_pages.get(page_index) else {
                        return Vec::<FlatRow>::new();
                    };
                    page.outlinks
                        .iter()
                        .enumerate()
                        .filter(|(_, link)| is_same_domain(&page.url, &link.dst_url))
                        .map(|(item_index, _)| FlatRow::LinkRow {
                            page: page_index,
                            item: item_index,
                        })
                        .collect()
                })
                .collect();
            self.filter_flat_rows();
            return;
        }
        let active_tab = self.active_tab;
        let all_pages = &self.all_pages;
        self.flat_rows = if active_tab == ResultTab::External {
            self.filtered_indices
                .iter()
                .flat_map(|&page_index| {
                    let Some(page) = all_pages.get(page_index) else {
                        return Vec::<FlatRow>::new();
                    };
                    page.outlinks
                        .iter()
                        .enumerate()
                        .filter(|(_, link)| !is_same_domain(&page.url, &link.dst_url))
                        .map(|(item_index, _)| FlatRow::Outlink {
                            page: page_index,
                            item: item_index,
                        })
                        .collect()
                })
                .collect()
        } else {
            self.filtered_indices
                .iter()
                .flat_map(|&page_index| {
                    let item_count = all_pages
                        .get(page_index)
                        .map(|page| flat_row_item_count(page, active_tab))
                        .unwrap_or(0);
                    (0..item_count)
                        .map(move |item_index| flat_row_variant(active_tab, page_index, item_index))
                })
                .collect()
        };
        self.filter_flat_rows();
    }

    fn filter_flat_rows(&mut self) {
        if self.issue_filter == IssueFilter::All {
            return;
        }
        if self.active_tab == ResultTab::Overview {
            let entries = build_issues_entries(&self.all_pages);
            self.flat_rows.retain(|row| {
                let FlatRow::IssuesRow { index } = row else {
                    return true;
                };
                let Some(entry) = entries.get(*index) else {
                    return false;
                };
                match self.issue_filter {
                    IssueFilter::IssueTypeError => entry.issue_type == IssueType::Issue,
                    IssueFilter::IssueTypeOpportunity => entry.issue_type == IssueType::Opportunity,
                    IssueFilter::IssueTypeWarning => entry.issue_type == IssueType::Warning,
                    IssueFilter::PriorityHigh => entry.priority == IssuePriority::High,
                    IssueFilter::PriorityMedium => entry.priority == IssuePriority::Medium,
                    IssueFilter::PriorityLow => entry.priority == IssuePriority::Low,
                    _ => true,
                }
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
                | FlatRow::Outlink { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. }
                | FlatRow::LinkRow { page, .. } => *page,
                FlatRow::IssuesRow { .. }
                | FlatRow::DirectoryAggregate { .. } => return true,
            };
            let Some(page) = self.all_pages.get(page_index) else {
                return false;
            };
            flat_row_matches_filter(row, page, self.issue_filter)
        });
    }

    pub fn count_for_filter(&self, filter: IssueFilter) -> usize {
        self.count_filter_for_tab(self.active_tab, filter, &self.occurrence_counts)
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
                    | FlatRow::Outlink { page, .. }
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
            .size_full()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(column.name.clone())
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
        let mut cell = div().flex().items_center().text_xs();
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
                    let text = match key.as_ref() {
                        "issue_name" => SharedString::from(entry.name.clone()),
                        "issue_type" => SharedString::from(entry.issue_type.label()),
                        "priority" => SharedString::from(entry.priority.label()),
                        "count" => SharedString::from(entry.count.to_string()),
                        "pct" => SharedString::from(format!("{:.1}%", entry.pct)),
                        "description" => SharedString::from(entry.description.clone()),
                        "hint" => SharedString::from(entry.hint.clone()),
                        _ => SharedString::default(),
                    };
                    match key.as_ref() {
                        "issue_type" => cell.child(tone_tag(entry.issue_type.tone()).child(text)),
                        "priority" => cell.child(tone_tag(entry.priority.tone()).child(text)),
                        "count" => {
                            let tone = if entry.count > 0 {
                                Tone::Warn
                            } else {
                                Tone::Ok
                            };
                            cell.child(tone_tag(tone).child(text))
                        }
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
                            cell.child(tone_tag(tone).child(text))
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
                            cell.child(tone_tag(Tone::Warn).child(text))
                        }
                        _ => cell.child(text),
                    }
                }
                _ => {
                    let page_index = match row {
                        FlatRow::Image { page, .. }
                        | FlatRow::Outlink { page, .. }
                        | FlatRow::A11yIssue { page, .. }
                        | FlatRow::Hreflang { page, .. }
                        | FlatRow::SdItem { page, .. } => *page,
                        FlatRow::IssuesRow { .. }
                        | FlatRow::LinkRow { .. }
                        | FlatRow::DirectoryAggregate { .. } => unreachable!(),
                    };
                    let Some(record) = self.all_pages.get(page_index) else {
                        return cell;
                    };
                    let text = flat_cell_text(record, row, &key, self.root_origin.as_deref());
                    if let Some(tag) = render_cell_tag(record, &key, &text) {
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
            if let Some(tag) = render_cell_tag(record, &key, &text) {
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
        let col_key = col.key.to_string();
        let numeric = is_numeric_column(&col_key);
        let root_origin = self.root_origin.clone();

        if tab_is_flattened(self.active_tab) {
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
                            _ => ae.name.cmp(&be.name),
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
                    | FlatRow::Outlink { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. }
                    | FlatRow::LinkRow { page, .. } => *page,
                    FlatRow::IssuesRow { .. }
                    | FlatRow::DirectoryAggregate { .. } => 0,
                };

                let b_page = match b {
                    FlatRow::Image { page, .. }
                    | FlatRow::Outlink { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. }
                    | FlatRow::LinkRow { page, .. } => *page,
                    FlatRow::IssuesRow { .. }
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
