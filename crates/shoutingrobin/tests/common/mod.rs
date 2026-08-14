use std::net::TcpListener as StdTcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;

/// Serialises the integration tests. They share a single Chrome profile dir and
/// rewrite `test-site/sitemap.xml`, so they must not run concurrently.
pub static CRAWL_TEST_MUTEX: Mutex<()> = Mutex::new(());

fn test_site_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("test-site")
}

/// Notifies the background server to stop accepting connections, both on an
/// explicit `kill()` and on drop, so a panicking test still tears it down.
pub struct ServerGuard {
    shutdown: Arc<Notify>,
}

impl ServerGuard {
    pub fn kill(&mut self) {
        self.shutdown.notify_one();
    }
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        self.shutdown.notify_one();
    }
}

/// Serves `test-site/` over HTTP on a free port using a tokio runtime on its own
/// thread, so it doesn't contend with the multi-threaded runtime the test uses
/// to drive the crawl. Returns once the listener is accepting connections.
pub fn spawn_http_server() -> (ServerGuard, u16) {
    let listener = StdTcpListener::bind("127.0.0.1:0").expect("bind free port");
    let port = listener.local_addr().unwrap().port();
    listener
        .set_nonblocking(true)
        .expect("set listener nonblocking");

    write_sitemap(port);

    let test_site_dir = test_site_dir();
    let shutdown = Arc::new(Notify::new());

    std::thread::spawn({
        let shutdown = shutdown.clone();
        move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("build server runtime");
            runtime.block_on(async move {
                let listener = TcpListener::from_std(listener).expect("convert listener to tokio");
                loop {
                    tokio::select! {
                        _ = shutdown.notified() => break,
                        accepted = listener.accept() => {
                            if let Ok((stream, _)) = accepted {
                                let dir = test_site_dir.clone();
                                tokio::spawn(async move {
                                    if let Err(error) = handle_connection(stream, dir).await {
                                        eprintln!("test server connection error: {error}");
                                    }
                                });
                            }
                        }
                    }
                }
            });
        }
    });

    for _ in 0..100 {
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return (ServerGuard { shutdown }, port);
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    panic!("test http server did not start within 2s");
}

async fn handle_connection(mut stream: TcpStream, test_site_dir: PathBuf) -> std::io::Result<()> {
    let mut buffer = [0u8; 8192];
    let read = stream.read(&mut buffer).await?;
    if read == 0 {
        return Ok(());
    }

    // A small deterministic latency so Chrome measures a non-zero TTFB. On
    // loopback a Rust server otherwise responds in microseconds, which rounds to
    // 0ms and makes performance-metric assertions environment-dependent.
    tokio::time::sleep(Duration::from_millis(10)).await;

    let request = String::from_utf8_lossy(&buffer[..read]);
    let request_line = request.lines().next().unwrap_or("");
    let raw_path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let path = raw_path.split(['?', '#']).next().unwrap_or("/");

    let relative = path.trim_start_matches('/');
    let relative = if relative.is_empty() {
        "index.html"
    } else {
        relative
    };

    // canonicalize() resolves `..` and fails for missing files, so a path that
    // escapes the test-site root or doesn't exist falls through to a 404.
    let candidate = test_site_dir.join(relative);
    let within_root = match (candidate.canonicalize(), test_site_dir.canonicalize()) {
        (Ok(resolved), Ok(root)) if resolved.starts_with(&root) && resolved.is_file() => {
            Some(resolved)
        }
        _ => None,
    };

    let Some(file_path) = within_root else {
        return send_404(&mut stream).await;
    };

    let body = std::fs::read(&file_path)?;
    let content_type = content_type_for(&file_path);
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes()).await?;
    stream.write_all(&body).await?;
    stream.flush().await
}

async fn send_404(stream: &mut TcpStream) -> std::io::Result<()> {
    let body = "404 Not Found";
    let response = format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("txt") => "text/plain",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

pub fn write_sitemap(port: u16) {
    let base = format!("http://127.0.0.1:{port}");
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>{base}/</loc></url>
  <url><loc>{base}/index.html</loc></url>
  <url><loc>{base}/about.html</loc><lastmod>2026-08-01</lastmod></url>
  <url><loc>{base}/orphan-page.html</loc><lastmod>2026-07-15T09:30:00+02:00</lastmod></url>
</urlset>
"#
    );
    let path = test_site_dir().join("sitemap.xml");
    std::fs::write(&path, xml).expect("write sitemap.xml");
}

pub fn crawl_test_site(root_url: &str) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    crawl_test_site_with_mode(
        root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Http,
        Duration::from_secs(30),
    )
}

/// Crawls with the post-crawl resource pass enabled, which is off for every
/// other test: it requests each discovered image, stylesheet, script and
/// external link. The fixture's off-site URLs are all `.invalid`, which cannot
/// resolve, so nothing leaves the machine.
pub fn crawl_test_site_checking_resources(
    root_url: &str,
) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    crawl_test_site_inner(
        root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Http,
        Duration::from_secs(30),
        false,
        true,
    )
}

pub fn chrome_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static CHROME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let guard = CHROME_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    for name in ["SingletonSocket", "SingletonCookie", "SingletonLock"] {
        let path = Path::new("/tmp/chromiumoxide-runner").join(name);
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
    guard
}

/// Crawls the test site and returns the records as the app sees them *after*
/// the crawl finishes: reloaded from the database, so they carry the results of
/// the post-crawl passes (link aggregation, depth, PageRank, near-duplicates,
/// hreflang validation) that the streamed records never have.
pub fn crawl_test_site_reloaded(root_url: &str) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    crawl_test_site_inner(
        root_url,
        shoutingrobin::crawl::render_mode::RenderMode::Http,
        Duration::from_secs(30),
        true,
        false,
    )
}

pub fn crawl_test_site_with_mode(
    root_url: &str,
    render_mode: shoutingrobin::crawl::render_mode::RenderMode,
    timeout: Duration,
) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    crawl_test_site_inner(root_url, render_mode, timeout, false, false)
}

fn crawl_test_site_inner(
    root_url: &str,
    render_mode: shoutingrobin::crawl::render_mode::RenderMode,
    timeout: Duration,
    reload_after_finish: bool,
    check_resources: bool,
) -> Vec<shoutingrobin::crawl::event::PageRecord> {
    let _chrome_guard = matches!(
        render_mode,
        shoutingrobin::crawl::render_mode::RenderMode::Chrome
    )
    .then(chrome_test_guard);

    let cancel = Arc::new(AtomicBool::new(false));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(8 * 1024 * 1024)
        .build()
        .unwrap();
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
            pool.clone(),
            render_mode,
            shoutingrobin::crawl::CrawlConfig {
                max_pages: 0,
                max_concurrent: 10,
                delay_ms: 0,
                timeout_seconds: 30,
                respect_robots_txt: true,
                follow_sitemaps: true,
                block_images: false,
                near_duplicate_threshold: 90,
                content_selector: String::new(),
                user_agent: None,
                extra_headers: Vec::new(),
                include_patterns: Vec::new(),
                exclude_patterns: Vec::new(),
                crawl_subdomains: false,
                list_mode: false,
                seed_urls: Vec::new(),
                check_resources,
            },
        )
    };

    rt.spawn(async move {
        fut.await;
    });

    let mut pages = Vec::new();
    let mut finished_crawl_id = None;
    let start = std::time::Instant::now();
    loop {
        let Some(remaining) = timeout.checked_sub(start.elapsed()) else {
            cancel.store(true, Ordering::Relaxed);
            break;
        };
        match rx.recv_timeout(remaining) {
            Ok(shoutingrobin::crawl::event::CrawlEvent::Page(record)) => {
                pages.push(*record);
            }
            Ok(shoutingrobin::crawl::event::CrawlEvent::Finished { crawl_id, .. }) => {
                finished_crawl_id = Some(crawl_id);
                break;
            }
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

    if reload_after_finish {
        let crawl_id = finished_crawl_id.expect("crawl should report a finished event");
        return rt.block_on(async {
            shoutingrobin::storage::load_pages_for_crawl(&pool, crawl_id, root_url)
                .await
                .expect("reload pages after crawl")
        });
    }

    pages
}

pub fn chrome_available() -> bool {
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

/// Finds a crawled page by path. Prefers an exact path match so a lookup for
/// `/about.html` can't return `/about.html?ref=nav&page=2`, and only falls back
/// to a substring match for callers passing a fragment of a URL.
pub fn find_page<'a>(
    pages: &'a [shoutingrobin::crawl::event::PageRecord],
    substr: &str,
) -> Option<&'a shoutingrobin::crawl::event::PageRecord> {
    pages
        .iter()
        .find(|p| path_of(&p.url) == substr)
        .or_else(|| pages.iter().find(|p| p.url.contains(substr)))
}

pub fn path_of(url: &str) -> String {
    let after = url.split_once("://").map(|x| x.1).unwrap_or(url);
    match after.find('/') {
        Some(i) => after[i..].to_string(),
        None => "/".to_string(),
    }
}

pub fn page_paths(pages: &[shoutingrobin::crawl::event::PageRecord]) -> Vec<String> {
    pages.iter().map(|p| path_of(&p.url)).collect()
}

pub fn ssr_diff_pct(record: &shoutingrobin::crawl::event::PageRecord) -> u32 {
    match (record.word_count, record.ssr_word_count) {
        (Some(csr), Some(ssr)) if csr > 0 => {
            (csr.saturating_sub(ssr) as f64 / csr as f64 * 100.0).round() as u32
        }
        _ => 100,
    }
}
