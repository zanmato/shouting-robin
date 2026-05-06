use std::net::TcpListener;
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Wraps `Child` so the process is killed even if the test panics before the
/// explicit `kill()`. Without this, leaked python http.server processes
/// accumulate every time an assertion fails.
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

fn crawl_test_site(root_url: &str) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    crawl_test_site_with_mode(
        root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Http,
        Duration::from_secs(30),
    )
}

fn crawl_test_site_with_mode(
    root_url: &str,
    render_mode: shoutingrobin::crawl::render_mode::RenderMode,
    timeout: Duration,
) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    let cancel = Arc::new(AtomicBool::new(false));

    let rt = tokio::runtime::Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let pool = sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();
        shoutingrobin::storage::run_migrations(&pool).await.unwrap();
        pool
    });

    let (tx, rx) = shoutingrobin::crawl::engine::channel();
    let (_, fut) = {
        let mut engine = shoutingrobin::crawl::engine::CrawlEngine::new();
        engine.start(
            root_url.to_string(),
            tx,
            pool,
            render_mode,
            shoutingrobin::crawl::CrawlConfig {
                max_pages: 0,
                max_concurrent: 10,
                delay_ms: 0,
                timeout_seconds: 30,
                respect_robots_txt: true,
                near_duplicate_threshold: 90,
            },
        )
    };

    let _ = cancel.clone();
    rt.spawn(async move {
        fut.await;
    });

    let mut pages = Vec::new();
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(shoutingrobin::crawl::event::CrawlEvent::Page(record)) => {
                pages.push(*record);
            }
            Ok(shoutingrobin::crawl::event::CrawlEvent::Finished { .. }) => break,
            Ok(shoutingrobin::crawl::event::CrawlEvent::Error { url, message }) => {
                eprintln!("crawl error: {url}: {message}");
            }
            Ok(_) => {}
            Err(flume::RecvTimeoutError::Timeout) => {
                cancel.store(true, Ordering::Relaxed);
                break;
            }
            Err(flume::RecvTimeoutError::Disconnected) => break,
        }
    }

    cancel.store(true, Ordering::Relaxed);
    pages
}

#[test]
fn test_status_codes_and_indexability() {
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site(&root_url);

    let _ = server.kill();

    assert!(!pages.is_empty(), "crawl should discover at least one page");

    let home = pages
        .iter()
        .find(|p| p.url.ends_with("/index.html") || p.url.ends_with(&format!(":{port}/")));
    assert!(home.is_some(), "home page should be crawled");
    let home = home.unwrap();
    assert_eq!(
        home.status,
        Some(200),
        "home status should be 200, got {:?}",
        home.status
    );
    assert_eq!(
        home.indexability.as_deref(),
        Some("Indexable"),
        "home should be Indexable"
    );

    let noindex = pages.iter().find(|p| p.url.contains("/noindex.html"));
    assert!(noindex.is_some(), "noindex page should be crawled");
    let noindex = noindex.unwrap();
    assert_eq!(noindex.status, Some(200));
    assert_eq!(noindex.indexability.as_deref(), Some("Non-Indexable"));

    let missing = pages
        .iter()
        .find(|p| p.url.contains("/does-not-exist.html"));
    if let Some(missing) = missing {
        assert_eq!(
            missing.status,
            Some(404),
            "missing page should be 404, got {:?}",
            missing.status
        );
        assert_eq!(
            missing.indexability.as_deref(),
            Some("N/A"),
            "404 page should be N/A, got {:?}",
            missing.indexability
        );
    }
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

/// Verifies the `automation_scripts` solution: every chrome navigation
/// runs `METRICS_AUTOMATION_JS` which injects `<script id="__sr_metrics">`
/// into the DOM, and the HTML analyzer reads the embedded JSON back into
/// `PageRecord` perf fields. Bypasses spider's broken `chrome_store_page`
/// path entirely.
#[test]
fn test_chrome_performance_metrics() {
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=debug".into()),
        )
        .with_test_writer()
        .try_init();

    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(120),
    );

    let _ = server.kill();

    for p in &pages {
        eprintln!(
            "page url={} status={:?} size={} ttfb={:?} cls={:?} a11y_err={} a11y_warn={}",
            p.url, p.status, p.size_bytes, p.ttfb_ms, p.cls, p.a11y_errors, p.a11y_warnings
        );
    }

    let pages_with_perf = pages
        .iter()
        .filter(|p| {
            p.ttfb_ms.is_some() || p.lcp_ms.is_some() || p.cls.is_some() || p.inp_ms.is_some()
        })
        .count();

    assert!(
        pages_with_perf == pages.len(),
        "expected every page to have perf metrics, only {pages_with_perf} of {} did",
        pages.len()
    );
}

#[test]
fn test_chrome_404_root_status() {
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=debug".into()),
        )
        .with_test_writer()
        .try_init();

    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/does-not-exist.html");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(60),
    );

    let _ = server.kill();

    assert!(!pages.is_empty(), "should report at least the root page");

    let root = pages
        .iter()
        .find(|p| p.url.contains("/does-not-exist.html"))
        .expect("404 root URL should appear in pages");

    assert_eq!(
        root.status,
        Some(404),
        "chrome-mode 404 root should report status 404, not {:?}",
        root.status
    );
}

/// Spawns a tiny TCP server that serves a single response for any GET request.
/// Lets the test control status code, headers, and body precisely (unlike
/// `python -m http.server`).
#[allow(clippy::zombie_processes)]
fn spawn_canned_server(
    status_line: &'static str,
    body: String,
) -> (std::thread::JoinHandle<()>, u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();

    let handle = std::thread::spawn(move || {
        use std::io::{Read, Write};
        while !stop_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let _ = stream.read(&mut buf);
                    let response = format!(
                        "{status_line}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {len}\r\nServer: nginx\r\nConnection: close\r\n\r\n{body}",
                        status_line = status_line,
                        len = body.len(),
                        body = body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });

    (handle, port, stop)
}

/// Serves a 200 home page that links to several paths, where some return 404
/// with a large nginx-style HTML body (mirrors the production repro: 200 root
/// linking to /sv/payment which returns 404 + ~100 KB SPA shell).
#[allow(clippy::zombie_processes)]
fn spawn_router_server() -> (std::thread::JoinHandle<()>, u16, Arc<AtomicBool>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind free port");
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();

    let handle = std::thread::spawn(move || {
        use std::io::{Read, Write};

        let mut spa_body = String::from(
            "<!doctype html><html lang=\"sv\"><head><meta charset=\"utf-8\"><title>404</title>",
        );
        while spa_body.len() < 100_000 {
            spa_body.push_str(
                "<style>@font-face{font-family:'Open Sans';src:url('/x.woff2') format('woff2');}</style>",
            );
        }
        spa_body.push_str(
            "</head><body><div id=\"app\"></div><script type=\"module\" src=\"/assets/index.js\"></script></body></html>",
        );

        let home_body = "<!doctype html><html><head><title>Home</title></head><body>\
             <a href=\"/sv/payment\">payment</a>\
             <a href=\"/sv/about\">about</a>\
             <a href=\"/sv/missing\">missing</a>\
             </body></html>"
            .to_string();

        while !stop_flag.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let path = req
                        .lines()
                        .next()
                        .and_then(|l| l.split_whitespace().nth(1))
                        .unwrap_or("/")
                        .to_string();

                    let robots_body = "User-agent: *\nAllow: /\n".to_string();
                    let (status, body): (&str, &str) = if path == "/" {
                        ("HTTP/1.1 200 OK", home_body.as_str())
                    } else if path == "/robots.txt" {
                        ("HTTP/1.1 200 OK", robots_body.as_str())
                    } else {
                        ("HTTP/1.1 404 Not Found", spa_body.as_str())
                    };

                    let response = format!(
                        "{status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {len}\r\nServer: nginx\r\nConnection: close\r\n\r\n{body}",
                        status = status,
                        len = body.len(),
                        body = body
                    );
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
    });

    (handle, port, stop)
}

/// Reproduces the production repro: 200 root → linked 404 child page in chrome
/// mode. This is the path that returns the wrong status in the real crawler.
#[test]
fn test_chrome_404_subsequent_page() {
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=debug".into()),
        )
        .with_test_writer()
        .try_init();

    let (handle, port, stop) = spawn_router_server();
    let root_url = format!("http://127.0.0.1:{port}/");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(120),
    );

    stop.store(true, Ordering::Relaxed);
    let _ = handle.join();

    eprintln!(
        "received {} pages: {:?}",
        pages.len(),
        pages
            .iter()
            .map(|p| (p.url.clone(), p.status))
            .collect::<Vec<_>>()
    );

    let payment = pages
        .iter()
        .find(|p| p.url.contains("/sv/payment"))
        .unwrap_or_else(|| panic!("expected /sv/payment to be crawled, got {pages:?}"));

    assert_eq!(
        payment.status,
        Some(404),
        "subsequent chrome-mode 404 should report status 404, not {:?}. \
         Mirrors the production bug on https://ro-se.envro.nextbatt.biz/sv/payment.",
        payment.status
    );
}

#[test]
fn test_chrome_404_with_large_spa_body() {
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }

    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=debug".into()),
        )
        .with_test_writer()
        .try_init();

    // Produce a ~100KB SPA shell body to mirror the real-world repro
    // (nginx-served Vue/SvelteKit 404 with a fully-rendered HTML shell).
    let mut body = String::from(
        "<!doctype html><html lang=\"sv\"><head><meta charset=\"utf-8\"><title>404</title>",
    );
    while body.len() < 100_000 {
        body.push_str(
            "<style>@font-face{font-family:'Open Sans';src:url('/x.woff2') format('woff2');}</style>",
        );
    }
    body.push_str(
        "</head><body><div id=\"app\"></div><script type=\"module\" src=\"/assets/index.js\"></script></body></html>",
    );

    let (handle, port, stop) = spawn_canned_server("HTTP/1.1 404 Not Found", body);
    let root_url = format!("http://127.0.0.1:{port}/sv/payment");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome,
        Duration::from_secs(60),
    );

    stop.store(true, Ordering::Relaxed);
    let _ = handle.join();

    assert!(!pages.is_empty(), "should report at least the root page");

    let root = pages
        .iter()
        .find(|p| p.url.contains("/sv/payment"))
        .unwrap_or_else(|| panic!("expected /sv/payment in pages, got {pages:?}"));

    assert_eq!(
        root.status,
        Some(404),
        "chrome-mode large SPA 404 should report status 404, not {:?}. \
         This regression matches the production bug on https://ro-se.envro.nextbatt.biz/sv/payment.",
        root.status
    );
}

#[test]
fn test_http_404_root_status() {
    let (mut server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/does-not-exist.html");

    let pages = crawl_test_site_with_mode(
        &root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Http,
        Duration::from_secs(30),
    );

    let _ = server.kill();

    assert!(!pages.is_empty(), "should report at least the root page");

    let root = pages
        .iter()
        .find(|p| p.url.contains("/does-not-exist.html"))
        .expect("404 root URL should appear in pages");

    assert_eq!(
        root.status,
        Some(404),
        "http-mode 404 root should report status 404, not {:?}",
        root.status
    );
}
