use std::collections::HashMap;

use gpui::{
    App, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme, StyledExt as _,
    table::{ColumnSort, TableDelegate, TableState},
    tooltip::Tooltip,
};

use crate::a11y_rules::rule_description;
use crate::crawl::event::PageRecord;
use crate::ui::tag::{Tone, tone_tag, tone_text_color};
use crate::views::ResultTab;
use crate::views::details_panel::{DetailsSelection, ImageDetails, ImageReference};

use super::cell::{
    cell_text, flat_cell_text, image_aggregate_cell_text, render_cell_tag, url_to_path,
};
use super::columns::{
    build_occurrence_counts, columns_for_tab_with_baseline, compare_numeric, hreflang_column_count,
    is_mono_column, is_numeric_column, is_right_aligned_column,
};
use super::data_build::{
    build_change_entries, build_image_aggregates, build_issues_entries, build_rows_for_tab,
    change_entry_matches, dir_format_size, issue_entry_matches, overview_issue_target,
};
use super::filter::{
    compute_tab_filter_counts, filter_for_tab, flat_row_matches_filter,
    image_aggregate_matches_filter,
};
use super::types::{
    ChangeEntry, ChangeKind, FlatRow, IssueEntry, IssueFilter, IssueType, TabFilterCounts,
    flat_row_page_index, tab_is_flattened,
};
use crate::report::{Report, ReportIssue};

/// Every tab's badge and sub-filter counts, for one set of pages. This is the
/// expensive part of loading a crawl: it walks the page set once per filter of
/// every tab, so on a few thousand pages it runs for seconds. Free-standing so
/// a crawl being opened from history can do it on a background thread and hand
/// the result to [`ResultsDelegate::prime_counts`].
pub(super) fn compute_all_tab_filter_counts(
    pages: &[PageRecord],
    change_entries: &[ChangeEntry],
    root_origin: Option<&str>,
) -> HashMap<ResultTab, TabFilterCounts> {
    let mut counts = HashMap::with_capacity(ResultTab::ALL.len());
    for &tab in ResultTab::ALL {
        counts.insert(
            tab,
            compute_tab_filter_counts(tab, pages, change_entries, root_origin),
        );
    }
    counts
}

/// The baseline crawl's issue counts, keyed by rule name, which the Overview
/// tab's delta column reads. Walks the baseline the same way the current crawl
/// is walked, so it is worth the same background treatment.
pub(super) fn baseline_issue_counts(pages: &[PageRecord]) -> HashMap<String, (usize, f32)> {
    build_issues_entries(pages)
        .into_iter()
        .map(|entry| (entry.name, (entry.count, entry.pct)))
        .collect()
}

/// The origin a root URL belongs to, as the delegate derives it.
pub(super) fn root_origin_of(root_url: &str) -> Option<String> {
    url::Url::parse(root_url)
        .ok()
        .map(|url| url.origin().ascii_serialization())
}

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
    /// The Overview tab's issue entries, rebuilt whenever its rows are. Every
    /// Overview row, cell, sort comparison and drill-down reads this rather than
    /// rebuilding: building the entries walks the whole page set once per rule,
    /// so recomputing it per cell made the tab quadratic in rules times cells.
    /// Empty on every other tab.
    issue_entries: Vec<IssueEntry>,
    /// How many hreflang column pairs the Hreflang tab shows, derived from the
    /// loaded records. Kept here so the columns are only rebuilt when the
    /// figure actually moves rather than on every streamed record.
    hreflang_columns: usize,
    /// Lazily computed per-tab badge and sub-filter counts, keyed by tab.
    /// Invalidated whenever the underlying data changes (pages, baseline, root),
    /// but not on tab/filter switches since the counts span all tabs.
    counts_cache: Option<HashMap<ResultTab, TabFilterCounts>>,
}

/// The loaded crawl, detached from the grid.
pub struct CrawlSnapshot {
    pages: Vec<PageRecord>,
    baseline: Option<(Vec<PageRecord>, i64)>,
    root_url: String,
}

/// One CSV per tab, each exactly what that tab's own export button produces.
/// Pure, so it belongs on a background thread: it rebuilds every tab's rows.
pub fn export_every_tab(snapshot: CrawlSnapshot) -> Vec<(ResultTab, Result<String, csv::Error>)> {
    let mut delegate = ResultsDelegate::new();
    delegate.set_root_url(&snapshot.root_url);
    delegate.all_pages = snapshot.pages;
    if let Some((baseline_pages, started_at)) = snapshot.baseline {
        delegate.baseline_issue_counts = baseline_issue_counts(&baseline_pages);
        delegate.baseline_pages = Some(baseline_pages);
        delegate.baseline_started_at = Some(started_at);
    }
    delegate.invalidate_counts();
    let has_baseline = delegate.baseline_pages.is_some();
    ResultTab::ALL
        .iter()
        .filter(|&&tab| tab != ResultTab::Changes || has_baseline)
        .map(|&tab| {
            delegate.switch_tab(tab);
            (tab, delegate.export_csv())
        })
        .collect()
}

impl ResultsDelegate {
    pub fn new() -> Self {
        let tab = ResultTab::Internal;
        Self {
            all_pages: Vec::new(),
            filtered_indices: Vec::new(),
            flat_rows: Vec::new(),
            occurrence_counts: HashMap::new(),
            columns: columns_for_tab_with_baseline(tab, false, 0),
            active_tab: tab,
            issue_filter: IssueFilter::All,
            root_origin: None,
            root_url: None,
            baseline_pages: None,
            baseline_started_at: None,
            baseline_issue_counts: HashMap::new(),
            issue_entries: Vec::new(),
            hreflang_columns: 0,
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
            self.counts_cache = Some(compute_all_tab_filter_counts(
                &self.all_pages,
                &change_entries,
                self.root_origin.as_deref(),
            ));
        }
        self.counts_cache
            .as_ref()
            .expect("counts_cache populated above")
    }

    /// Installs counts computed elsewhere, skipping the lazy pass above. The
    /// caller is responsible for having computed them from this same data, so
    /// this must be the last step of a load: any mutation after it invalidates
    /// the cache and the work is done again on the next render.
    pub(super) fn prime_counts(&mut self, counts: HashMap<ResultTab, TabFilterCounts>) {
        self.counts_cache = Some(counts);
    }

    fn rebuild_columns(&mut self) {
        self.columns = columns_for_tab_with_baseline(
            self.active_tab,
            self.baseline_pages.is_some(),
            self.hreflang_columns,
        );
    }

    pub fn switch_tab(&mut self, tab: ResultTab) {
        self.active_tab = tab;
        self.issue_filter = IssueFilter::All;
        self.rebuild_columns();
        self.rebuild_filter();
    }

    /// Installs a whole crawl at once, with the baseline's issue counts already
    /// built. Records and baseline each rebuild the filtered rows on their own,
    /// which on the Overview tab means building every issue entry, so setting
    /// them one after the other did that work twice.
    pub(super) fn apply_loaded_crawl(
        &mut self,
        pages: Vec<PageRecord>,
        baseline: Option<(Vec<PageRecord>, i64)>,
        baseline_issue_counts: HashMap<String, (usize, f32)>,
    ) {
        self.all_pages = pages;
        match baseline {
            Some((baseline_pages, started_at)) => {
                self.baseline_pages = Some(baseline_pages);
                self.baseline_started_at = Some(started_at);
                self.baseline_issue_counts = baseline_issue_counts;
            }
            None => {
                self.baseline_pages = None;
                self.baseline_started_at = None;
                self.baseline_issue_counts.clear();
            }
        }
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
        self.root_origin = root_origin_of(root_url);
        // Directory aggregates on the Site Structure tab key off the origin.
        self.invalidate_counts();
    }

    pub fn root_url(&self) -> Option<&str> {
        self.root_url.as_deref()
    }

    /// Test-facing: swaps in a fresh set of records, keeping the active tab,
    /// filter, root URL and baseline. The application installs crawls through
    /// `apply_loaded_crawl` with aggregates prepared off the foreground thread.
    #[cfg(test)]
    pub fn replace_records(&mut self, records: Vec<PageRecord>) {
        self.all_pages = records;
        self.invalidate_counts();
        self.rebuild_filter();
    }

    #[cfg(test)]
    pub fn set_baseline(&mut self, pages: Vec<PageRecord>, started_at: i64) {
        self.baseline_issue_counts = baseline_issue_counts(&pages);
        self.baseline_pages = Some(pages);
        self.baseline_started_at = Some(started_at);
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    #[cfg(test)]
    pub fn clear_baseline(&mut self) {
        self.baseline_pages = None;
        self.baseline_started_at = None;
        self.baseline_issue_counts.clear();
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    pub fn push(&mut self, record: PageRecord) {
        self.push_many(vec![record]);
    }

    /// One rebuild for a whole batch of streamed pages.
    pub fn push_many(&mut self, records: Vec<PageRecord>) {
        if records.is_empty() {
            return;
        }
        self.all_pages.extend(records);
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
                FlatRow::A11yIssue { page, .. } | FlatRow::SdItem { page, .. } => {
                    self.all_pages.get(*page)
                }
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
                FlatRow::IssuesRow { .. }
                | FlatRow::ImageAggregate(_)
                | FlatRow::DirectoryAggregate { .. } => None,
            }
        } else {
            self.filtered_indices
                .get(index)
                .and_then(|&idx| self.all_pages.get(idx))
        }
    }

    /// What the details panel should show for a row: the page behind it, or,
    /// on the Images tab, the image source and the pages referencing it.
    pub fn selection_at(&self, index: usize) -> Option<DetailsSelection> {
        if tab_is_flattened(self.active_tab)
            && let Some(FlatRow::ImageAggregate(image)) = self.flat_rows.get(index)
        {
            let references = image
                .pages
                .iter()
                .filter_map(|&page_index| self.all_pages.get(page_index))
                .flat_map(|page| {
                    page.images
                        .iter()
                        .filter(|candidate| candidate.src == image.src)
                        .map(|candidate| ImageReference {
                            page_url: page.url.clone(),
                            alt: candidate.alt.clone(),
                            has_alt_attr: candidate.has_alt_attr,
                        })
                })
                .collect();
            return Some(DetailsSelection::Image(Box::new(ImageDetails {
                src: image.src.clone(),
                width: image.width,
                height: image.height,
                references,
            })));
        }
        self.record_at(index)
            .cloned()
            .map(|record| DetailsSelection::Page(Box::new(record)))
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

    /// The Overview tab's issue entries, positionally aligned with its rows.
    /// Empty on every other tab.
    pub fn issue_entries(&self) -> &[IssueEntry] {
        &self.issue_entries
    }

    #[cfg(test)]
    pub(super) fn all_pages_for_test(&self) -> &[PageRecord] {
        &self.all_pages
    }

    /// Everything the PDF report says, gathered from the loaded crawl.
    ///
    /// The rules and their counts are the Overview's own, and each rule's URLs
    /// come from the filter its row clicks through to, so the report cannot
    /// disagree with the app about what a rule found.
    pub fn build_report(&self, render_mode: &str) -> Report {
        let documents = self
            .all_pages
            .iter()
            .filter(|page| page.is_page && page.is_internal)
            .count();
        let indexable = self
            .all_pages
            .iter()
            .filter(|page| page.is_page && page.indexability.as_deref() == Some("Indexable"))
            .count();
        let issues = build_issues_entries(&self.all_pages);
        let issue_rows = issues
            .iter()
            .filter(|entry| entry.issue_type == IssueType::Issue)
            .count();

        Report {
            site: self.root_url().unwrap_or_default().to_string(),
            generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            render_mode: render_mode.to_string(),
            summary: vec![
                ("URLs recorded".into(), self.all_pages.len().to_string()),
                ("Documents".into(), documents.to_string()),
                ("Indexable".into(), indexable.to_string()),
                ("Rules firing".into(), issues.len().to_string()),
                ("Of those, issues".into(), issue_rows.to_string()),
            ],
            issues: issues
                .iter()
                .map(|entry| ReportIssue {
                    name: entry.name.clone(),
                    issue_type: entry.issue_type.label().to_string(),
                    priority: entry.priority.label().to_string(),
                    count: entry.count,
                    pct: entry.pct,
                    description: entry.description.clone(),
                    hint: entry.hint.clone(),
                    offenders: self.offenders_for(&entry.name),
                })
                .collect(),
        }
    }

    /// The first few URLs a rule's own drill-down lands on.
    ///
    /// Goes through `filter_for_tab`, the same predicate the tab applies, so
    /// these are the rows the reader would see after clicking the rule. On the
    /// Images tab a row is an image source rather than a page, and the report
    /// lists what the tab lists.
    fn offenders_for(&self, rule: &str) -> Vec<String> {
        let Some((tab, filter)) = overview_issue_target(rule) else {
            return Vec::new();
        };
        let occurrences = build_occurrence_counts(tab, &self.all_pages);
        let indices = filter_for_tab(tab, filter, &self.all_pages, &occurrences);

        if tab == ResultTab::Images {
            return build_image_aggregates(&indices, &self.all_pages)
                .into_iter()
                .filter_map(|row| match row {
                    FlatRow::ImageAggregate(image)
                        if image_aggregate_matches_filter(&image, filter) =>
                    {
                        Some(image.src)
                    }
                    _ => None,
                })
                .take(crate::report::MAX_OFFENDERS)
                .collect();
        }

        indices
            .iter()
            .filter_map(|&index| self.all_pages.get(index))
            .map(|page| page.url.clone())
            .take(crate::report::MAX_OFFENDERS)
            .collect()
    }

    fn rebuild_filter(&mut self) {
        let hreflang_columns = hreflang_column_count(&self.all_pages);
        if hreflang_columns != self.hreflang_columns {
            self.hreflang_columns = hreflang_columns;
            self.rebuild_columns();
        }
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
            self.issue_entries.clear();
            return;
        }
        if self.active_tab == ResultTab::Overview {
            // The entries are the rows here, so build them once and keep them:
            // everything downstream reads `issue_entries` by index.
            self.issue_entries = build_issues_entries(&self.all_pages);
            self.flat_rows = (0..self.issue_entries.len())
                .map(|index| FlatRow::IssuesRow { index })
                .collect();
        } else {
            self.issue_entries.clear();
            let change_entries = self.change_entries();
            // Image aggregates span every page referencing the image, so they
            // are built from the tab's whole page set and filtered as rows
            // afterwards. Building them from the narrowed set would make a
            // logo's reference count shrink as soon as a filter was applied.
            let source_indices = if self.active_tab == ResultTab::Images {
                filter_for_tab(
                    self.active_tab,
                    IssueFilter::All,
                    &self.all_pages,
                    &self.occurrence_counts,
                )
            } else {
                self.filtered_indices.clone()
            };
            self.flat_rows = build_rows_for_tab(
                self.active_tab,
                &source_indices,
                &self.all_pages,
                &change_entries,
                self.root_origin.as_deref(),
            );
        }
        self.filter_flat_rows();
    }

    fn filter_flat_rows(&mut self) {
        if self.issue_filter == IssueFilter::All {
            return;
        }
        if self.active_tab == ResultTab::Overview {
            let entries = &self.issue_entries;
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
        if self.active_tab == ResultTab::Images {
            let filter = self.issue_filter;
            self.flat_rows.retain(|row| {
                let FlatRow::ImageAggregate(image) = row else {
                    return true;
                };
                image_aggregate_matches_filter(image, filter)
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
                FlatRow::A11yIssue { page, .. } | FlatRow::SdItem { page, .. } => *page,
                FlatRow::IssuesRow { .. }
                | FlatRow::ChangeRow { .. }
                | FlatRow::ImageAggregate(_)
                | FlatRow::DirectoryAggregate { .. } => return true,
            };
            let Some(page) = self.all_pages.get(page_index) else {
                return false;
            };
            flat_row_matches_filter(row, page, self.issue_filter)
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
        // Excel assumes a legacy code page for a CSV without a byte order
        // mark and mangles every non-ASCII title.
        let mut csv = String::from("\u{FEFF}");
        csv.push_str(&String::from_utf8_lossy(&bytes));
        Ok(csv)
    }

    /// A copy of the loaded crawl and its baseline, for work that should run
    /// off the foreground thread (see [`export_every_tab`]).
    pub fn snapshot(&self) -> CrawlSnapshot {
        CrawlSnapshot {
            pages: self.all_pages.clone(),
            baseline: self.baseline_pages.clone().zip(self.baseline_started_at),
            root_url: self.root_url.clone().unwrap_or_default(),
        }
    }

    fn flat_row_cell_text(&self, row: &FlatRow, col_key: &str) -> String {
        match row {
            FlatRow::IssuesRow { index } => {
                let Some(entry) = self.issue_entries.get(*index) else {
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
                        .unwrap_or_default(),
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
            FlatRow::ImageAggregate(image) => image_aggregate_cell_text(image, col_key).into(),
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
            _ => {
                let page_index = match row {
                    FlatRow::A11yIssue { page, .. } | FlatRow::SdItem { page, .. } => *page,
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
        // The heading sits over its own column of digits, so it follows the
        // cells rather than staying left while they move right.
        let mut head = div().flex().size_full().items_center();
        if is_right_aligned_column(&column.key) {
            head = head.justify_end();
        }
        head.text_xs()
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
        // Quantities line up on their last digit, so columns of them can be
        // compared down the page rather than read one row at a time.
        if is_right_aligned_column(&key) {
            cell = cell.justify_end();
        }
        if mono {
            cell = cell.font_family(cx.theme().mono_font_family.clone());
        }

        if tab_is_flattened(self.active_tab) {
            let Some(row) = self.flat_rows.get(row_ix) else {
                return cell;
            };
            match row {
                FlatRow::IssuesRow { index } => {
                    let Some(entry) = self.issue_entries.get(*index) else {
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
                        "count_prev" => {
                            SharedString::from(previous.map(|c| c.to_string()).unwrap_or_default())
                        }
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
                FlatRow::ImageAggregate(image) => {
                    let text = image_aggregate_cell_text(image, &key);
                    match key.as_ref() {
                        "image_has_alt" => {
                            let tone = if image.missing_alt_attr {
                                Tone::Warn
                            } else {
                                Tone::Ok
                            };
                            cell.child(tone_tag(tone, cx).child(text))
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
                        FlatRow::SdItem { page, .. } => *page,
                        FlatRow::IssuesRow { .. }
                        | FlatRow::ChangeRow { .. }
                        | FlatRow::ImageAggregate(_)
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
            let issue_entries = std::mem::take(&mut self.issue_entries);
            self.flat_rows.sort_by(|a, b| {
                if let (FlatRow::IssuesRow { index: a_idx }, FlatRow::IssuesRow { index: b_idx }) =
                    (a, b)
                {
                    let a_entry = issue_entries.get(*a_idx);
                    let b_entry = issue_entries.get(*b_idx);
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
                let a_page = flat_row_page_index(a).unwrap_or(0);
                let b_page = flat_row_page_index(b).unwrap_or(0);
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
            self.issue_entries = issue_entries;
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

    /// The report is only worth sending if its URLs are the ones the app would
    /// show for the same rule, so this asserts the two agree rather than that
    /// the report merely produced something.
    #[test]
    fn the_report_lists_the_urls_each_rule_lands_on() {
        let mut delegate = ResultsDelegate::new();
        delegate.set_root_url("https://a.test");
        for index in 0..3 {
            let mut page = internal_page(&format!("https://a.test/no-h1-{index}"));
            page.title = Some("A title of a perfectly reasonable length".into());
            page.compute_indexability();
            delegate.push(page);
        }

        let report = delegate.build_report("HTTP, no JavaScript");
        assert_eq!(report.site, "https://a.test");
        assert_eq!(
            report.summary.first().map(|(label, _)| label.as_str()),
            Some("URLs recorded")
        );

        let missing_h1 = report
            .issues
            .iter()
            .find(|issue| issue.name == "Missing H1")
            .expect("three pages without an H1 is a rule that fires");
        assert_eq!(missing_h1.count, 3);
        assert_eq!(missing_h1.offenders.len(), 3);
        for url in &missing_h1.offenders {
            assert!(url.starts_with("https://a.test/no-h1-"), "got {url}");
        }
    }

    #[test]
    fn the_report_lists_no_more_urls_than_the_cap() {
        let mut delegate = ResultsDelegate::new();
        delegate.set_root_url("https://a.test");
        for index in 0..(crate::report::MAX_OFFENDERS + 5) {
            let mut page = internal_page(&format!("https://a.test/page-{index}"));
            page.compute_indexability();
            delegate.push(page);
        }

        let report = delegate.build_report("HTTP, no JavaScript");
        for issue in &report.issues {
            assert!(
                issue.offenders.len() <= crate::report::MAX_OFFENDERS,
                "{} listed {} URLs",
                issue.name,
                issue.offenders.len()
            );
        }
        // The count is the whole finding even where the list is a sample.
        let missing_title = report
            .issues
            .iter()
            .find(|issue| issue.name == "Missing Page Title")
            .expect("none of these pages has a title");
        assert_eq!(missing_title.count, crate::report::MAX_OFFENDERS + 5);
    }
}

#[cfg(test)]
mod image_aggregate_tests {
    use super::*;
    use crate::crawl::event::ImageRef;

    fn page_with_images(url: &str, images: Vec<ImageRef>) -> PageRecord {
        PageRecord {
            url: url.into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            images,
            ..Default::default()
        }
    }

    fn image(src: &str, alt: Option<&str>) -> ImageRef {
        ImageRef {
            src: src.into(),
            alt: alt.map(|a| a.to_string()),
            width: Some(10),
            height: Some(10),
            has_alt_attr: alt.is_some(),
        }
    }

    fn pages() -> Vec<PageRecord> {
        vec![
            page_with_images(
                "https://a.test/one",
                vec![image("/logo.svg", Some("Logo")), image("/hero.png", None)],
            ),
            page_with_images("https://a.test/two", vec![image("/logo.svg", Some("Logo"))]),
        ]
    }

    #[test]
    fn a_filter_narrows_the_rows_without_shrinking_the_reference_count() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Images);
        delegate.replace_records(pages());
        assert_eq!(delegate.flat_rows().len(), 2);

        // /hero.png is the only source missing its alt attribute, and the logo
        // must keep both references even though only one page is selected by
        // the filter that produced the row set.
        delegate.set_issue_filter(IssueFilter::MissingAltAttribute);
        let rows = delegate.flat_rows();
        assert_eq!(rows.len(), 1);
        let FlatRow::ImageAggregate(hero) = &rows[0] else {
            panic!("expected an image aggregate");
        };
        assert_eq!(hero.src, "/hero.png");

        delegate.set_issue_filter(IssueFilter::All);
        let logo = delegate
            .flat_rows()
            .iter()
            .find_map(|row| match row {
                FlatRow::ImageAggregate(image) if image.src == "/logo.svg" => Some(image.clone()),
                _ => None,
            })
            .expect("logo row");
        assert_eq!(logo.reference_count, 2);
    }

    #[test]
    fn selecting_a_row_resolves_the_pages_referencing_the_image() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Images);
        delegate.replace_records(pages());
        let logo_row = delegate
            .flat_rows()
            .iter()
            .position(
                |row| matches!(row, FlatRow::ImageAggregate(image) if image.src == "/logo.svg"),
            )
            .expect("logo row");

        let Some(DetailsSelection::Image(details)) = delegate.selection_at(logo_row) else {
            panic!("an image row should select an image, not a page");
        };
        assert_eq!(details.src, "/logo.svg");
        let urls: Vec<&str> = details
            .references
            .iter()
            .map(|reference| reference.page_url.as_str())
            .collect();
        assert_eq!(urls, vec!["https://a.test/one", "https://a.test/two"]);
        assert!(details.references.iter().all(|r| r.has_alt_attr));
    }
}

#[cfg(test)]
mod issue_entry_cache_tests {
    use super::*;

    fn pages() -> Vec<PageRecord> {
        (0..5)
            .map(|i| PageRecord {
                url: format!("https://a.test/page-{i}"),
                is_internal: true,
                is_page: true,
                status: Some(200),
                title: Some("Shared title across every page".into()),
                word_count: Some(10),
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn overview_rows_and_entries_stay_aligned() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages());

        let expected = build_issues_entries(delegate.all_pages_for_test());
        assert!(!expected.is_empty(), "fixture should trip some rules");
        assert_eq!(delegate.issue_entries().len(), expected.len());
        assert_eq!(delegate.flat_rows().len(), expected.len());
        for (cached, fresh) in delegate.issue_entries().iter().zip(expected.iter()) {
            assert_eq!(cached.name, fresh.name);
            assert_eq!(cached.count, fresh.count);
        }
    }

    #[test]
    fn the_cache_refreshes_when_the_records_change() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages());
        let before = delegate.issue_entries().len();

        let mut fixed = pages();
        for (i, page) in fixed.iter_mut().enumerate() {
            page.title = Some(format!("A distinct title for page number {i}"));
            page.meta_description = Some(format!(
                "A distinct meta description for page number {i}, long enough to clear the bound."
            ));
            page.h1 = Some(format!("Heading {i}"));
            page.h2 = Some(format!("Sub {i}"));
            page.h1_count = 1;
            page.word_count = Some(500);
        }
        delegate.replace_records(fixed);
        assert!(
            delegate.issue_entries().len() < before,
            "fixing the pages should drop rules, got {} then {}",
            before,
            delegate.issue_entries().len()
        );
        assert_eq!(delegate.flat_rows().len(), delegate.issue_entries().len());
    }

    /// Stands in for a frame of the Overview grid: every visible cell asks the
    /// delegate for its text. This is the path that got slow, because each cell
    /// used to rebuild every issue entry from the full page set.
    #[test]
    fn a_full_grid_of_cells_is_cheap_to_render() {
        use std::time::Instant;
        let pages: Vec<PageRecord> = (0..1000)
            .map(|i| PageRecord {
                url: format!("https://a.test/page-{i}"),
                is_internal: true,
                is_page: true,
                status: Some(200),
                title: Some(format!("Title number {i} for the page")),
                word_count: Some(10),
                ..Default::default()
            })
            .collect();
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages);

        let columns = [
            "issue_name",
            "issue_type",
            "priority",
            "count",
            "pct",
            "delta",
        ];
        let start = Instant::now();
        let mut cells = 0;
        for row in delegate.flat_rows().to_vec() {
            for col in columns {
                std::hint::black_box(delegate.flat_row_cell_text(&row, col));
                cells += 1;
            }
        }
        let elapsed = start.elapsed();
        eprintln!("{cells} overview cells over 1000 pages in {elapsed:?}");
        assert!(
            elapsed < std::time::Duration::from_millis(50),
            "{cells} cells took {elapsed:?}"
        );
    }

    #[test]
    fn leaving_the_overview_drops_the_entries() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages());
        assert!(!delegate.issue_entries().is_empty());

        delegate.switch_tab(ResultTab::Internal);
        assert!(delegate.issue_entries().is_empty());
    }
}
