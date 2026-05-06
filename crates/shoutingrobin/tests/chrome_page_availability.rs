/// Standalone test that mirrors the spider `chrome_screenshot` example pattern
/// to verify that `Page::get_chrome_page()` returns a valid handle for *every*
/// page yielded by the subscription, not just the first one.
use std::net::TcpListener;
use std::process::{Child, Command};

struct ChildGuard(Child);

impl ChildGuard {
    fn kill(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill();
    }
}
use std::sync::Arc;
use std::time::Duration;

use spider::configuration::WaitForIdleNetwork;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::website::Website;

#[allow(clippy::zombie_processes)]
fn spawn_http_server() -> (ChildGuard, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let test_site_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-site");

    let child = Command::new("python3")
        .arg("-m")
        .arg("http.server")
        .arg(port.to_string())
        .arg("--directory")
        .arg(&test_site_dir)
        .spawn()
        .expect("spawn python3 http.server");

    let guard = ChildGuard(child);
    for _ in 0..20 {
        if std::net::TcpStream::connect(format!("127.0.0.1:{port}")).is_ok() {
            return (guard, port);
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    panic!("http.server did not start within 2s");
}

fn chrome_available() -> bool {
    [
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ]
    .iter()
    .any(|bin| {
        std::process::Command::new(bin)
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    })
}

fn crawl_with_chrome_pages(root_url: &str) -> Vec<(String, bool, bool)> {
    let root_url = root_url.to_string();

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .unwrap()
        .block_on(crawl_body(&root_url))
}

async fn crawl_body(root_url: &str) -> Vec<(String, bool, bool)> {
    let mut website: Website = Website::new(root_url);
    website.configuration.request_timeout = Some(Duration::from_secs(60));
    // Probe: chrome_intercept disabled to isolate slowdown source.
    let _ = RequestInterceptConfiguration::new(true);
    website
        .with_stealth(true)
        .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(Duration::from_secs(5)))));

    if website.build().is_err() {
        panic!("failed to build Website for {root_url}");
    }

    let mut rx = website.subscribe(1024);
    let mut subscribe_guard = website
        .subscribe_guard()
        .expect("subscribe_guard requires spider sync feature");

    let results: Arc<tokio::sync::Mutex<Vec<(String, bool, bool)>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let pump = tokio::spawn(async move {
        loop {
            let mut page = match rx.recv().await {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("subscribe recv ended: {e}");
                    break;
                }
            };

            let url = page.get_url().to_string();
            let has_chrome_page = page.get_chrome_page().is_some();

            let mut perf_ok = false;
            if let Some(chrome_page) = page.get_chrome_page() {
                let js = r#"
                    (async function() {
                        await new Promise(function(resolve) { setTimeout(resolve, 0); });
                        var result = { ttfb: null };
                        try {
                            var nav = performance.getEntriesByType('navigation')[0];
                            if (nav) result.ttfb = Math.round(nav.responseStart - nav.requestStart);
                        } catch(e) {}
                        return result;
                    })()
                "#;
                let params =
                    spider::chromiumoxide::cdp::js_protocol::runtime::EvaluateParams::builder()
                        .expression(js)
                        .await_promise(true)
                        .return_by_value(true)
                        .build();

                if let Ok(params) = params {
                    match chrome_page.evaluate(params).await {
                        Ok(eval_result) => {
                            if let Some(value) = eval_result.value()
                                && value.as_object().is_some()
                            {
                                perf_ok = true;
                            }
                        }
                        Err(e) => {
                            eprintln!("  evaluate failed for {url}: {e}");
                        }
                    }
                }
            }

            eprintln!("  page: {url}  chrome_page={has_chrome_page}  perf_ok={perf_ok}");

            results_clone
                .lock()
                .await
                .push((url, has_chrome_page, perf_ok));
            page.close_page().await;
            subscribe_guard.inc();
        }
    });

    let start = tokio::time::Instant::now();
    website.crawl().await;
    let elapsed = start.elapsed();

    // Give the pump time to drain remaining pages, then close the broadcast
    // so the pump exits cleanly rather than being aborted mid-evaluate.
    tokio::time::sleep(Duration::from_secs(2)).await;
    website.unsubscribe();
    let _ = pump.await;

    let links = website.get_all_links_visited().await;
    eprintln!(
        "crawl finished in {elapsed:?}, spider reports {} visited URLs",
        links.len()
    );

    let guard = results.lock().await;
    guard.clone()
}

#[test]
fn test_chrome_page_available_on_every_page() {
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "spider=warn".into()),
        )
        .try_init();

    // Clean up stale chromiumoxide lock if present (prevents fallback to HTTP)
    let lock_path = std::path::Path::new("/tmp/chromiumoxide-runner/SingletonSocket");
    if lock_path.exists() {
        eprintln!("removing stale chromiumoxide lock: {}", lock_path.display());
        let _ = std::fs::remove_file(lock_path);
    }
    let cookie_path = std::path::Path::new("/tmp/chromiumoxide-runner/SingletonCookie");
    if cookie_path.exists() {
        let _ = std::fs::remove_file(cookie_path);
    }
    let lock_file = std::path::Path::new("/tmp/chromiumoxide-runner/SingletonLock");
    if lock_file.exists() {
        let _ = std::fs::remove_file(lock_file);
    }

    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    eprintln!("crawling {root_url} with chrome_store_page + subscribe_guard...");
    let results = crawl_with_chrome_pages(&root_url);

    let _ = server.kill();

    assert!(
        !results.is_empty(),
        "crawl should discover at least one page"
    );

    eprintln!("\n--- results ---");
    for (url, has_chrome, perf_ok) in &results {
        let chrome_icon = if *has_chrome { "OK" } else { "MISSING" };
        let perf_icon = if *perf_ok { "OK" } else { "FAIL" };
        eprintln!("  {chrome_icon} chrome | {perf_icon} perf | {url}");
    }

    let missing_chrome: Vec<_> = results
        .iter()
        .filter(|(_, has_chrome, _)| !has_chrome)
        .collect();

    assert_eq!(
        missing_chrome.len(),
        0,
        "expected chrome page handle on every page, but {} of {} were missing: {:?}",
        missing_chrome.len(),
        results.len(),
        missing_chrome
            .iter()
            .map(|(url, _, _)| url.as_str())
            .collect::<Vec<_>>()
    );

    let perf_failed: Vec<_> = results.iter().filter(|(_, _, perf_ok)| !perf_ok).collect();

    assert_eq!(
        perf_failed.len(),
        0,
        "expected perf metrics on every chrome page, but {} of {} failed: {:?}",
        perf_failed.len(),
        results.len(),
        perf_failed
            .iter()
            .map(|(url, _, _)| url.as_str())
            .collect::<Vec<_>>()
    );
}
