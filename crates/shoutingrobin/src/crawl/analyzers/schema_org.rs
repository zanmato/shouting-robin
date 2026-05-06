use std::collections::HashSet;
use std::sync::LazyLock;

static VALID_TYPES: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("../../../resources/schema_types.txt")
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect()
});

pub fn is_valid_type(type_name: &str) -> bool {
    let normalized = normalize_type(type_name);
    VALID_TYPES.contains(normalized)
}

pub fn normalize_type(type_name: &str) -> &str {
    type_name
        .strip_prefix("https://schema.org/")
        .or_else(|| type_name.strip_prefix("http://schema.org/"))
        .unwrap_or(type_name)
}
