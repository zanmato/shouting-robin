//! Status-checking the resources a crawl discovers.
//!
//! The page crawl records HTML documents. Everything those documents point at,
//! images, stylesheets, scripts and links to other sites, is known by URL but
//! never requested, so a broken image or a dead external link is invisible and
//! an image's size is unknown. This pass requests each discovered URL once,
//! after the page crawl has finished, and records what came back.
//!
//! It is a HEAD first, falling back to GET when the server will not answer a
//! HEAD or answers without a length. Many servers reject HEAD outright, and a
//! HEAD that omits `Content-Length` tells us nothing about size, which is the
//! whole point for images.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use tokio::task::JoinSet;

/// How many resource requests are in flight at once. The page crawl is over by
/// the time this runs, so this bounds our own footprint on the target site
/// rather than competing with it.
const RESOURCE_CONCURRENCY: usize = 8;

/// The most resources a single crawl will status-check. A large site links out
/// far more often than it has pages, and this pass is one request per URL, so
/// it is capped rather than left to grow with the link graph.
pub const MAX_RESOURCE_CHECKS: usize = 10_000;

/// Bytes read from a GET fallback before we stop counting. Only reached when a
/// server gives no `Content-Length`, and only to keep one pathological asset
/// from being downloaded in full.
const MAX_BODY_BYTES: u64 = 8 * 1024 * 1024;

/// Why a URL is being checked, which is also how the row is labelled when the
/// response says nothing useful about its type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Image,
    Stylesheet,
    Script,
    /// A link to another origin, from an `<a href>`.
    ExternalLink,
}

impl ResourceKind {
    /// The `initiator` recorded on the row, matching the vocabulary Chrome's
    /// Resource Timing entries use so both discovery paths agree.
    pub fn initiator(self) -> &'static str {
        match self {
            Self::Image => "img",
            Self::Stylesheet => "css",
            Self::Script => "script",
            Self::ExternalLink => "link",
        }
    }

    /// The content type to assume when the response gives none.
    fn fallback_content_type(self) -> Option<&'static str> {
        match self {
            Self::Image => None,
            Self::Stylesheet => Some("text/css"),
            Self::Script => Some("text/javascript"),
            Self::ExternalLink => None,
        }
    }
}

/// What one resource turned out to be.
#[derive(Clone, Debug)]
pub struct ResourceCheck {
    pub url: String,
    pub kind: ResourceKind,
    pub status: Option<u16>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    /// The response headers, as sent. Kept for the same reason a page's are:
    /// the security rules read them, and a row with none is indistinguishable
    /// from a URL whose server sends none — which is how every asset came to
    /// report a missing HSTS header it was in fact being served.
    pub headers: Vec<(String, String)>,
    pub response_time: Duration,
    /// Where the resource redirected to, when it did.
    pub redirect_url: Option<String>,
    /// Set when the request could not be made at all (DNS, TLS, timeout).
    pub error: Option<String>,
}

/// True when a URL is worth requesting: an absolute http(s) URL. Inline `data:`
/// payloads, `mailto:`, `javascript:` and the like are not fetchable.
pub fn is_checkable(url: &str) -> bool {
    let lowered = url.trim().to_ascii_lowercase();
    lowered.starts_with("http://") || lowered.starts_with("https://")
}

/// Collects the URLs to check, in discovery order, keeping the first kind seen
/// for each and dropping anything already recorded as a page of its own.
///
/// Discovery order matters: it is the order the rows appear in, and a stable
/// order makes two crawls of the same site comparable.
pub fn plan_checks(
    discovered: &[(String, ResourceKind)],
    already_recorded: &std::collections::HashSet<String>,
) -> Vec<(String, ResourceKind)> {
    let mut seen: HashMap<&str, ()> = HashMap::new();
    let mut planned = Vec::new();
    for (url, kind) in discovered {
        if !is_checkable(url) || already_recorded.contains(url) {
            continue;
        }
        if seen.insert(url.as_str(), ()).is_some() {
            continue;
        }
        planned.push((url.clone(), *kind));
        if planned.len() >= MAX_RESOURCE_CHECKS {
            tracing::warn!(
                cap = MAX_RESOURCE_CHECKS,
                "resource check cap reached; remaining resources are not status-checked"
            );
            break;
        }
    }
    planned
}

/// Runs the checks with bounded concurrency, handing each result to `on_result`
/// as it lands so rows reach the UI during the pass rather than after it.
pub async fn check_all<F, Fut>(
    client: &reqwest::Client,
    planned: Vec<(String, ResourceKind)>,
    cancel: &Arc<AtomicBool>,
    mut on_result: F,
) where
    F: FnMut(ResourceCheck) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let mut queue = planned.into_iter();
    let mut in_flight: JoinSet<ResourceCheck> = JoinSet::new();

    loop {
        while in_flight.len() < RESOURCE_CONCURRENCY && !cancel.load(Ordering::Relaxed) {
            let Some((url, kind)) = queue.next() else {
                break;
            };
            let client = client.clone();
            in_flight.spawn(async move { check_one(&client, url, kind).await });
        }
        let Some(joined) = in_flight.join_next().await else {
            break;
        };
        match joined {
            Ok(check) => on_result(check).await,
            Err(e) => tracing::warn!(error=%e, "resource check task failed"),
        }
        if cancel.load(Ordering::Relaxed) {
            in_flight.abort_all();
            break;
        }
    }
}

async fn check_one(client: &reqwest::Client, url: String, kind: ResourceKind) -> ResourceCheck {
    let started = Instant::now();
    let mut check = ResourceCheck {
        url: url.clone(),
        kind,
        status: None,
        content_type: None,
        headers: Vec::new(),
        size_bytes: 0,
        response_time: Duration::ZERO,
        redirect_url: None,
        error: None,
    };

    match client.head(&url).send().await {
        Ok(response) => {
            apply_response(&mut check, &response);
            // A HEAD that answered but gave no length leaves the size unknown,
            // which is the one thing an image row is here for.
            let needs_body = check.size_bytes == 0 && response.status().is_success();
            let head_refused = matches!(response.status().as_u16(), 405 | 501);
            if !needs_body && !head_refused {
                check.response_time = started.elapsed();
                return check;
            }
        }
        Err(e) => {
            check.error = Some(e.to_string());
        }
    }

    match client.get(&url).send().await {
        Ok(mut response) => {
            check.error = None;
            apply_response(&mut check, &response);
            if check.size_bytes == 0 {
                let mut counted: u64 = 0;
                loop {
                    match response.chunk().await {
                        Ok(Some(chunk)) => {
                            counted += chunk.len() as u64;
                            if counted >= MAX_BODY_BYTES {
                                break;
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            tracing::debug!(url = %url, error=%e, "reading resource body failed");
                            break;
                        }
                    }
                }
                check.size_bytes = counted;
            }
        }
        Err(e) => {
            check.error = Some(e.to_string());
        }
    }

    check.response_time = started.elapsed();
    check
}

fn apply_response(check: &mut ResourceCheck, response: &reqwest::Response) {
    check.status = Some(response.status().as_u16());
    check.headers = response
        .headers()
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.to_string(), value.to_string()))
        })
        .collect();
    if let Some(value) = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    {
        check.content_type = Some(
            value
                .split(';')
                .next()
                .unwrap_or(value)
                .trim()
                .trim_matches('"')
                .to_string(),
        );
    }
    if check.content_type.is_none() {
        check.content_type = check
            .kind
            .fallback_content_type()
            .map(|content_type| content_type.to_string());
    }
    if let Some(length) = response.content_length() {
        check.size_bytes = length;
    }
    // reqwest follows redirects by default, so a final URL different from the
    // one we asked for is the redirect this resource served.
    let final_url = response.url().as_str();
    if final_url != check.url {
        check.redirect_url = Some(final_url.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn only_absolute_http_urls_are_checkable() {
        assert!(is_checkable("https://a.test/logo.png"));
        assert!(is_checkable("HTTP://a.test/logo.png"));
        assert!(!is_checkable("data:image/png;base64,#abc (10 B)"));
        assert!(!is_checkable("mailto:me@a.test"));
        assert!(!is_checkable("/relative.png"));
        assert!(!is_checkable(""));
    }

    #[test]
    fn planning_dedups_and_skips_urls_already_recorded() {
        let discovered = vec![
            ("https://a.test/logo.png".into(), ResourceKind::Image),
            // The same asset on a second page.
            ("https://a.test/logo.png".into(), ResourceKind::Image),
            ("https://a.test/app.css".into(), ResourceKind::Stylesheet),
            // Already crawled as a page of its own.
            ("https://a.test/about".into(), ResourceKind::ExternalLink),
            ("data:image/png;base64,x".into(), ResourceKind::Image),
        ];
        let recorded: HashSet<String> = ["https://a.test/about".to_string()].into_iter().collect();

        let planned = plan_checks(&discovered, &recorded);
        assert_eq!(
            planned,
            vec![
                ("https://a.test/logo.png".to_string(), ResourceKind::Image),
                (
                    "https://a.test/app.css".to_string(),
                    ResourceKind::Stylesheet
                ),
            ]
        );
    }

    #[test]
    fn planning_stops_at_the_cap() {
        let discovered: Vec<(String, ResourceKind)> = (0..MAX_RESOURCE_CHECKS + 50)
            .map(|i| (format!("https://a.test/{i}.png"), ResourceKind::Image))
            .collect();
        let planned = plan_checks(&discovered, &HashSet::new());
        assert_eq!(planned.len(), MAX_RESOURCE_CHECKS);
    }
}
