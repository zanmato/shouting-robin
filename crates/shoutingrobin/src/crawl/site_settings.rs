//! What the crawl settings panel holds for one site, and the checks its
//! patterns and headers pass before a crawl may start.

use regex::Regex;
use url::Url;

/// The panel's settings as they are remembered for a site and restored on its
/// next crawl.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SiteSettings {
    pub crawl_subdomains: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    /// Zero crawls every page.
    pub max_pages: u32,
    pub max_concurrent: u32,
    pub block_images: bool,
    pub capture_layout: bool,
    pub scroll_for_layout: bool,
    /// Header values are often credentials, so only the names are kept and
    /// the values are asked for again.
    pub header_names: Vec<String>,
}

/// The key settings are remembered under: the host, with its port when it
/// has one. An address typed without a scheme is read as https, the way the
/// crawl reads it.
pub fn site_host(address: &str) -> Option<String> {
    let address = address.trim();
    if address.is_empty() {
        return None;
    }
    let parsed = Url::parse(address)
        .ok()
        .filter(|url| url.has_host())
        .or_else(|| Url::parse(&format!("https://{address}")).ok())?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

/// A line of the panel that would keep the crawl from doing what it says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputProblem {
    pub line: String,
    pub message: String,
}

/// One regex per line, as the include or exclude field holds them.
#[derive(Clone, Debug, Default)]
pub struct PatternList {
    patterns: Vec<(String, Regex)>,
    pub problems: Vec<InputProblem>,
}

impl PatternList {
    /// Spider compiles every line into one set and, when any line fails,
    /// drops the whole list without a word. So every line is checked here
    /// with the same regex engine, and a crawl with a problem does not start.
    pub fn parse(text: &str) -> Self {
        let mut list = Self::default();
        for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            match Regex::new(line) {
                Ok(regex) => list.patterns.push((line.to_string(), regex)),
                Err(error) => list.problems.push(InputProblem {
                    line: line.to_string(),
                    message: regex_problem(&error),
                }),
            }
        }
        list
    }

    pub fn len(&self) -> usize {
        self.patterns.len() + self.problems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Spider matches the whole URL, anywhere in it.
    fn first_match(&self, url: &str) -> Option<&str> {
        self.patterns
            .iter()
            .find(|(_, regex)| regex.is_match(url))
            .map(|(source, _)| source.as_str())
    }
}

/// The regex crate's syntax errors draw the pattern with a caret under the
/// fault. Only the closing `error:` line fits under a text field.
fn regex_problem(error: &regex::Error) -> String {
    let reason = match error {
        regex::Error::Syntax(text) => text
            .lines()
            .rev()
            .find_map(|line| line.trim().strip_prefix("error: "))
            .map(str::to_string),
        regex::Error::CompiledTooBig(_) => Some("the pattern is too large".to_string()),
        _ => None,
    };
    match reason {
        Some(reason) => format!("Not a valid pattern: {reason}"),
        None => "Not a valid pattern".to_string(),
    }
}

/// Whether a URL is crawled under the include and exclude patterns, and which
/// pattern decided it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UrlVerdict<'a> {
    Crawled { included_by: Option<&'a str> },
    Excluded { by: &'a str },
    NotIncluded,
}

/// Spider skips a URL that any exclude pattern matches, and, when there are
/// include patterns, one that none of them match.
pub fn url_verdict<'a>(
    url: &str,
    include: &'a PatternList,
    exclude: &'a PatternList,
) -> UrlVerdict<'a> {
    if let Some(by) = exclude.first_match(url) {
        return UrlVerdict::Excluded { by };
    }
    if include.patterns.is_empty() {
        return UrlVerdict::Crawled { included_by: None };
    }
    match include.first_match(url) {
        Some(pattern) => UrlVerdict::Crawled {
            included_by: Some(pattern),
        },
        None => UrlVerdict::NotIncluded,
    }
}

/// Why a header row would not be sent as typed, or `None` when it would be.
/// An empty row is left out, so it is not a problem.
pub fn header_problem(name: &str, value: &str) -> Option<String> {
    let name = name.trim();
    let value = value.trim();
    if name.is_empty() {
        return (!value.is_empty()).then(|| "Add a name for this header".to_string());
    }
    if reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err() {
        return Some(if name.contains(' ') {
            format!(
                "Header names can't contain spaces. Did you mean {}?",
                name.split_whitespace().collect::<Vec<_>>().join("-")
            )
        } else {
            format!("{name} isn't a valid header name")
        });
    }
    if value.is_empty() {
        return Some(format!("Add a value for {name}, or remove it"));
    }
    if reqwest::header::HeaderValue::from_str(value).is_err() {
        return Some(format!(
            "The value for {name} can't contain line breaks or control characters"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_site_is_the_host_with_its_port() {
        assert_eq!(
            site_host("https://Books.ToScrape.com/catalogue/"),
            Some("books.toscrape.com".to_string())
        );
        assert_eq!(
            site_host("books.toscrape.com/catalogue"),
            Some("books.toscrape.com".to_string())
        );
        assert_eq!(
            site_host("http://localhost:8080/"),
            Some("localhost:8080".to_string())
        );
        assert_eq!(site_host("   "), None);
    }

    #[test]
    fn one_bad_line_is_reported_and_the_rest_still_match() {
        let exclude = PatternList::parse("/tag/\n\n  \\?sort=[  \n");
        assert_eq!(exclude.len(), 2);
        assert_eq!(exclude.problems.len(), 1);
        assert_eq!(exclude.problems[0].line, "\\?sort=[");
        assert_eq!(
            exclude.problems[0].message,
            "Not a valid pattern: unclosed character class"
        );
        let include = PatternList::default();
        assert_eq!(
            url_verdict("https://a.com/tag/x", &include, &exclude),
            UrlVerdict::Excluded { by: "/tag/" }
        );
    }

    #[test]
    fn include_patterns_keep_only_what_they_match() {
        let include = PatternList::parse("/catalogue/");
        let exclude = PatternList::parse("/catalogue/category/");
        assert_eq!(
            url_verdict("https://a.com/catalogue/book_1/", &include, &exclude),
            UrlVerdict::Crawled {
                included_by: Some("/catalogue/")
            }
        );
        assert_eq!(
            url_verdict(
                "https://a.com/catalogue/category/travel/",
                &include,
                &exclude
            ),
            UrlVerdict::Excluded {
                by: "/catalogue/category/"
            }
        );
        assert_eq!(
            url_verdict("https://a.com/about/", &include, &exclude),
            UrlVerdict::NotIncluded
        );
    }

    #[test]
    fn header_rows_are_checked_the_way_they_are_sent() {
        assert_eq!(header_problem("", ""), None);
        assert_eq!(header_problem("Authorization", "Bearer abc"), None);
        assert_eq!(
            header_problem("Accept Language", "sv-SE").as_deref(),
            Some("Header names can't contain spaces. Did you mean Accept-Language?")
        );
        assert_eq!(
            header_problem("Authorization", "").as_deref(),
            Some("Add a value for Authorization, or remove it")
        );
        assert_eq!(
            header_problem("", "token").as_deref(),
            Some("Add a name for this header")
        );
        assert!(header_problem("X-Token", "a\u{7}b").is_some());
    }
}
