//! Readability characterization test against a real-world page.
//!
//! Fixture: an anonymized copy of a Swedish "forgot password" page. An
//! established SEO tool reports the following for the same SSR HTML:
//!
//!   Word count: 66, Sentence count: 13, Average words/sentence: 5.08,
//!   Reading score: 65.81
//!
//! This test pins what *we* currently produce so the gap is visible and any
//! change to the readability pipeline is intentional. The divergence has two
//! main causes, documented here for whoever tightens these numbers:
//!
//!   * Content scope: we score the whole <body>, including the (responsive,
//!     duplicated) nav, header and footer boilerplate, so our word count is
//!     ~2.3x the reference tool's main-content word count.
//!   * Sentence segmentation: we inject a synthetic sentence break at every
//!     block-element boundary, so each nav item / list row counts as a
//!     sentence, inflating the sentence count well past real punctuation.

use shoutingrobin::crawl::analyzers::analyze_html;
use shoutingrobin::crawl::event::PageRecord;

fn analyze_fixture(name: &str) -> PageRecord {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let html = std::fs::read_to_string(&path).expect("read fixture");
    let mut record = PageRecord::default();
    analyze_html(&mut record, &html, "");
    record
}

#[test]
fn swedish_forgot_password_readability() {
    let record = analyze_fixture("readability_swedish_forgot_password.html");

    // Current behavior (see module docs for the reference values of
    // 66 words / 13 sentences / 65.81). These assertions are expected to move
    // toward those targets as the pipeline improves.
    assert_eq!(record.word_count, Some(155));
    assert_eq!(record.sentence_count, Some(47));
    assert_eq!(record.flesch_reading_ease.map(|s| s.round()), Some(36.0));
}
