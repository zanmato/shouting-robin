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

    issues
}
