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

/// Decodes the character references an HTML author writes inside an attribute
/// value, so a href of `?a=1&amp;b=2` becomes the URL `?a=1&b=2`.
///
/// HTML requires `&` to be escaped in attribute values, and browsers decode it
/// before requesting the URL. A crawler that skips this step requests a URL
/// nobody linked to: it invents `?a=1&amp;b=2` (a query with an `amp;b`
/// parameter) and never visits the real page.
///
/// Only the five predefined entities and numeric references are decoded. An
/// unrecognised sequence such as `&sect` is left exactly as it is, because in a
/// query string it is far more likely to be a parameter named `sect` than an
/// author's attempt to write `§`.
pub fn decode_entities(url: &str) -> String {
    match quick_xml::escape::unescape(url) {
        Ok(decoded) => decoded.into_owned(),
        Err(_) => url.to_string(),
    }
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

    #[test]
    fn decodes_escaped_ampersands_in_a_query_string() {
        assert_eq!(
            decode_entities("/list?f[serie]=Touch&amp;page=1"),
            "/list?f[serie]=Touch&page=1"
        );
    }

    #[test]
    fn decodes_numeric_character_references() {
        assert_eq!(decode_entities("/a?x=1&#38;y=2"), "/a?x=1&y=2");
        assert_eq!(decode_entities("/a?x=1&#x26;y=2"), "/a?x=1&y=2");
    }

    #[test]
    fn leaves_a_plain_url_untouched() {
        let url = "https://example.com/a?x=1&y=2";
        assert_eq!(decode_entities(url), url);
    }

    #[test]
    fn leaves_an_unrecognised_sequence_untouched() {
        // `sect` here is a query parameter, not an attempt to write a section sign.
        let url = "https://example.com/a?x=1&sect=news";
        assert_eq!(decode_entities(url), url);
    }
}
