use serde_json::Map;
use serde_json::Value;
use url::Url;

use super::schema_org::normalize_type;
use crate::crawl::event::{SdIssue, SdSeverity};

struct TypeRule {
    type_name: &'static str,
    required: &'static [&'static str],
    recommended: &'static [&'static str],
}

static RULES: &[TypeRule] = &[
    TypeRule {
        type_name: "Article",
        required: &["author", "datePublished", "headline", "image"],
        recommended: &["dateModified", "mainEntityOfPage", "publisher"],
    },
    TypeRule {
        type_name: "Product",
        required: &["name"],
        recommended: &[
            "image",
            "description",
            "brand",
            "review",
            "aggregateRating",
            "offers",
        ],
    },
    TypeRule {
        type_name: "Recipe",
        required: &["name"],
        recommended: &[
            "image",
            "description",
            "recipeIngredient",
            "recipeInstructions",
            "totalTime",
            "recipeYield",
            "recipeCategory",
            "recipeCuisine",
            "nutrition",
        ],
    },
    TypeRule {
        type_name: "FAQPage",
        required: &["mainEntity"],
        recommended: &[],
    },
    TypeRule {
        type_name: "Event",
        required: &["name", "startDate", "location"],
        recommended: &["endDate", "description", "image", "offers"],
    },
    TypeRule {
        type_name: "LocalBusiness",
        required: &["name", "address"],
        recommended: &["telephone", "url", "image", "priceRange", "openingHours"],
    },
    TypeRule {
        type_name: "BreadcrumbList",
        required: &["itemListElement"],
        recommended: &[],
    },
    TypeRule {
        type_name: "Organization",
        required: &["name"],
        recommended: &["url", "logo", "contactPoint", "sameAs"],
    },
    TypeRule {
        type_name: "VideoObject",
        required: &["name", "description", "thumbnailUrl", "uploadDate"],
        recommended: &["duration", "contentUrl", "embedUrl"],
    },
    TypeRule {
        type_name: "JobPosting",
        required: &[
            "title",
            "description",
            "datePosted",
            "hiringOrganization",
            "jobLocation",
        ],
        recommended: &["employmentType", "salary", "validThrough"],
    },
    TypeRule {
        type_name: "HowTo",
        required: &["name"],
        recommended: &["step", "image", "totalTime", "estimatedCost"],
    },
    TypeRule {
        type_name: "Course",
        required: &["name", "description", "provider"],
        recommended: &["url"],
    },
    TypeRule {
        type_name: "Book",
        required: &["name"],
        recommended: &["author", "url", "isbn", "workExample"],
    },
    TypeRule {
        type_name: "Review",
        required: &["reviewRating", "author"],
        recommended: &["itemReviewed", "reviewBody"],
    },
    TypeRule {
        type_name: "SoftwareApplication",
        required: &["name"],
        recommended: &["operatingSystem", "applicationCategory", "offers"],
    },
    TypeRule {
        type_name: "Dataset",
        required: &["name", "description"],
        recommended: &["url", "creator", "distribution", "temporalCoverage"],
    },
];

pub fn validate_type(
    type_name: &str,
    properties: &Map<String, Value>,
    base: Option<&Url>,
) -> Vec<SdIssue> {
    let normalized = normalize_type(type_name).to_string();
    let mut issues = Vec::new();

    validate_url_properties(&normalized, properties, base, &mut issues);

    let Some(rule) = RULES.iter().find(|r| r.type_name == normalized) else {
        return issues;
    };

    for &prop in rule.required {
        if !properties.contains_key(prop) {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: normalized.clone(),
                code: format!("missing-required:{prop}"),
                message: format!("Required property '{prop}' is missing from {normalized}"),
            });
        }
    }

    for &prop in rule.recommended {
        if !properties.contains_key(prop) {
            issues.push(SdIssue {
                severity: SdSeverity::Warning,
                type_name: normalized.clone(),
                code: format!("missing-recommended:{prop}"),
                message: format!("Recommended property '{prop}' is missing from {normalized}"),
            });
        }
    }

    if normalized == "Product" {
        validate_product_offers(properties, &mut issues);
    }
    if normalized == "BreadcrumbList" {
        validate_breadcrumb_list(properties, base, &mut issues);
    }

    issues
}

/// Properties Google reads as a URL. A value that is not one costs the rich
/// result the same way a missing property does, and the report ("Invalid URL
/// in field ...") is the only place it surfaces, so the value is checked and
/// not just its presence.
static URL_PROPERTIES: &[&str] = &[
    "@id",
    "url",
    "image",
    "logo",
    "sameAs",
    "thumbnailUrl",
    "contentUrl",
    "embedUrl",
    "mainEntityOfPage",
    "acquireLicensePage",
];

/// The properties of a nested object that carry its URL, in the order Google
/// reads them: `{"@type": "ImageObject", "url": "..."}` is as valid an `image`
/// as a bare string.
static NESTED_URL_KEYS: &[&str] = &["@id", "url", "contentUrl"];

enum UrlVerdict {
    Absolute,
    /// Resolvable against the page, which Google does for most fields but not
    /// reliably, and never for an identifier that has to match across pages.
    Relative,
    Invalid(&'static str),
}

fn classify_url(raw: &str, base: Option<&Url>) -> UrlVerdict {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return UrlVerdict::Invalid("the value is empty");
    }
    if trimmed.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return UrlVerdict::Invalid("the value contains whitespace");
    }
    // An unrendered template leaves the placeholder in the markup, which is
    // the most common way a URL field ends up invalid on a generated page.
    if trimmed.contains(['{', '}', '<', '>'])
        || matches!(trimmed, "null" | "undefined" | "NaN" | "#" | "-")
    {
        return UrlVerdict::Invalid("the value is a placeholder, not a URL");
    }

    match Url::parse(trimmed) {
        Ok(url) => {
            if !matches!(url.scheme(), "http" | "https") {
                return UrlVerdict::Invalid("the scheme is not http or https");
            }
            if url.host_str().is_none_or(|host| host.is_empty()) {
                return UrlVerdict::Invalid("the URL has no host");
            }
            UrlVerdict::Absolute
        }
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            // `www.example.com/page` reads as a relative path, resolves
            // against the page and silently points at the wrong URL, so it is
            // a mistake rather than a legal relative reference.
            if trimmed.starts_with("www.") {
                return UrlVerdict::Invalid("the URL is missing its scheme");
            }
            match base {
                Some(base) if base.join(trimmed).is_ok() => UrlVerdict::Relative,
                Some(_) => UrlVerdict::Invalid("the value is not a URL"),
                None => UrlVerdict::Relative,
            }
        }
        Err(_) => UrlVerdict::Invalid("the value is not a URL"),
    }
}

/// Whether the property is an `@id`, either directly or as the key reached
/// through a nested object such as `item.@id`.
fn is_identifier_property(property: &str) -> bool {
    property == "@id" || property.ends_with(".@id")
}

fn check_url_value(
    type_name: &str,
    property: &str,
    value: &Value,
    base: Option<&Url>,
    issues: &mut Vec<SdIssue>,
) {
    match value {
        // A blank node identifier is how JSON-LD names an entity that has no
        // URL of its own, and it is legal wherever an `@id` is.
        Value::String(raw) if is_identifier_property(property) && raw.trim().starts_with("_:") => {}
        Value::String(raw) => match classify_url(raw, base) {
            UrlVerdict::Absolute => {}
            UrlVerdict::Relative => issues.push(SdIssue {
                severity: SdSeverity::Warning,
                type_name: type_name.to_string(),
                code: format!("relative-url:{property}"),
                message: format!(
                    "Property '{property}' of {type_name} is the relative URL '{raw}'; Google expects an absolute URL here"
                ),
            }),
            UrlVerdict::Invalid(reason) => issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: type_name.to_string(),
                code: format!("invalid-url:{property}"),
                message: format!(
                    "Invalid URL '{raw}' in property '{property}' of {type_name}: {reason}"
                ),
            }),
        },
        Value::Array(items) => {
            for item in items {
                check_url_value(type_name, property, item, base, issues);
            }
        }
        Value::Object(map) => {
            // A nested entity is validated on its own when the crawler walks
            // into it; here only the URL it stands for is of interest.
            if let Some((key, nested)) = NESTED_URL_KEYS
                .iter()
                .find_map(|key| map.get(*key).map(|value| (*key, value)))
            {
                let nested_property = if property == key {
                    property.to_string()
                } else {
                    format!("{property}.{key}")
                };
                check_url_value(type_name, &nested_property, nested, base, issues);
            }
        }
        Value::Null => {}
        other => issues.push(SdIssue {
            severity: SdSeverity::Error,
            type_name: type_name.to_string(),
            code: format!("invalid-url:{property}"),
            message: format!(
                "Invalid URL in property '{property}' of {type_name}: {other} is not a URL"
            ),
        }),
    }
}

fn validate_url_properties(
    type_name: &str,
    properties: &Map<String, Value>,
    base: Option<&Url>,
    issues: &mut Vec<SdIssue>,
) {
    for &property in URL_PROPERTIES {
        if let Some(value) = properties.get(property) {
            check_url_value(type_name, property, value, base, issues);
        }
    }
}

/// Google's breadcrumb spec: an ordered `ListItem` per level, each with a
/// `position` and a `name`, and a URL for every level but the last. The URL
/// may be the item itself or the `@id` of the entity it points at, and an
/// invalid one there is reported as "invalid URL in field id".
fn validate_breadcrumb_list(
    properties: &Map<String, Value>,
    base: Option<&Url>,
    issues: &mut Vec<SdIssue>,
) {
    let entries: Vec<&Value> = match properties.get("itemListElement") {
        Some(Value::Array(items)) => items.iter().collect(),
        Some(value @ Value::Object(_)) => vec![value],
        Some(_) => {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "BreadcrumbList".into(),
                code: "invalid-value:itemListElement".into(),
                message: "Property 'itemListElement' of BreadcrumbList must be a list of ListItem objects"
                    .into(),
            });
            return;
        }
        None => return,
    };

    if entries.is_empty() {
        issues.push(SdIssue {
            severity: SdSeverity::Error,
            type_name: "BreadcrumbList".into(),
            code: "empty-value:itemListElement".into(),
            message: "BreadcrumbList has an empty 'itemListElement' and produces no breadcrumb"
                .into(),
        });
        return;
    }

    let last_index = entries.len() - 1;
    let level_count = entries.len() as u64;
    let mut positions = Vec::with_capacity(entries.len());
    for (index, entry) in entries.iter().enumerate() {
        let level = index + 1;
        let Some(map) = entry.as_object() else {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "ListItem".into(),
                code: "invalid-value:itemListElement".into(),
                message: format!("Breadcrumb level {level} is not a ListItem object"),
            });
            continue;
        };

        if let Some(type_name) = map.get("@type").and_then(Value::as_str)
            && normalize_type(type_name) != "ListItem"
        {
            issues.push(SdIssue {
                severity: SdSeverity::Warning,
                type_name: "ListItem".into(),
                code: "invalid-value:@type".into(),
                message: format!(
                    "Breadcrumb level {level} is a '{type_name}'; Google reads only ListItem entries"
                ),
            });
        }

        let position = map.get("position").and_then(position_number);
        positions.push(position);
        match map.get("position") {
            None => issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "ListItem".into(),
                code: "missing-required:position".into(),
                message: format!("Breadcrumb level {level} is missing its 'position'"),
            }),
            Some(value) if position.is_none() => issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "ListItem".into(),
                code: "invalid-value:position".into(),
                message: format!("Breadcrumb level {level} has the non-numeric position '{value}'"),
            }),
            Some(_) => {}
        }

        let item = map.get("item");
        let item_object = item.and_then(Value::as_object);
        let has_name =
            has_text(map.get("name")) || item_object.is_some_and(|item| has_text(item.get("name")));
        if !has_name {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "ListItem".into(),
                code: "missing-required:name".into(),
                message: format!("Breadcrumb level {level} is missing its 'name'"),
            });
        }

        match item {
            None => {
                // The deepest crumb is the current page and needs no URL. The
                // list may be written in any order, so where a position says
                // which level this is, that is what decides; the position in
                // the list only stands in when there is no usable one.
                let is_deepest = match position {
                    Some(position) => position == level_count,
                    None => index == last_index,
                };
                if !is_deepest {
                    issues.push(SdIssue {
                        severity: SdSeverity::Error,
                        type_name: "ListItem".into(),
                        code: "missing-required:item".into(),
                        message: format!(
                            "Breadcrumb level {level} is missing its 'item' URL; only the last level may omit it"
                        ),
                    });
                }
            }
            Some(item @ (Value::String(_) | Value::Object(_))) => {
                if let Some(item_object) = item_object
                    && !NESTED_URL_KEYS.iter().any(|key| item_object.contains_key(*key))
                {
                    issues.push(SdIssue {
                        severity: SdSeverity::Error,
                        type_name: "ListItem".into(),
                        code: "missing-required:item.@id".into(),
                        message: format!(
                            "Breadcrumb level {level} has an 'item' object without an '@id' or 'url'"
                        ),
                    });
                }
                // An item that names its own type is an entity the walker
                // descends into and validates in its own right, so checking
                // its URL here too would count one mistake twice.
                let is_typed_entity = item_object.is_some_and(|item| item.contains_key("@type"));
                if !is_typed_entity {
                    check_url_value("ListItem", "item", item, base, issues);
                }
            }
            Some(other) => issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "ListItem".into(),
                code: "invalid-value:item".into(),
                message: format!(
                    "Breadcrumb level {level} has an 'item' that is neither a URL nor an entity: {other}"
                ),
            }),
        }
    }

    // The trail is ordered by `position`, not by the order the levels are
    // written in, so the run has to be 1..n once sorted and no more than that.
    let mut numbered: Vec<u64> = positions.iter().filter_map(|position| *position).collect();
    numbered.sort_unstable();
    if numbered.len() == entries.len()
        && !numbered
            .iter()
            .enumerate()
            .all(|(index, position)| *position == index as u64 + 1)
    {
        issues.push(SdIssue {
            severity: SdSeverity::Warning,
            type_name: "BreadcrumbList".into(),
            code: "invalid-value:position-order".into(),
            message: "Breadcrumb positions are not the consecutive run 1, 2, 3 ... that Google orders the trail by"
                .into(),
        });
    }
}

/// Shops emit `"position": 1` and `"position": "1"` in equal measure, and
/// Google accepts both.
fn position_number(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

fn has_text(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Number(_)) => true,
        _ => false,
    }
}

/// Google's product snippet requires one of `offers`, `review` or
/// `aggregateRating`, and a merchant listing needs an offer with a price and
/// currency. A product with none of them is valid schema.org and invisible as
/// a rich result, which is the thing a shop marks it up for.
fn validate_product_offers(properties: &Map<String, Value>, issues: &mut Vec<SdIssue>) {
    let has_any = ["offers", "review", "aggregateRating"]
        .iter()
        .any(|key| properties.get(*key).is_some_and(|v| !v.is_null()));
    if !has_any {
        issues.push(SdIssue {
            severity: SdSeverity::Error,
            type_name: "Product".into(),
            code: "missing-required:offers|review|aggregateRating".into(),
            message: "Product needs one of 'offers', 'review' or 'aggregateRating' to be \
                      eligible for rich results"
                .into(),
        });
    }

    let Some(offers) = properties.get("offers") else {
        return;
    };
    let offer_objects: Vec<&Map<String, Value>> = match offers {
        Value::Object(map) => vec![map],
        Value::Array(items) => items.iter().filter_map(Value::as_object).collect(),
        _ => Vec::new(),
    };
    for offer in offer_objects {
        let is_aggregate = offer
            .get("@type")
            .and_then(Value::as_str)
            .is_some_and(|t| normalize_type(t) == "AggregateOffer");
        let has_price = if is_aggregate {
            offer.contains_key("lowPrice") || offer.contains_key("price")
        } else {
            offer.contains_key("price") || offer.contains_key("priceSpecification")
        };
        if !has_price {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "Offer".into(),
                code: "missing-required:price".into(),
                message: "Required property 'price' is missing from Offer".into(),
            });
        }
        if !offer.contains_key("priceCurrency") && !offer.contains_key("priceSpecification") {
            issues.push(SdIssue {
                severity: SdSeverity::Error,
                type_name: "Offer".into(),
                code: "missing-required:priceCurrency".into(),
                message: "Required property 'priceCurrency' is missing from Offer".into(),
            });
        }
        if !offer.contains_key("availability") {
            issues.push(SdIssue {
                severity: SdSeverity::Warning,
                type_name: "Offer".into(),
                code: "missing-recommended:availability".into(),
                message: "Recommended property 'availability' is missing from Offer".into(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> Url {
        Url::parse("https://shop.test/catalogue/widget").expect("base url")
    }

    fn codes_for(type_name: &str, value: &Value) -> Vec<String> {
        let map = value.as_object().cloned().unwrap_or_default();
        validate_type(type_name, &map, Some(&base()))
            .into_iter()
            .map(|issue| issue.code)
            .collect()
    }

    fn issues_for(value: &Value) -> Vec<String> {
        codes_for("Product", value)
    }

    #[test]
    fn product_without_offers_or_reviews_is_an_error() {
        let codes = issues_for(&json!({"@type": "Product", "name": "Widget"}));
        assert!(codes.contains(&"missing-required:offers|review|aggregateRating".to_string()));
    }

    #[test]
    fn offer_without_price_or_currency_is_an_error() {
        let codes = issues_for(&json!({
            "@type": "Product", "name": "Widget",
            "offers": {"@type": "Offer"}
        }));
        assert!(codes.contains(&"missing-required:price".to_string()));
        assert!(codes.contains(&"missing-required:priceCurrency".to_string()));
        assert!(codes.contains(&"missing-recommended:availability".to_string()));
    }

    #[test]
    fn complete_offers_and_aggregate_offers_pass() {
        let codes = issues_for(&json!({
            "@type": "Product", "name": "Widget",
            "offers": [
                {"@type": "Offer", "price": "9.99", "priceCurrency": "SEK",
                 "availability": "https://schema.org/InStock"},
                {"@type": "AggregateOffer", "lowPrice": "5", "highPrice": "9",
                 "priceCurrency": "SEK", "availability": "https://schema.org/InStock"}
            ]
        }));
        assert!(
            !codes
                .iter()
                .any(|code| code.starts_with("missing-required"))
        );
    }

    #[test]
    fn breadcrumb_item_with_invalid_id_is_an_error() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home",
                     "item": {"@id": "{{ site.url }}", "name": "Home"}},
                    {"@type": "ListItem", "position": 2, "name": "Widget"}
                ]
            }),
        );
        assert!(
            codes.contains(&"invalid-url:item.@id".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn breadcrumb_item_missing_scheme_is_an_error() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home",
                     "item": "www.shop.test/"},
                    {"@type": "ListItem", "position": 2, "name": "Widget",
                     "item": "https://shop.test/catalogue/widget"}
                ]
            }),
        );
        assert!(codes.contains(&"invalid-url:item".to_string()), "{codes:?}");
    }

    #[test]
    fn breadcrumb_relative_item_is_a_warning_not_an_error() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home", "item": "/"},
                    {"@type": "ListItem", "position": 2, "name": "Widget"}
                ]
            }),
        );
        assert_eq!(codes, vec!["relative-url:item".to_string()]);
    }

    #[test]
    fn breadcrumb_missing_name_position_and_intermediate_url_are_errors() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home",
                     "item": "https://shop.test/"},
                    {"@type": "ListItem", "position": 2},
                    {"@type": "ListItem", "position": 3, "name": "Widget"}
                ]
            }),
        );
        assert!(
            codes.contains(&"missing-required:name".to_string()),
            "{codes:?}"
        );
        assert!(
            codes.contains(&"missing-required:item".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn breadcrumb_positions_out_of_order_warn() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 2, "name": "Home",
                     "item": "https://shop.test/"},
                    {"@type": "ListItem", "position": 3, "name": "Widget",
                     "item": "https://shop.test/catalogue/widget"}
                ]
            }),
        );
        assert_eq!(codes, vec!["invalid-value:position-order".to_string()]);
    }

    #[test]
    fn valid_breadcrumb_passes() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "@id": "https://shop.test/catalogue/widget#breadcrumb",
                "itemListElement": [
                    {"@type": "ListItem", "position": "1", "name": "Home",
                     "item": "https://shop.test/"},
                    {"@type": "ListItem", "position": "2", "name": "Catalogue",
                     "item": {"@id": "https://shop.test/catalogue", "name": "Catalogue"}},
                    {"@type": "ListItem", "position": "3", "name": "Widget"}
                ]
            }),
        );
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn url_properties_are_validated_on_every_type() {
        let codes = codes_for(
            "Organization",
            &json!({
                "@type": "Organization", "name": "Shop",
                "url": "https://shop.test/",
                "logo": {"@type": "ImageObject", "url": "javascript:void(0)"},
                "sameAs": ["https://shop.test/about", "not a url"],
                "contactPoint": {"@type": "ContactPoint", "telephone": "+46"}
            }),
        );
        assert!(
            codes.contains(&"invalid-url:logo.url".to_string()),
            "{codes:?}"
        );
        assert_eq!(
            codes
                .iter()
                .filter(|code| code.as_str() == "invalid-url:sameAs")
                .count(),
            1,
            "{codes:?}"
        );
    }

    #[test]
    fn absolute_urls_and_data_free_types_are_quiet() {
        let codes = codes_for(
            "Product",
            &json!({
                "@type": "Product", "name": "Widget",
                "@id": "https://shop.test/catalogue/widget#product",
                "image": ["https://shop.test/img/widget.jpg"],
                "brand": {"@type": "Brand", "name": "Acme"},
                "review": {"@type": "Review"},
                "offers": {"@type": "Offer", "price": "9.99", "priceCurrency": "SEK",
                           "availability": "https://schema.org/InStock",
                           "url": "https://shop.test/catalogue/widget"}
            }),
        );
        assert!(
            !codes.iter().any(|code| code.starts_with("invalid-url")),
            "{codes:?}"
        );
        assert!(
            !codes.iter().any(|code| code.starts_with("relative-url")),
            "{codes:?}"
        );
    }

    #[test]
    fn breadcrumb_entry_written_last_still_needs_an_item_unless_its_position_is_last() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 2, "name": "Widget",
                     "item": "https://shop.test/catalogue/widget"},
                    {"@type": "ListItem", "position": 1, "name": "Home"}
                ]
            }),
        );
        assert!(
            codes.contains(&"missing-required:item".to_string()),
            "{codes:?}"
        );
    }

    #[test]
    fn breadcrumb_typed_item_is_left_to_the_walker_and_not_checked_twice() {
        let codes = codes_for(
            "BreadcrumbList",
            &json!({
                "@type": "BreadcrumbList",
                "itemListElement": [
                    {"@type": "ListItem", "position": 1, "name": "Home",
                     "item": {"@type": "WebPage", "@id": "{{ site.url }}"}},
                    {"@type": "ListItem", "position": 2, "name": "Widget"}
                ]
            }),
        );
        assert!(codes.is_empty(), "{codes:?}");
    }

    #[test]
    fn a_pipe_in_a_query_string_is_an_absolute_url() {
        assert!(matches!(
            classify_url("https://maps.google.com/?q=a|b", Some(&base())),
            UrlVerdict::Absolute
        ));
        assert!(matches!(
            classify_url("https://shop.test/?bbox=1^2", Some(&base())),
            UrlVerdict::Absolute
        ));
    }

    #[test]
    fn a_blank_node_identifier_is_not_a_url_issue() {
        let codes = codes_for(
            "Organization",
            &json!({"@type": "Organization", "name": "Shop", "@id": "_:b0"}),
        );
        assert!(
            !codes
                .iter()
                .any(|code| code.starts_with("invalid-url") || code.starts_with("relative-url")),
            "{codes:?}"
        );

        let codes = codes_for(
            "Organization",
            &json!({"@type": "Organization", "name": "Shop", "@id": "{{ site.url }}"}),
        );
        assert!(codes.contains(&"invalid-url:@id".to_string()), "{codes:?}");
    }
}
