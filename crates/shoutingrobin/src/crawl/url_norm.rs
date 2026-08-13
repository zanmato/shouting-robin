//! URL normalisation for comparing two URLs that address the same resource.
//!
//! Crawlers see the same page written several ways: `https://example.com` and
//! `https://example.com/` differ as strings but not as resources, and a site
//! that emits its canonical without the trailing slash would otherwise look
//! like it canonicalises every page somewhere else. Only differences the URL
//! standard itself calls insignificant are erased here — scheme and host case,
//! the default port, an empty path, and the fragment. Path case and trailing
//! slashes on non-empty paths are left alone, because servers are free to treat
//! `/a` and `/a/` as different pages.

/// Returns a canonical string form of `url`, or `None` when it doesn't parse.
pub fn normalize_url(url: &str) -> Option<String> {
    let mut parsed = url::Url::parse(url.trim()).ok()?;
    parsed.set_fragment(None);
    // `Url` already lowercases the scheme and host, drops the default port and
    // rewrites an empty path as "/", so serialising is the normalisation.
    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// True when both URLs address the same resource once normalised. Falls back
    /// to exact string comparison when either side fails to parse, so an
    /// unparseable value is never silently treated as equal to something else.
    fn urls_equivalent(left: &str, right: &str) -> bool {
        match (normalize_url(left), normalize_url(right)) {
            (Some(left), Some(right)) => left == right,
            _ => left.trim() == right.trim(),
        }
    }

    #[test]
    fn empty_path_is_equivalent_to_root() {
        assert!(urls_equivalent(
            "https://example.com",
            "https://example.com/"
        ));
    }

    #[test]
    fn scheme_and_host_case_are_insignificant() {
        assert!(urls_equivalent(
            "HTTPS://Example.COM/a",
            "https://example.com/a"
        ));
    }

    #[test]
    fn default_port_is_insignificant() {
        assert!(urls_equivalent(
            "https://example.com:443/a",
            "https://example.com/a"
        ));
        assert!(urls_equivalent(
            "http://example.com:80/a",
            "http://example.com/a"
        ));
    }

    #[test]
    fn fragment_is_insignificant() {
        assert!(urls_equivalent(
            "https://example.com/a#section",
            "https://example.com/a"
        ));
    }

    #[test]
    fn trailing_slash_on_a_non_empty_path_is_significant() {
        assert!(!urls_equivalent(
            "https://example.com/a",
            "https://example.com/a/"
        ));
    }

    #[test]
    fn path_case_is_significant() {
        assert!(!urls_equivalent(
            "https://example.com/A",
            "https://example.com/a"
        ));
    }

    #[test]
    fn query_is_significant() {
        assert!(!urls_equivalent(
            "https://example.com/a?x=1",
            "https://example.com/a"
        ));
    }

    #[test]
    fn non_default_port_is_significant() {
        assert!(!urls_equivalent(
            "https://example.com:8443/a",
            "https://example.com/a"
        ));
    }

    #[test]
    fn unparseable_urls_fall_back_to_string_comparison() {
        assert!(urls_equivalent("not a url", "not a url"));
        assert!(!urls_equivalent("not a url", "https://example.com/"));
    }
}
