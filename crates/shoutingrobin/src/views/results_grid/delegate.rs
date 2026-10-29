use std::collections::HashMap;
use std::sync::Arc;

use gpui_kit::component::{
    ActiveTheme, IconName, StyledExt as _,
    menu::{PopupMenu, PopupMenuItem},
    table::{ColumnSort, TableDelegate, TableState},
    tooltip::Tooltip,
};
use gpui_kit::{
    App, ClipboardItem, Context, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div,
};

use crate::a11y_rules::rule_description;
use crate::crawl::event::PageRecord;
use crate::crawl::layout::PageLayoutChange;
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
    change_entry_matches, dir_format_size, issue_entry_matches, layout_template_cell_text,
    layout_template_matches, overview_issue_target,
};
use super::filter::{
    CountsProgress, count_every_tab, filter_for_tab, flat_row_matches_filter,
    image_aggregate_matches_filter,
};
use super::types::{
    ChangeEntry, ChangeKind, FlatRow, IssueEntry, IssueFilter, IssueType, TabFilterCounts,
    flat_row_page_index, tab_is_flattened,
};
use crate::report::{Report, ReportIssue};

/// Which data a count describes, so a pass that lands after the crawl was
/// replaced is dropped, and one that lands after more pages arrived is kept
/// but known to be behind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CountsStamp {
    pages_epoch: u64,
    version: u64,
}

/// What one counting pass reads, detached from the grid. Counting every tab
/// walks the crawl once per filter, which on tens of thousands of rows takes
/// seconds, so the grid runs it on a background thread.
pub(super) struct CountsJob {
    stamp: CountsStamp,
    pages: Vec<Arc<PageRecord>>,
    baseline: Option<Arc<[BaselinePage]>>,
    layout_changes: HashMap<String, PageLayoutChange>,
    root_origin: Option<String>,
    progress: CountsProgress,
}

/// A finished counting pass: every tab's counts and the Overview's issue
/// entries, which the Overview counts are built from.
pub(super) struct CountedCrawl {
    stamp: CountsStamp,
    pages_counted: usize,
    counts: HashMap<ResultTab, TabFilterCounts>,
    issue_entries: Vec<IssueEntry>,
    progress: CountsProgress,
}

impl CountsJob {
    /// A pass over a crawl that is not on screen yet, for
    /// [`ResultsDelegate::apply_loaded_crawl`] to install with it.
    pub(super) fn for_crawl(
        pages: Vec<Arc<PageRecord>>,
        baseline: Option<&Baseline>,
        root_url: &str,
    ) -> Self {
        Self {
            stamp: CountsStamp::default(),
            pages,
            baseline: baseline.map(|baseline| baseline.pages.clone()),
            layout_changes: HashMap::new(),
            root_origin: root_origin_of(root_url),
            progress: CountsProgress::default(),
        }
    }

    pub(super) fn run(self) -> CountedCrawl {
        let change_entries =
            changes_between(&self.pages, self.baseline.as_deref(), &self.layout_changes);
        let issue_entries = build_issues_entries(&self.pages);
        let mut progress = self.progress;
        let counts = count_every_tab(
            &self.pages,
            &change_entries,
            &issue_entries,
            self.root_origin.as_deref(),
            &mut progress,
        );
        CountedCrawl {
            stamp: self.stamp,
            pages_counted: self.pages.len(),
            counts,
            issue_entries,
            progress,
        }
    }
}

impl CountedCrawl {
    #[cfg(test)]
    pub(super) fn counts(&self) -> &HashMap<ResultTab, TabFilterCounts> {
        &self.counts
    }
}

/// The Overview's issue entries and the pages they were built from.
struct IssueEntriesCache {
    pages_epoch: u64,
    pages_len: usize,
    entries: Vec<IssueEntry>,
}

/// What the Changes tab compares a page on, for one internal page of the
/// baseline crawl.
#[derive(Debug, Clone)]
pub(super) struct BaselinePage {
    pub url: String,
    pub status: Option<u16>,
    pub title: Option<String>,
    pub meta_description: Option<String>,
    pub h1: Option<String>,
    pub indexability: Option<Arc<str>>,
    pub word_count: Option<u32>,
}

impl BaselinePage {
    pub(super) fn from_record(record: &PageRecord) -> Self {
        Self {
            url: record.url.clone(),
            status: record.status,
            title: record.title.clone(),
            meta_description: record.meta_description.clone(),
            h1: record.h1.clone(),
            indexability: record.indexability.clone(),
            word_count: record.word_count,
        }
    }

    /// A record for a page the current crawl no longer has, so selecting its
    /// Changes row still shows what the baseline knew about it.
    fn to_record(&self) -> PageRecord {
        PageRecord {
            url: self.url.clone(),
            status: self.status,
            title: self.title.clone(),
            meta_description: self.meta_description.clone(),
            h1: self.h1.clone(),
            indexability: self.indexability.clone(),
            word_count: self.word_count,
            is_internal: true,
            ..Default::default()
        }
    }
}

/// The previous crawl a crawl is compared with. Only what the comparison reads
/// is kept: the Changes tab needs a few fields per internal page and the
/// Overview needs the baseline's issue counts, which are counted once here.
/// Holding every record of a second large crawl cost as much memory as the
/// crawl on screen.
#[derive(Clone)]
pub struct Baseline {
    pages: Arc<[BaselinePage]>,
    started_at: i64,
    issue_counts: Arc<HashMap<String, (usize, f32)>>,
}

impl Baseline {
    /// Counts the baseline's issues and keeps its internal pages' compared
    /// fields. Walks every record once per rule, so it belongs on a background
    /// thread.
    pub fn new(records: &[Arc<PageRecord>], started_at: i64) -> Self {
        Self {
            pages: records
                .iter()
                .filter(|record| record.is_internal)
                .map(|record| BaselinePage::from_record(record))
                .collect(),
            started_at,
            issue_counts: Arc::new(baseline_issue_counts(records)),
        }
    }
}

/// The baseline crawl's issue counts, keyed by rule name, which the Overview
/// tab's delta column reads. Walks the baseline the same way the current crawl
/// is walked, so it is worth the same background treatment.
fn baseline_issue_counts(pages: &[Arc<PageRecord>]) -> HashMap<String, (usize, f32)> {
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
    pub(super) all_pages: Vec<Arc<PageRecord>>,
    filtered_indices: Vec<usize>,
    flat_rows: Vec<FlatRow>,
    occurrence_counts: HashMap<String, usize>,
    columns: Vec<gpui_kit::component::table::Column>,
    pub(super) active_tab: ResultTab,
    issue_filter: IssueFilter,
    pub(super) root_origin: Option<String>,
    root_url: Option<String>,
    baseline: Option<Baseline>,
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
    /// Bumped whenever `all_pages` is replaced rather than added to. Counting
    /// progress and issue entries carry over only within one.
    pages_epoch: u64,
    /// Bumped by every change the counts read: the pages, the baseline, the
    /// layout changes and the root.
    counts_version: u64,
    /// The latest per-tab badge and sub-filter counts, kept on screen while
    /// newer ones are counted. `None` until the first pass over a crawl lands.
    counts: Option<(CountsStamp, HashMap<ResultTab, TabFilterCounts>)>,
    /// What the last pass learned about the pages it saw, so the next one only
    /// looks at pages added since. Out while a pass is running.
    counts_progress: Option<(u64, CountsProgress)>,
    /// The Overview's issue entries from the latest pass or rebuild. Kept when
    /// another tab is open, so opening the Overview does not rebuild them.
    issue_entries_cache: Option<IssueEntriesCache>,
    /// Set by the grid, which counts on a background thread. A streamed page
    /// then leaves the Overview on the latest counted entries until the next
    /// pass lands, rather than rebuilding them for every batch. Exports and
    /// tests leave it unset and always get entries for every page.
    defer_aggregates: bool,
    has_layouts: bool,
    /// See [`Self::change_entries`]. Emptied by `invalidate_counts`, which
    /// every change to the pages, baseline or layout changes goes through.
    change_entries: std::cell::OnceCell<Vec<ChangeEntry>>,
    /// Pages whose layout changed since the baseline, by URL. Filled in after
    /// the crawl is on screen, since comparing layouts reads every page's.
    layout_changes: HashMap<String, PageLayoutChange>,
    /// The cell under the last right click, as row and column. The table
    /// only tells the context menu the row.
    right_clicked_cell: Option<(usize, usize)>,
}

/// The loaded crawl, detached from the grid.
pub struct CrawlSnapshot {
    pages: Vec<Arc<PageRecord>>,
    baseline: Option<Baseline>,
    root_url: String,
}

fn changes_between(
    pages: &[Arc<PageRecord>],
    baseline: Option<&[BaselinePage]>,
    layout_changes: &HashMap<String, PageLayoutChange>,
) -> Vec<ChangeEntry> {
    match baseline {
        Some(baseline) => build_change_entries(pages, baseline, layout_changes),
        None => Vec::new(),
    }
}

/// Every use of `src` as an image among `pages`, one per `<img>`.
fn image_references<'a>(
    pages: impl Iterator<Item = &'a Arc<PageRecord>>,
    src: &str,
) -> Vec<ImageReference> {
    pages
        .flat_map(|page| {
            page.images
                .iter()
                .filter(|image| *image.src == *src)
                .map(|image| ImageReference {
                    page_url: page.url.clone(),
                    alt: image.alt.as_deref().map(str::to_string),
                    has_alt_attr: image.has_alt_attr,
                })
        })
        .collect()
}

/// One CSV per tab, each exactly what that tab's own export button produces.
/// Pure, so it belongs on a background thread: it rebuilds every tab's rows.
pub fn export_every_tab(snapshot: CrawlSnapshot) -> Vec<(ResultTab, Result<String, csv::Error>)> {
    let mut delegate = ResultsDelegate::new();
    delegate.set_root_url(&snapshot.root_url);
    delegate.set_pages(snapshot.pages);
    delegate.baseline = snapshot.baseline;
    delegate.invalidate_counts();
    let has_baseline = delegate.baseline.is_some();
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
            baseline: None,
            issue_entries: Vec::new(),
            hreflang_columns: 0,
            pages_epoch: 0,
            counts_version: 0,
            counts: None,
            counts_progress: None,
            issue_entries_cache: None,
            defer_aggregates: false,
            has_layouts: false,
            change_entries: std::cell::OnceCell::new(),
            layout_changes: HashMap::new(),
            right_clicked_cell: None,
        }
    }

    /// Installs the result of comparing the crawl's layouts with the
    /// baseline's, which changes the Changes and Layout tabs.
    pub fn set_layout_changes(&mut self, layout_changes: HashMap<String, PageLayoutChange>) {
        self.layout_changes = layout_changes;
        self.invalidate_counts();
        self.rebuild_filter();
    }

    /// Whether any loaded page has a captured layout, which is when the
    /// Layout tab has anything to show.
    pub fn has_layouts(&self) -> bool {
        self.has_layouts
    }

    /// See `defer_aggregates`.
    pub(super) fn count_in_background(mut self) -> Self {
        self.defer_aggregates = true;
        self
    }

    /// Marks the counts on screen as behind the data, and drops what was
    /// derived from it. The counts stay on screen until the next pass
    /// replaces them.
    fn invalidate_counts(&mut self) {
        self.counts_version += 1;
        self.change_entries.take();
    }

    /// Swaps in a different set of pages: a loaded crawl, or none.
    fn set_pages(&mut self, pages: Vec<Arc<PageRecord>>) {
        self.has_layouts = pages.iter().any(|page| page.layout.is_some());
        self.all_pages = pages;
        self.pages_epoch += 1;
        self.counts = None;
        self.counts_progress = None;
        self.issue_entries_cache = None;
        self.invalidate_counts();
    }

    fn counts_stamp(&self) -> CountsStamp {
        CountsStamp {
            pages_epoch: self.pages_epoch,
            version: self.counts_version,
        }
    }

    pub(super) fn counts_are_current(&self) -> bool {
        self.counts
            .as_ref()
            .is_some_and(|(stamp, _)| *stamp == self.counts_stamp())
    }

    /// The counts on screen, which may be behind pages streamed in since the
    /// last pass.
    pub(super) fn displayed_counts(&self) -> Option<&HashMap<ResultTab, TabFilterCounts>> {
        self.counts.as_ref().map(|(_, counts)| counts)
    }

    /// The pass that would bring the counts up to date, or `None` when they
    /// are. Takes the counting progress with it; [`Self::finish_counts`] puts
    /// it back.
    pub(super) fn counts_job(&mut self) -> Option<CountsJob> {
        if self.counts_are_current() {
            return None;
        }
        let progress = match self.counts_progress.take() {
            Some((pages_epoch, progress)) if pages_epoch == self.pages_epoch => progress,
            _ => CountsProgress::default(),
        };
        Some(CountsJob {
            stamp: self.counts_stamp(),
            pages: self.all_pages.clone(),
            baseline: self
                .baseline
                .as_ref()
                .map(|baseline| baseline.pages.clone()),
            layout_changes: self.layout_changes.clone(),
            root_origin: self.root_origin.clone(),
            progress,
        })
    }

    /// Installs a finished pass, unless the crawl it counted was replaced
    /// while it ran.
    pub(super) fn finish_counts(&mut self, counted: CountedCrawl) {
        if counted.stamp.pages_epoch != self.pages_epoch {
            return;
        }
        self.install_counts(counted.stamp, counted);
    }

    fn install_counts(&mut self, stamp: CountsStamp, counted: CountedCrawl) {
        self.counts = Some((stamp, counted.counts));
        self.counts_progress = Some((stamp.pages_epoch, counted.progress));
        let entries_are_newer = self.issue_entries_cache.as_ref().is_none_or(|cache| {
            cache.pages_epoch != stamp.pages_epoch || cache.pages_len <= counted.pages_counted
        });
        if entries_are_newer {
            self.issue_entries_cache = Some(IssueEntriesCache {
                pages_epoch: stamp.pages_epoch,
                pages_len: counted.pages_counted,
                entries: counted.issue_entries,
            });
            if self.active_tab == ResultTab::Overview {
                self.rebuild_flat_rows();
            }
        }
    }

    /// Every tab's badge and sub-filter counts for all the pages, counted on
    /// the spot when they are behind. The badge and the sub-filter buttons both
    /// read these, so they can never disagree.
    #[cfg(test)]
    pub fn tab_filter_counts(&mut self) -> &HashMap<ResultTab, TabFilterCounts> {
        if let Some(job) = self.counts_job() {
            let counted = job.run();
            self.finish_counts(counted);
        }
        let stamp = self.counts_stamp();
        &self.counts.get_or_insert_with(|| (stamp, HashMap::new())).1
    }

    /// The Overview's issue entries. While a crawl streams into the grid,
    /// these are the latest counted ones rather than a fresh build, which on a
    /// large crawl takes a second.
    fn overview_entries(&mut self) -> Vec<IssueEntry> {
        let pages_len = self.all_pages.len();
        if let Some(cache) = &self.issue_entries_cache
            && cache.pages_epoch == self.pages_epoch
            && (cache.pages_len == pages_len || self.defer_aggregates)
        {
            return cache.entries.clone();
        }
        let entries = build_issues_entries(&self.all_pages);
        self.issue_entries_cache = Some(IssueEntriesCache {
            pages_epoch: self.pages_epoch,
            pages_len,
            entries: entries.clone(),
        });
        entries
    }

    fn rebuild_columns(&mut self) {
        self.columns = columns_for_tab_with_baseline(
            self.active_tab,
            self.baseline.is_some(),
            self.hreflang_columns,
        );
    }

    pub fn switch_tab(&mut self, tab: ResultTab) {
        self.active_tab = tab;
        self.issue_filter = IssueFilter::All;
        self.rebuild_columns();
        self.rebuild_filter();
    }

    /// Installs a whole crawl at once, with its baseline and a counting pass
    /// over both already done. Records and baseline each rebuild
    /// the filtered rows on their own, which on the Overview tab means building
    /// every issue entry, so setting them one after the other did that work
    /// twice. The counted pass goes in before the rows are built, so the
    /// Overview takes its issue entries from it rather than building them here.
    pub(super) fn apply_loaded_crawl(
        &mut self,
        pages: Vec<Arc<PageRecord>>,
        baseline: Option<Baseline>,
        counted: CountedCrawl,
    ) {
        self.set_pages(pages);
        self.baseline = baseline;
        self.invalidate_counts();
        let stamp = self.counts_stamp();
        self.install_counts(stamp, counted);
        self.rebuild_columns();
        self.rebuild_filter();
    }

    pub fn has_baseline(&self) -> bool {
        self.baseline.is_some()
    }

    pub fn baseline_started_at(&self) -> Option<i64> {
        self.baseline.as_ref().map(|baseline| baseline.started_at)
    }

    /// The baseline's count and share for an Overview rule, by rule name.
    fn baseline_issue_count(&self, name: &str) -> Option<&(usize, f32)> {
        self.baseline.as_ref()?.issue_counts.get(name)
    }

    fn baseline_pages(&self) -> Option<&[BaselinePage]> {
        self.baseline
            .as_ref()
            .map(|baseline| baseline.pages.as_ref())
    }

    /// The Changes tab's entries, built on first use after the data changes.
    /// Building them compares both crawls in full, and every Changes cell,
    /// selection and sort reads them, so building them per call froze the
    /// window on a large comparison.
    fn change_entries(&self) -> &[ChangeEntry] {
        self.change_entries.get_or_init(|| {
            changes_between(&self.all_pages, self.baseline_pages(), &self.layout_changes)
        })
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
        self.set_pages(records.into_iter().map(Arc::new).collect());
        self.rebuild_filter();
    }

    #[cfg(test)]
    pub fn set_baseline(&mut self, pages: Vec<PageRecord>, started_at: i64) {
        let pages: Vec<Arc<PageRecord>> = pages.into_iter().map(Arc::new).collect();
        self.baseline = Some(Baseline::new(&pages, started_at));
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    #[cfg(test)]
    pub fn clear_baseline(&mut self) {
        self.baseline = None;
        self.layout_changes.clear();
        self.invalidate_counts();
        self.rebuild_columns();
        self.rebuild_filter();
    }

    #[cfg(test)]
    pub fn push(&mut self, record: PageRecord) {
        self.push_many(vec![record]);
    }

    /// One rebuild for a whole batch of streamed pages.
    pub fn push_many(&mut self, records: Vec<PageRecord>) {
        if records.is_empty() {
            return;
        }
        self.has_layouts |= records.iter().any(|record| record.layout.is_some());
        self.all_pages.extend(records.into_iter().map(Arc::new));
        self.invalidate_counts();
        self.rebuild_filter();
    }

    /// The records whose URL no row has yet.
    pub(super) fn without_held(&self, records: Vec<PageRecord>) -> Vec<PageRecord> {
        if records.is_empty() {
            return records;
        }
        let held: std::collections::HashSet<&str> = self
            .all_pages
            .iter()
            .map(|page| page.url.as_str())
            .collect();
        records
            .into_iter()
            .filter(|record| !held.contains(record.url.as_str()))
            .collect()
    }

    pub fn clear(&mut self) {
        self.set_pages(Vec::new());
        self.filtered_indices.clear();
        self.flat_rows.clear();
        self.occurrence_counts.clear();
        self.root_origin = None;
        self.root_url = None;
        self.baseline = None;
        self.layout_changes.clear();
        self.invalidate_counts();
        self.rebuild_columns();
    }

    /// The page behind a row. A removed page's Changes row is the one row
    /// with no record in the crawl, so it gets one made from the baseline.
    pub fn record_at(&self, index: usize) -> Option<std::borrow::Cow<'_, PageRecord>> {
        use std::borrow::Cow;
        if tab_is_flattened(self.active_tab) {
            match self.flat_rows.get(index)? {
                FlatRow::A11yIssue { page, .. } | FlatRow::SdItem { page, .. } => self
                    .all_pages
                    .get(*page)
                    .map(|page| Cow::Borrowed(page.as_ref())),
                FlatRow::ChangeRow { index } => {
                    let entries = self.change_entries();
                    let entry = entries.get(*index)?;
                    let url = entry.url.as_str();
                    match entry.kind {
                        ChangeKind::Removed => self
                            .baseline_pages()?
                            .iter()
                            .find(|page| page.url == url)
                            .map(|page| Cow::Owned(page.to_record())),
                        _ => self
                            .all_pages
                            .iter()
                            .find(|page| page.url == url)
                            .map(|page| Cow::Borrowed(page.as_ref())),
                    }
                }
                FlatRow::LayoutTemplate(template) => template
                    .example_page()
                    .and_then(|page_index| self.all_pages.get(page_index))
                    .map(|page| Cow::Borrowed(page.as_ref())),
                FlatRow::IssuesRow { .. }
                | FlatRow::ImageAggregate(_)
                | FlatRow::DirectoryAggregate { .. } => None,
            }
        } else {
            self.filtered_indices
                .get(index)
                .and_then(|&idx| self.all_pages.get(idx))
                .map(|page| Cow::Borrowed(page.as_ref()))
        }
    }

    /// What the details panel should show for a row: the page behind it, or,
    /// on the Images tab, the image source and the pages referencing it.
    pub fn selection_at(&self, index: usize) -> Option<DetailsSelection> {
        if tab_is_flattened(self.active_tab)
            && let Some(FlatRow::ImageAggregate(image)) = self.flat_rows.get(index)
        {
            let pages = image
                .pages
                .iter()
                .filter_map(|&page_index| self.all_pages.get(page_index));
            return Some(DetailsSelection::Image(Box::new(ImageDetails {
                src: image.src.clone(),
                width: image.width,
                height: image.height,
                references: image_references(pages, &image.src),
            })));
        }
        let record = self.record_at(index)?;
        // Every page, since any row can be a URL some page loads as an image:
        // a harvested resource, or an image also reached through a link. One
        // pass over the crawl's image lists, and only on selection.
        let references = image_references(self.all_pages.iter(), &record.url);
        Some(DetailsSelection::Page {
            record: Box::new(record.into_owned()),
            references,
        })
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
    pub(super) fn all_pages_for_test(&self) -> &[Arc<PageRecord>] {
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
            self.issue_entries = self.overview_entries();
            self.flat_rows = (0..self.issue_entries.len())
                .map(|index| FlatRow::IssuesRow { index })
                .collect();
        } else {
            self.issue_entries.clear();
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
            let rows = build_rows_for_tab(
                self.active_tab,
                &source_indices,
                &self.all_pages,
                self.change_entries(),
                self.root_origin.as_deref(),
            );
            self.flat_rows = rows;
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
            // The cell rather than `change_entries()`, which would borrow all
            // of `self` while the rows are being changed.
            let entries = self.change_entries.get_or_init(|| {
                changes_between(
                    &self.all_pages,
                    self.baseline
                        .as_ref()
                        .map(|baseline| baseline.pages.as_ref()),
                    &self.layout_changes,
                )
            });
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
        if self.active_tab == ResultTab::Layout {
            let filter = self.issue_filter;
            self.flat_rows.retain(|row| {
                let FlatRow::LayoutTemplate(template) = row else {
                    return true;
                };
                layout_template_matches(template, filter)
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
                | FlatRow::LayoutTemplate(_)
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
            let cells: Vec<String> = self
                .columns
                .iter()
                .map(|column| self.text_at(row_ix, column.key.as_ref()))
                .collect();
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

    /// The text of one cell as the export writes it.
    fn text_at(&self, row_ix: usize, col_key: &str) -> String {
        if tab_is_flattened(self.active_tab) {
            return self
                .flat_rows
                .get(row_ix)
                .map(|row| self.flat_row_cell_text(row, col_key))
                .unwrap_or_default();
        }
        let Some(record) = self
            .filtered_indices
            .get(row_ix)
            .and_then(|&index| self.all_pages.get(index))
        else {
            return String::new();
        };
        cell_text(
            record,
            col_key,
            &self.occurrence_counts,
            self.active_tab,
            self.root_origin.as_deref(),
        )
        .into()
    }

    /// A copy of the loaded crawl and its baseline, for work that should run
    /// off the foreground thread (see [`export_every_tab`]).
    pub fn snapshot(&self) -> CrawlSnapshot {
        CrawlSnapshot {
            pages: self.all_pages.clone(),
            baseline: self.baseline.clone(),
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
                        .baseline_issue_count(&entry.name)
                        .map(|(count, _)| count.to_string())
                        .unwrap_or_default(),
                    "count_delta" => {
                        let previous = self
                            .baseline_issue_count(&entry.name)
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
            FlatRow::LayoutTemplate(template) => layout_template_cell_text(template, col_key),
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

    fn column(&self, col_ix: usize, _: &App) -> gpui_kit::component::table::Column {
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

    fn context_menu(
        &mut self,
        row_ix: usize,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> PopupMenu {
        let text = self
            .right_clicked_cell
            .take()
            .filter(|(row, _)| *row == row_ix)
            .and_then(|(_, col_ix)| self.columns.get(col_ix))
            .map(|column| self.text_at(row_ix, column.key.as_ref()));
        let has_text = text.is_some();
        menu.item(
            PopupMenuItem::new("Copy")
                .icon(IconName::Copy)
                .disabled(!has_text)
                .on_click(move |_event, _window, cx| {
                    if let Some(text) = &text {
                        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                    }
                }),
        )
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
        let mut cell = div()
            .flex()
            .size_full()
            .items_center()
            .text_xs()
            .on_mouse_down(
                MouseButton::Right,
                cx.listener(move |table, _: &MouseDownEvent, _window, _cx| {
                    table.delegate_mut().right_clicked_cell = Some((row_ix, col_ix));
                }),
            );
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
                    let previous = self.baseline_issue_count(&entry.name).map(|(c, _)| *c);
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
                FlatRow::LayoutTemplate(template) => {
                    let text = SharedString::from(layout_template_cell_text(template, &key));
                    match key.as_ref() {
                        "layout_missing_markup" if !template.missing_markup.is_empty() => {
                            cell.child(tone_tag(Tone::Warn, cx).child(text))
                        }
                        "layout_changed" if !template.changed.is_empty() => {
                            cell.child(tone_tag(ChangeKind::Changed.tone(), cx).child(text))
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
                        | FlatRow::LayoutTemplate(_)
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
            let change_entries: &[ChangeEntry] = if self.active_tab == ResultTab::Changes {
                self.change_entries.get_or_init(|| {
                    changes_between(
                        &self.all_pages,
                        self.baseline
                            .as_ref()
                            .map(|baseline| baseline.pages.as_ref()),
                        &self.layout_changes,
                    )
                })
            } else {
                &[]
            };
            let baseline_issue_counts = self
                .baseline
                .as_ref()
                .map(|baseline| baseline.issue_counts.clone());
            let previous_count = |name: &str| {
                baseline_issue_counts
                    .as_ref()
                    .and_then(|counts| counts.get(name))
                    .map_or(0, |(count, _)| *count)
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
                            "count_prev" => previous_count(&ae.name).cmp(&previous_count(&be.name)),
                            "count_delta" => {
                                let ad = ae.count as i64 - previous_count(&ae.name) as i64;
                                let bd = be.count as i64 - previous_count(&be.name) as i64;
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

    /// A running crawl read back from the database already holds some of the
    /// pages that streamed in during the read; adding those again would list
    /// them twice.
    #[test]
    fn only_records_the_grid_lacks_are_kept() {
        let mut delegate = ResultsDelegate::new();
        delegate.replace_records(vec![
            internal_page("https://a.test/one"),
            internal_page("https://a.test/two"),
        ]);
        let kept: Vec<String> = delegate
            .without_held(vec![
                internal_page("https://a.test/two"),
                internal_page("https://a.test/three"),
            ])
            .into_iter()
            .map(|record| record.url)
            .collect();
        assert_eq!(kept, vec!["https://a.test/three".to_string()]);
    }

    /// Every cell of the Changes tab reads the change entries, and building
    /// them compares both crawls in full. Building them per cell froze the
    /// window on a large comparison.
    #[test]
    fn a_screen_of_change_cells_is_cheap_to_render() {
        let mut delegate = ResultsDelegate::new();
        delegate.replace_records(
            (0..20_000)
                .map(|index| internal_page(&format!("https://a.test/new-{index}")))
                .collect(),
        );
        delegate.set_baseline(
            (0..20_000)
                .map(|index| internal_page(&format!("https://a.test/old-{index}")))
                .collect(),
            0,
        );
        delegate.switch_tab(ResultTab::Changes);
        assert_eq!(delegate.flat_rows().len(), 40_000);

        let started = std::time::Instant::now();
        for row in &delegate.flat_rows()[..50] {
            for column in [
                "change_url",
                "change_kind",
                "change_status",
                "change_detail",
            ] {
                std::hint::black_box(delegate.flat_row_cell_text(row, column));
            }
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed < std::time::Duration::from_millis(50),
            "200 change cells took {elapsed:?}"
        );
    }

    /// The entries are kept between reads, so they have to follow the data.
    #[test]
    fn change_entries_follow_the_baseline() {
        let mut delegate = ResultsDelegate::new();
        delegate.replace_records(vec![internal_page("https://a.test/kept")]);
        delegate.set_baseline(
            vec![
                internal_page("https://a.test/kept"),
                internal_page("https://a.test/gone"),
            ],
            0,
        );
        delegate.switch_tab(ResultTab::Changes);
        assert_eq!(delegate.flat_rows().len(), 1, "the removed page");

        delegate.push(internal_page("https://a.test/added"));
        assert_eq!(delegate.flat_rows().len(), 2, "plus the added page");

        delegate.clear_baseline();
        assert!(delegate.flat_rows().is_empty());
    }

    /// The baseline keeps only what the comparison reads, so selecting a page
    /// the current crawl no longer has shows a record built from that.
    #[test]
    fn a_removed_page_is_selectable_from_the_slim_baseline() {
        let mut delegate = ResultsDelegate::new();
        delegate.replace_records(vec![internal_page("https://a.test/kept")]);
        let mut gone = internal_page("https://a.test/gone");
        gone.status = Some(404);
        gone.title = Some("Gone".into());
        gone.outlinks = thin_vec::thin_vec![crate::crawl::event::Outlink {
            dst_url: "https://a.test/kept".into(),
            anchor: None,
            rel: None,
            csr_only: false,
        }];
        delegate.set_baseline(vec![internal_page("https://a.test/kept"), gone], 0);
        delegate.switch_tab(ResultTab::Changes);

        let record = delegate.record_at(0).expect("the removed page's row");
        assert_eq!(record.url, "https://a.test/gone");
        assert_eq!(record.status, Some(404));
        assert_eq!(record.title.as_deref(), Some("Gone"));
        assert!(
            record.outlinks.is_empty(),
            "only the compared fields are kept"
        );
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
            images: images.into(),
            ..Default::default()
        }
    }

    fn image(src: &str, alt: Option<&str>) -> ImageRef {
        ImageRef {
            src: src.into(),
            alt: alt.map(Into::into),
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

    /// A broken image is a row of its own on Response Codes, and Inlinks only
    /// lists anchors, so the pages using it are the only lead to where it
    /// has to be fixed.
    #[test]
    fn a_resource_row_lists_the_pages_using_it_as_an_image() {
        let broken = "https://cdn.test/missing.jpg";
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::ResponseCodes);
        delegate.replace_records(vec![
            page_with_images("https://a.test/one", vec![image(broken, Some("Product"))]),
            page_with_images("https://a.test/two", vec![image("/logo.svg", Some("Logo"))]),
            page_with_images("https://a.test/three", vec![image(broken, None)]),
            PageRecord {
                url: broken.into(),
                status: Some(404),
                is_resource: true,
                ..Default::default()
            },
        ]);

        let row = (0..delegate.filtered_count())
            .find(|&row| {
                delegate
                    .record_at(row)
                    .is_some_and(|record| record.url == broken)
            })
            .expect("the broken image is listed");
        let Some(DetailsSelection::Page { references, .. }) = delegate.selection_at(row) else {
            panic!("a resource row selects a page");
        };
        let used_on: Vec<(&str, Option<&str>)> = references
            .iter()
            .map(|reference| (reference.page_url.as_str(), reference.alt.as_deref()))
            .collect();
        assert_eq!(
            used_on,
            vec![
                ("https://a.test/one", Some("Product")),
                ("https://a.test/three", None),
            ]
        );
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

    fn entry_names(entries: &[IssueEntry]) -> Vec<(String, usize)> {
        entries
            .iter()
            .map(|entry| (entry.name.clone(), entry.count))
            .collect()
    }

    /// Rebuilding the issue entries for every streamed batch is what froze the
    /// window on the Overview during a large crawl, so the grid shows the
    /// latest counted entries and the counting pass brings them up to date.
    #[test]
    fn a_streamed_page_reaches_the_overview_with_the_next_count() {
        let mut delegate = ResultsDelegate::new().count_in_background();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages());
        let before = entry_names(delegate.issue_entries());

        delegate.push(PageRecord {
            url: "https://a.test/untitled".into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            word_count: Some(10),
            ..Default::default()
        });
        assert_eq!(
            entry_names(delegate.issue_entries()),
            before,
            "a streamed page waits for the next count"
        );

        let job = delegate.counts_job().expect("the counts are behind");
        delegate.finish_counts(job.run());
        let expected = build_issues_entries(delegate.all_pages_for_test());
        assert_ne!(
            entry_names(&expected),
            before,
            "the page should change a rule"
        );
        assert_eq!(
            entry_names(delegate.issue_entries()),
            entry_names(&expected)
        );
        assert_eq!(delegate.flat_rows().len(), expected.len());
        assert!(delegate.counts_are_current());
    }

    /// Exports and the report build a delegate of their own and read it at
    /// once, so without the grid's background counting a push is reflected
    /// straight away.
    #[test]
    fn without_background_counting_a_push_rebuilds_the_overview() {
        let mut delegate = ResultsDelegate::new();
        delegate.switch_tab(ResultTab::Overview);
        delegate.replace_records(pages());
        delegate.push(PageRecord {
            url: "https://a.test/untitled".into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            word_count: Some(10),
            ..Default::default()
        });
        let expected = build_issues_entries(delegate.all_pages_for_test());
        assert_eq!(
            entry_names(delegate.issue_entries()),
            entry_names(&expected)
        );
    }

    /// A pass still running when another crawl is loaded describes the old
    /// one, and must not land on the new one.
    #[test]
    fn a_count_of_a_replaced_crawl_is_dropped() {
        let mut delegate = ResultsDelegate::new().count_in_background();
        delegate.replace_records(pages());
        let stale = delegate.counts_job().expect("nothing is counted yet");

        delegate.clear();
        delegate.push(PageRecord {
            url: "https://b.test/".into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            ..Default::default()
        });
        delegate.finish_counts(stale.run());
        assert!(delegate.displayed_counts().is_none());

        let job = delegate.counts_job().expect("the new crawl is not counted");
        delegate.finish_counts(job.run());
        let internal = delegate
            .displayed_counts()
            .and_then(|counts| counts.get(&ResultTab::Internal))
            .map(|counts| counts.badge.total);
        assert_eq!(internal, Some(1));
    }

    /// A pass that lands after more pages arrived is shown, since it is newer
    /// than what is on screen, but the next one still has to run.
    #[test]
    fn a_count_that_missed_new_pages_is_shown_but_not_current() {
        let mut delegate = ResultsDelegate::new().count_in_background();
        delegate.replace_records(pages());
        let job = delegate.counts_job().expect("nothing is counted yet");
        delegate.push(PageRecord {
            url: "https://a.test/late".into(),
            is_internal: true,
            is_page: true,
            status: Some(200),
            ..Default::default()
        });
        delegate.finish_counts(job.run());
        assert!(delegate.displayed_counts().is_some());
        assert!(!delegate.counts_are_current());

        let job = delegate.counts_job().expect("the late page is not counted");
        delegate.finish_counts(job.run());
        assert!(delegate.counts_are_current());
        let internal = delegate
            .displayed_counts()
            .and_then(|counts| counts.get(&ResultTab::Internal))
            .map(|counts| counts.badge.total);
        assert_eq!(internal, Some(pages().len() + 1));
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
