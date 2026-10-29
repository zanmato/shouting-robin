//! A search and column conditions that narrow a tab's rows further than its
//! sub-filter. Rows are matched on the text their cells show, so what can be
//! seen in the grid is what can be found.

use gpui_kit::SharedString;

use super::columns::{is_numeric_column, parse_numeric};

/// How a condition compares a cell with its value. Text comparisons ignore
/// case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterOperator {
    Contains,
    NotContains,
    Equals,
    NotEquals,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
    StartsWith,
    EndsWith,
    /// A comma separated list of values, any of which the cell equals.
    In,
    NotIn,
    IsEmpty,
    IsNotEmpty,
}

impl FilterOperator {
    pub const ALL: [FilterOperator; 14] = [
        Self::Contains,
        Self::NotContains,
        Self::Equals,
        Self::NotEquals,
        Self::Less,
        Self::LessOrEqual,
        Self::Greater,
        Self::GreaterOrEqual,
        Self::StartsWith,
        Self::EndsWith,
        Self::In,
        Self::NotIn,
        Self::IsEmpty,
        Self::IsNotEmpty,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::NotContains => "does not contain",
            Self::Equals => "=",
            Self::NotEquals => "≠",
            Self::Less => "<",
            Self::LessOrEqual => "≤",
            Self::Greater => ">",
            Self::GreaterOrEqual => "≥",
            Self::StartsWith => "starts with",
            Self::EndsWith => "ends with",
            Self::In => "is one of",
            Self::NotIn => "is none of",
            Self::IsEmpty => "is empty",
            Self::IsNotEmpty => "is not empty",
        }
    }

    pub fn takes_value(self) -> bool {
        !matches!(self, Self::IsEmpty | Self::IsNotEmpty)
    }
}

/// How the conditions of a filter combine.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FilterJoin {
    #[default]
    All,
    Any,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilterCondition {
    pub column: SharedString,
    pub operator: FilterOperator,
    pub value: String,
}

/// What narrows one tab's rows: every word of the search has to appear in
/// some text column, and the conditions have to hold as `join` says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowFilter {
    pub search: String,
    pub join: FilterJoin,
    pub conditions: Vec<FilterCondition>,
}

impl RowFilter {
    /// The filter as it matches rows, or `None` when it would keep them all.
    /// A condition still waiting for its value is left out rather than
    /// matching nothing, so adding a row does not empty the grid.
    pub(super) fn compile(&self, columns: &[SharedString]) -> Option<CompiledFilter> {
        let words: Vec<String> = self
            .search
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        let conditions: Vec<CompiledCondition> = self
            .conditions
            .iter()
            .filter(|condition| columns.contains(&condition.column))
            .filter(|condition| {
                !condition.operator.takes_value() || !condition.value.trim().is_empty()
            })
            .map(CompiledCondition::new)
            .collect();
        if words.is_empty() && conditions.is_empty() {
            return None;
        }
        let search_columns = if words.is_empty() {
            Vec::new()
        } else {
            columns
                .iter()
                .filter(|key| !is_numeric_column(key))
                .cloned()
                .collect()
        };
        Some(CompiledFilter {
            words,
            search_columns,
            join: self.join,
            conditions,
        })
    }
}

pub(super) struct CompiledFilter {
    words: Vec<String>,
    search_columns: Vec<SharedString>,
    join: FilterJoin,
    conditions: Vec<CompiledCondition>,
}

impl CompiledFilter {
    /// Whether the row whose cells `text_of` gives is kept.
    pub(super) fn matches<T: AsRef<str>>(&self, mut text_of: impl FnMut(&str) -> T) -> bool {
        if !self.conditions.is_empty() {
            let mut results = self
                .conditions
                .iter()
                .map(|condition| condition.matches(text_of(&condition.column).as_ref()));
            let kept = match self.join {
                FilterJoin::All => results.all(|matched| matched),
                FilterJoin::Any => results.any(|matched| matched),
            };
            if !kept {
                return false;
            }
        }
        if self.words.is_empty() {
            return true;
        }
        let mut found = vec![false; self.words.len()];
        let mut remaining = self.words.len();
        for column in &self.search_columns {
            let text = text_of(column);
            for (word, found) in self.words.iter().zip(found.iter_mut()) {
                if !*found && contains_ignoring_case(text.as_ref(), word) {
                    *found = true;
                    remaining -= 1;
                }
            }
            if remaining == 0 {
                return true;
            }
        }
        false
    }
}

struct CompiledCondition {
    column: SharedString,
    operator: FilterOperator,
    numeric_column: bool,
    /// Lowercased, and split on commas for the list operators.
    values: Vec<String>,
    number: Option<f64>,
}

impl CompiledCondition {
    fn new(condition: &FilterCondition) -> Self {
        let value = condition.value.trim();
        let values = match condition.operator {
            FilterOperator::In | FilterOperator::NotIn => value
                .split(',')
                .map(|item| item.trim().to_lowercase())
                .filter(|item| !item.is_empty())
                .collect(),
            _ => vec![value.to_lowercase()],
        };
        Self {
            column: condition.column.clone(),
            operator: condition.operator,
            numeric_column: is_numeric_column(&condition.column),
            values,
            number: parse_numeric(value),
        }
    }

    fn matches(&self, text: &str) -> bool {
        let text = text.trim();
        let value = self.values.first().map(String::as_str).unwrap_or_default();
        match self.operator {
            FilterOperator::Contains => contains_ignoring_case(text, value),
            FilterOperator::NotContains => !contains_ignoring_case(text, value),
            FilterOperator::Equals => self.equals(text, value),
            FilterOperator::NotEquals => !self.equals(text, value),
            FilterOperator::StartsWith => text.to_lowercase().starts_with(value),
            FilterOperator::EndsWith => text.to_lowercase().ends_with(value),
            FilterOperator::In => self.values.iter().any(|value| self.equals(text, value)),
            FilterOperator::NotIn => !self.values.iter().any(|value| self.equals(text, value)),
            FilterOperator::IsEmpty => text.is_empty(),
            FilterOperator::IsNotEmpty => !text.is_empty(),
            FilterOperator::Less => self.compare(text, |cell, value| cell < value),
            FilterOperator::LessOrEqual => self.compare(text, |cell, value| cell <= value),
            FilterOperator::Greater => self.compare(text, |cell, value| cell > value),
            FilterOperator::GreaterOrEqual => self.compare(text, |cell, value| cell >= value),
        }
    }

    /// Numbers compare as numbers in a numeric column, so `200` equals a
    /// cell showing `200.0` and `500` a cell showing `500ms`.
    fn equals(&self, text: &str, value: &str) -> bool {
        if self.numeric_column
            && let (Some(cell), Some(value)) = (parse_numeric(text), parse_numeric(value))
        {
            return cell == value;
        }
        text.to_lowercase() == value
    }

    /// An ordering holds only between two numbers: an empty cell is neither
    /// below nor above a value.
    fn compare(&self, text: &str, holds: impl Fn(f64, f64) -> bool) -> bool {
        match (parse_numeric(text), self.number) {
            (Some(cell), Some(value)) => holds(cell, value),
            _ => false,
        }
    }
}

/// `needle` is already lowercase. Most cells are ASCII, which is compared in
/// place rather than lowercasing a copy of every cell.
fn contains_ignoring_case(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle.is_ascii() {
        let needle = needle.as_bytes();
        return haystack
            .as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle));
    }
    haystack.to_lowercase().contains(needle)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn columns() -> Vec<SharedString> {
        ["address", "title", "status_code", "response_time"]
            .into_iter()
            .map(SharedString::from)
            .collect()
    }

    fn row(cells: &[(&str, &str)]) -> HashMap<String, String> {
        cells
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    fn keeps(filter: &RowFilter, row: &HashMap<String, String>) -> bool {
        filter.compile(&columns()).is_none_or(|compiled| {
            compiled.matches(|key| row.get(key).cloned().unwrap_or_default())
        })
    }

    fn condition(column: &str, operator: FilterOperator, value: &str) -> FilterCondition {
        FilterCondition {
            column: column.into(),
            operator,
            value: value.into(),
        }
    }

    #[test]
    fn every_search_word_has_to_appear_in_some_column() {
        let page = row(&[("address", "/blog/2023/launch"), ("title", "Our Launch")]);
        let search = |text: &str| RowFilter {
            search: text.into(),
            ..Default::default()
        };
        assert!(keeps(&search("BLOG launch"), &page));
        assert!(keeps(&search("our 2023"), &page));
        assert!(!keeps(&search("blog shoes"), &page));
    }

    #[test]
    fn search_skips_number_columns() {
        let page = row(&[("address", "/a"), ("status_code", "200")]);
        let filter = RowFilter {
            search: "200".into(),
            ..Default::default()
        };
        assert!(!keeps(&filter, &page));
    }

    #[test]
    fn numbers_compare_as_numbers() {
        let slow = row(&[("response_time", "812ms"), ("status_code", "200")]);
        let fast = row(&[("response_time", "90ms"), ("status_code", "200")]);
        let filter = RowFilter {
            conditions: vec![condition("response_time", FilterOperator::Greater, "500")],
            ..Default::default()
        };
        assert!(keeps(&filter, &slow));
        assert!(!keeps(&filter, &fast));
        assert!(!keeps(&filter, &row(&[("response_time", "")])));
    }

    #[test]
    fn conditions_join_as_all_or_any() {
        let page = row(&[("status_code", "404"), ("title", "Missing")]);
        let mut filter = RowFilter {
            conditions: vec![
                condition("status_code", FilterOperator::In, "301, 404"),
                condition("title", FilterOperator::StartsWith, "home"),
            ],
            ..Default::default()
        };
        assert!(!keeps(&filter, &page));
        filter.join = FilterJoin::Any;
        assert!(keeps(&filter, &page));
    }

    #[test]
    fn a_condition_without_its_value_keeps_every_row() {
        let filter = RowFilter {
            conditions: vec![condition("title", FilterOperator::Equals, " ")],
            ..Default::default()
        };
        assert!(filter.compile(&columns()).is_none());
        let empty = RowFilter {
            conditions: vec![condition("title", FilterOperator::IsEmpty, "")],
            ..Default::default()
        };
        assert!(keeps(&empty, &row(&[("title", "")])));
        assert!(!keeps(&empty, &row(&[("title", "Home")])));
    }

    #[test]
    fn a_condition_on_a_column_the_tab_lacks_is_left_out() {
        let filter = RowFilter {
            conditions: vec![condition("lcp", FilterOperator::Greater, "1")],
            ..Default::default()
        };
        assert!(filter.compile(&columns()).is_none());
    }

    #[test]
    fn text_matching_ignores_case_beyond_ascii() {
        let page = row(&[("title", "Ölkorv och Äpple")]);
        let filter = RowFilter {
            conditions: vec![
                condition("title", FilterOperator::Contains, "äpple"),
                condition("title", FilterOperator::EndsWith, "ÄPPLE"),
                condition("title", FilterOperator::StartsWith, "öl"),
            ],
            ..Default::default()
        };
        assert!(keeps(&filter, &page));
    }
}
