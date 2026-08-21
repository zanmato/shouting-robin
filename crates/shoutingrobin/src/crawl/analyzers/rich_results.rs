use serde_json::Map;
use serde_json::Value;

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

pub fn validate_type(type_name: &str, properties: &Map<String, Value>) -> Vec<SdIssue> {
    let normalized = normalize_type(type_name).to_string();
    let mut issues = Vec::new();

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

    issues
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

    fn issues_for(value: &Value) -> Vec<String> {
        let map = value.as_object().cloned().unwrap_or_default();
        validate_type("Product", &map)
            .into_iter()
            .map(|issue| issue.code)
            .collect()
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
}
