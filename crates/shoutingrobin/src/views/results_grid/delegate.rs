use std::collections::HashMap;

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
use super::filter::{filter_for_tab, flat_row_matches_filter};
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
        self.root_origin = url::Url::parse(root_url)
            .ok()
            .map(|u| u.origin().ascii_serialization());
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
                FlatRow::OverviewIssue { .. }
                | FlatRow::IssuesRow { .. }
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
        let internal: Vec<&PageRecord> = self.all_pages.iter().filter(|p| p.is_internal).collect();

        let errors = self
            .all_pages
            .iter()
            .filter(|p| p.status.is_some_and(|c| c >= 400))
            .count();

        let non_indexable = internal
            .iter()
            .filter(|p| p.indexability.as_deref() == Some("Non-Indexable"))
            .count();

        let missing_title = internal
            .iter()
            .filter(|p| p.title.as_deref() == Some(""))
            .count();
        let duplicate_title = {
            let mut title_counts: HashMap<&str, usize> = HashMap::new();
            for p in &internal {
                let t = p.title.as_deref().unwrap_or("");
                *title_counts.entry(t).or_insert(0) += 1;
            }
            internal
                .iter()
                .filter(|p| {
                    *title_counts
                        .get(p.title.as_deref().unwrap_or(""))
                        .unwrap_or(&0)
                        > 1
                })
                .count()
        };

        let over_length_title = internal
            .iter()
            .filter(|p| p.title.as_deref().is_some_and(|t| t.len() > 60))
            .count();

        let missing_desc = internal
            .iter()
            .filter(|p| p.meta_description.as_deref() == Some(""))
            .count();
        let over_length_desc = internal
            .iter()
            .filter(|p| p.meta_description.as_deref().is_some_and(|t| t.len() > 160))
            .count();
        let missing_h1 = internal
            .iter()
            .filter(|p| p.h1.as_deref() == Some(""))
            .count();
        let over_length_h1 = internal
            .iter()
            .filter(|p| p.h1.as_deref().is_some_and(|t| t.len() > 70))
            .count();
        let missing_h2 = internal
            .iter()
            .filter(|p| p.h2.as_deref() == Some(""))
            .count();
        let over_length_h2 = internal
            .iter()
            .filter(|p| p.h2.as_deref().is_some_and(|t| t.len() > 70))
            .count();
        let missing_canonical = internal
            .iter()
            .filter(|p| p.canonical.as_deref() == Some(""))
            .count();

        let mut counts = HashMap::new();
        counts.insert(
            ResultTab::Internal,
            TabCounts {
                total: internal.len(),
                errors,
                warnings: non_indexable,
            },
        );
        counts.insert(
            ResultTab::External,
            TabCounts {
                total: internal
                    .iter()
                    .map(|p| {
                        p.outlinks
                            .iter()
                            .filter(|link| !is_same_domain(&p.url, &link.dst_url))
                            .count()
                    })
                    .sum(),
                errors: 0,
                warnings: 0,
            },
        );
        counts.insert(
            ResultTab::ResponseCodes,
            TabCounts {
                total: self.all_pages.len(),
                errors,
                warnings: self
                    .all_pages
                    .iter()
                    .filter(|p| p.redirect_url.is_some())
                    .count(),
            },
        );
        counts.insert(
            ResultTab::PageTitles,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_title + duplicate_title + over_length_title,
            },
        );
        counts.insert(
            ResultTab::MetaDesc,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_desc + over_length_desc,
            },
        );
        counts.insert(
            ResultTab::H1,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_h1 + over_length_h1,
            },
        );
        counts.insert(
            ResultTab::H2,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_h2 + over_length_h2,
            },
        );
        let exact_dup_count = {
            let mut hash_counts: HashMap<&str, usize> = HashMap::new();
            for p in &internal {
                if let Some(hash) = p.content_hash.as_deref() {
                    *hash_counts.entry(hash).or_insert(0) += 1;
                }
            }
            internal
                .iter()
                .filter(|p| {
                    p.content_hash
                        .as_deref()
                        .is_some_and(|h| *hash_counts.get(h).unwrap_or(&0) > 1)
                })
                .count()
        };
        let near_dup_count = internal
            .iter()
            .filter(|p| p.near_duplicate_count.is_some_and(|c| c > 0))
            .count();
        counts.insert(
            ResultTab::Content,
            TabCounts {
                total: internal.len(),
                errors: exact_dup_count,
                warnings: near_dup_count,
            },
        );
        counts.insert(
            ResultTab::Images,
            TabCounts {
                total: internal.iter().map(|p| p.images.len()).sum(),
                errors: 0,
                warnings: internal
                    .iter()
                    .flat_map(|p| p.images.iter())
                    .filter(|img| {
                        !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty())
                    })
                    .count(),
            },
        );
        counts.insert(
            ResultTab::Canonicals,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: missing_canonical,
            },
        );
        counts.insert(
            ResultTab::Hreflang,
            TabCounts {
                total: internal.len(),
                errors: 0,
                warnings: internal
                    .iter()
                    .filter(|p| p.hreflang_tags.is_empty())
                    .count(),
            },
        );
        let sd_missing = internal.iter().filter(|p| p.sd_types.is_empty()).count();
        let sd_error_count = internal.iter().filter(|p| p.sd_errors > 0).count();
        counts.insert(
            ResultTab::StructuredData,
            TabCounts {
                total: internal.len(),
                errors: sd_error_count,
                warnings: sd_missing,
            },
        );
        counts.insert(
            ResultTab::Performance,
            TabCounts {
                total: internal.len(),
                errors: internal
                    .iter()
                    .filter(|p| p.lcp_ms.is_some_and(|ms| ms > 4000))
                    .count(),
                warnings: internal
                    .iter()
                    .filter(|p| {
                        p.lcp_ms.is_some_and(|ms| ms > 2500 && ms <= 4000)
                            || p.cls.is_some_and(|v| v > 0.1)
                            || p.inp_ms.is_some_and(|ms| ms > 200)
                    })
                    .count(),
            },
        );
        counts.insert(
            ResultTab::Accessibility,
            TabCounts {
                total: self
                    .all_pages
                    .iter()
                    .filter(|p| p.is_internal)
                    .map(|p| p.a11y_issues.len())
                    .sum(),
                errors: self
                    .all_pages
                    .iter()
                    .flat_map(|p| p.a11y_issues.iter())
                    .filter(|i| matches!(i.impact.as_str(), "critical" | "serious"))
                    .count(),
                warnings: self
                    .all_pages
                    .iter()
                    .flat_map(|p| p.a11y_issues.iter())
                    .filter(|i| !matches!(i.impact.as_str(), "critical" | "serious"))
                    .count(),
            },
        );
        let product_count = self
            .all_pages
            .iter()
            .filter(|p| p.ecommerce.is_some())
            .count();
        let missing_price = self
            .all_pages
            .iter()
            .filter(|p| p.ecommerce.as_ref().is_some_and(|a| a.price.is_none()))
            .count();
        counts.insert(
            ResultTab::Ecommerce,
            TabCounts {
                total: product_count,
                errors: 0,
                warnings: missing_price,
            },
        );

        use super::columns::{header_exists, header_value};
        use super::data_build::directory_path;

        let sitemap_orphan_count = self
            .all_pages
            .iter()
            .filter(|p| p.in_sitemap == Some(true) && p.status.is_none())
            .count();
        let non_indexable_in_sitemap = self
            .all_pages
            .iter()
            .filter(|p| {
                p.in_sitemap == Some(true) && p.indexability.as_deref() == Some("Non-Indexable")
            })
            .count();
        counts.insert(
            ResultTab::Sitemaps,
            TabCounts {
                total: self
                    .all_pages
                    .iter()
                    .filter(|p| p.in_sitemap.is_some())
                    .count(),
                errors: sitemap_orphan_count,
                warnings: non_indexable_in_sitemap,
            },
        );
        let missing_https = internal
            .iter()
            .filter(|p| !p.url.starts_with("https://"))
            .count();
        let missing_hsts = internal
            .iter()
            .filter(|p| !header_exists(&p.headers, "strict-transport-security"))
            .count();
        counts.insert(
            ResultTab::Security,
            TabCounts {
                total: internal.len(),
                errors: missing_https,
                warnings: missing_hsts,
            },
        );
        let url_non_ascii = internal.iter().filter(|p| !p.url.is_ascii()).count();
        let url_uppercase = internal
            .iter()
            .filter(|p| p.url.chars().any(|c| c.is_ascii_uppercase()))
            .count();
        counts.insert(
            ResultTab::Url,
            TabCounts {
                total: internal.len(),
                errors: url_non_ascii + url_uppercase,
                warnings: internal.iter().filter(|p| p.url.contains('_')).count(),
            },
        );
        let directive_noindex = internal
            .iter()
            .filter(|p| {
                p.robots
                    .as_deref()
                    .is_some_and(|r| r.to_ascii_lowercase().contains("noindex"))
                    || header_value(&p.headers, "x-robots-tag")
                        .is_some_and(|v| v.to_ascii_lowercase().contains("noindex"))
            })
            .count();
        counts.insert(
            ResultTab::Directives,
            TabCounts {
                total: internal.len(),
                errors: directive_noindex,
                warnings: 0,
            },
        );
        let overview_entries = build_issues_entries(&self.all_pages);
        let overview_errors = overview_entries
            .iter()
            .filter(|e| e.issue_type == IssueType::Issue)
            .count();
        let overview_warnings = overview_entries.len() - overview_errors;
        counts.insert(
            ResultTab::Overview,
            TabCounts {
                total: overview_entries.len(),
                errors: overview_errors,
                warnings: overview_warnings,
            },
        );
        let internal_links: usize = self
            .all_pages
            .iter()
            .filter(|p| p.is_internal)
            .map(|p| {
                p.outlinks
                    .iter()
                    .filter(|l| is_same_domain(&p.url, &l.dst_url))
                    .count()
            })
            .sum();
        counts.insert(
            ResultTab::Links,
            TabCounts {
                total: internal_links,
                errors: 0,
                warnings: 0,
            },
        );

        let unique_dirs: std::collections::HashSet<String> = internal
            .iter()
            .filter_map(|p| {
                let path = p.url.strip_prefix(self.root_origin.as_deref()?)?;
                Some(directory_path(path))
            })
            .collect();
        counts.insert(
            ResultTab::SiteStructure,
            TabCounts {
                total: unique_dirs.len(),
                errors: 0,
                warnings: 0,
            },
        );

        counts
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
        self.flat_rows.retain(|row| {
            let page_index = match row {
                FlatRow::Image { page, .. }
                | FlatRow::Outlink { page, .. }
                | FlatRow::A11yIssue { page, .. }
                | FlatRow::Hreflang { page, .. }
                | FlatRow::SdItem { page, .. }
                | FlatRow::LinkRow { page, .. } => *page,
                FlatRow::OverviewIssue { .. }
                | FlatRow::IssuesRow { .. }
                | FlatRow::DirectoryAggregate { .. } => return true,
            };
            let Some(page) = self.all_pages.get(page_index) else {
                return false;
            };
            flat_row_matches_filter(row, page, self.issue_filter)
        });
    }

    pub fn count_for_filter(&self, filter: IssueFilter) -> usize {
        let indices = filter_for_tab(
            self.active_tab,
            filter,
            &self.all_pages,
            &self.occurrence_counts,
        );
        if self.active_tab == ResultTab::Overview {
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
        if tab_is_flattened(self.active_tab) {
            if filter == IssueFilter::All {
                indices
                    .iter()
                    .map(|&page_ix| {
                        self.all_pages
                            .get(page_ix)
                            .map(|p| flat_row_item_count(p, self.active_tab))
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
                                let item_count = flat_row_item_count(p, self.active_tab);
                                (0..item_count)
                                    .filter(|item| {
                                        let row = flat_row_variant(self.active_tab, page_ix, *item);
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
                FlatRow::OverviewIssue { label, count } => {
                    let text = match key.as_ref() {
                        "issue" => SharedString::from(label.clone()),
                        "count" => SharedString::from(count.to_string()),
                        _ => SharedString::default(),
                    };
                    if key.as_ref() == "count" {
                        let tone = if *count > 0 { Tone::Warn } else { Tone::Ok };
                        cell.child(tone_tag(tone).child(text))
                    } else {
                        cell.child(text)
                    }
                }
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
                        FlatRow::OverviewIssue { .. }
                        | FlatRow::IssuesRow { .. }
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
                if let (
                    FlatRow::OverviewIssue {
                        label: a_label,
                        count: a_count,
                    },
                    FlatRow::OverviewIssue {
                        label: b_label,
                        count: b_count,
                    },
                ) = (a, b)
                {
                    let ordering = match col_key.as_ref() {
                        "count" => a_count.cmp(b_count),
                        _ => a_label.cmp(b_label),
                    };
                    return match sort {
                        ColumnSort::Descending => ordering.reverse(),
                        _ => ordering,
                    };
                }
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
                    FlatRow::OverviewIssue { .. }
                    | FlatRow::IssuesRow { .. }
                    | FlatRow::DirectoryAggregate { .. } => 0,
                };

                let b_page = match b {
                    FlatRow::Image { page, .. }
                    | FlatRow::Outlink { page, .. }
                    | FlatRow::A11yIssue { page, .. }
                    | FlatRow::Hreflang { page, .. }
                    | FlatRow::SdItem { page, .. }
                    | FlatRow::LinkRow { page, .. } => *page,
                    FlatRow::OverviewIssue { .. }
                    | FlatRow::IssuesRow { .. }
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
