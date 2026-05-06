#[derive(Debug, Clone)]
pub struct SimilarityResult {
    pub url: String,
    pub closest_similarity_percent: u8,
    pub near_duplicate_count: u32,
}

pub fn find_near_duplicates(
    pages: &[(String, u64, Option<String>)],
    threshold_percent: u8,
) -> Vec<SimilarityResult> {
    if pages.len() < 2 {
        return Vec::new();
    }

    let max_differing_bits = ((100 - threshold_percent) as u32 * 64).div_ceil(100);

    let mut best_match = vec![0u8; pages.len()];
    let mut counts = vec![0u32; pages.len()];

    for index in 0..pages.len() {
        for other in (index + 1)..pages.len() {
            let hamming = (pages[index].1 ^ pages[other].1).count_ones();
            let similarity = ((64 - hamming) * 100 / 64) as u8;

            if hamming > max_differing_bits {
                continue;
            }

            let is_exact = pages[index].1 == pages[other].1
                && pages[index].2.as_deref() == pages[other].2.as_deref();

            if !is_exact {
                counts[index] += 1;
                counts[other] += 1;
            }

            if similarity > best_match[index] {
                best_match[index] = similarity;
            }
            if similarity > best_match[other] {
                best_match[other] = similarity;
            }
        }
    }

    let mut results = Vec::new();
    for index in 0..pages.len() {
        if counts[index] > 0 {
            results.push(SimilarityResult {
                url: pages[index].0.clone(),
                closest_similarity_percent: best_match[index],
                near_duplicate_count: counts[index],
            });
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_page(url: &str, text: &str) -> (String, u64, Option<String>) {
        let hash = crate::crawl::analyzers::compute_simhash(text);
        let content_hash = Some(format!("{:x}", md5::compute(text)));
        (url.to_string(), hash, content_hash)
    }

    #[test]
    fn empty_input_returns_empty() {
        let result = find_near_duplicates(&[], 90);
        assert!(result.is_empty());
    }

    #[test]
    fn single_page_returns_empty() {
        let pages = vec![make_page("http://a.com", "hello world")];
        let result = find_near_duplicates(&pages, 90);
        assert!(result.is_empty());
    }

    #[test]
    fn identical_pages_are_not_near_duplicates() {
        let pages = vec![
            make_page(
                "http://a.com/page1",
                "the quick brown fox jumps over the lazy dog",
            ),
            make_page(
                "http://a.com/page2",
                "the quick brown fox jumps over the lazy dog",
            ),
        ];
        let result = find_near_duplicates(&pages, 90);
        assert_eq!(
            result.len(),
            0,
            "exact duplicates should not be counted as near duplicates"
        );
    }

    #[test]
    fn near_duplicates_detected() {
        let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
        let modified = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi OTHER";
        let pages = vec![
            make_page("http://a.com/page1", base),
            make_page("http://a.com/page2", modified),
        ];
        let result = find_near_duplicates(&pages, 50);
        assert!(
            !result.is_empty(),
            "near duplicates should be detected at 50% threshold"
        );
        for entry in &result {
            assert!(entry.near_duplicate_count > 0);
            assert!(entry.closest_similarity_percent > 0);
        }
    }

    #[test]
    fn threshold_respected() {
        let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
        let different = "red green blue yellow orange purple pink brown black white gray silver gold copper bronze iron steel platinum diamond ruby emerald sapphire topaz jade amber coral";
        let pages = vec![
            make_page("http://a.com/page1", base),
            make_page("http://a.com/page2", different),
        ];
        let result = find_near_duplicates(&pages, 90);
        assert_eq!(
            result.len(),
            0,
            "very different pages should not be near duplicates at 90%"
        );
    }

    #[test]
    fn tracks_closest_match() {
        let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
        let near = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi OTHER";
        let far = "completely different text with no overlap at all xyz";
        let pages = vec![
            make_page("http://a.com/page1", base),
            make_page("http://a.com/page2", near),
            make_page("http://a.com/page3", far),
        ];
        let result = find_near_duplicates(&pages, 50);
        let page1 = result
            .iter()
            .find(|r| r.url == "http://a.com/page1")
            .expect("page1 should have results");
        assert!(
            page1.closest_similarity_percent > 80,
            "closest match should be high for near-duplicate"
        );
    }

    #[test]
    fn counts_multiple_near_duplicates() {
        let base = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi omega";
        let modified_one = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi ONE";
        let modified_two = "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron pi rho sigma tau upsilon phi chi psi TWO";
        let pages = vec![
            make_page("http://a.com/page1", base),
            make_page("http://a.com/page2", modified_one),
            make_page("http://a.com/page3", modified_two),
        ];
        let result = find_near_duplicates(&pages, 50);
        let page1 = result
            .iter()
            .find(|r| r.url == "http://a.com/page1")
            .expect("page1 should have results");
        assert_eq!(
            page1.near_duplicate_count, 2,
            "page1 should have 2 near duplicates"
        );
    }
}
