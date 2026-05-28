//! Minimal chrome smoke test: spawns the local test-site, crawls it with a
//! headless chrome, and screenshots every page. If each page prints a 📸 we
//! know chrome launched and `get_chrome_page()` works end to end.
//!
//! cargo run -p shoutingrobin --example chrome_screenshot

use std::net::TcpListener;
use std::process::{Child, Command};
use std::time::Duration;

use spider::configuration::WaitForIdleNetwork;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::tokio;
use spider::utils::create_output_path;
use spider::website::Website;

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

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

#[tokio::main]
async fn main() {
    // Clear any stale chromiumoxide singleton lock so chrome launches.
    for name in ["SingletonSocket", "SingletonCookie", "SingletonLock"] {
        let path = std::path::Path::new("/tmp/chromiumoxide-runner").join(name);
        let _ = std::fs::remove_file(path);
    }

    let (_server, port) = spawn_http_server();
    let root_url = format!("http://127.0.0.1:{port}/");
    println!("crawling {root_url}");

    let mut website: Website = Website::new(&root_url);
    website.configuration.request_timeout = Some(Duration::from_secs(60));
    // Mirror the engine's chrome config to find what breaks the handle / stalls.
    website
        .with_chrome_intercept(RequestInterceptConfiguration::new(true))
        .with_stealth(true)
        .with_wait_for_idle_network(Some(WaitForIdleNetwork::new(Some(Duration::from_secs(30)))));

    let mut rx = website.subscribe(18);
    let mut rxg = website.subscribe_guard().unwrap();

    let pump = tokio::spawn(async move {
        let mut count = 0u32;
        let mut ok = 0u32;
        while let Ok(mut page) = rx.recv().await {
            let has_chrome = page.get_chrome_page().is_some();
            let output_dir = std::env::temp_dir().join("shoutingrobin-screenshots");
            let output_path = create_output_path(&output_dir, page.get_url(), ".png").await;
            let bytes = page
                .screenshot(
                    true,
                    true,
                    spider::configuration::CaptureScreenshotFormat::Png,
                    Some(75),
                    Some(output_path),
                    None,
                )
                .await;
            count += 1;
            if !bytes.is_empty() {
                ok += 1;
            }
            println!(
                "{} chrome_page={has_chrome} bytes={} {}",
                if bytes.is_empty() { "🚫" } else { "📸" },
                bytes.len(),
                page.get_url()
            );
            page.close_page().await;
            rxg.inc();
        }
        println!("\n{ok}/{count} pages screenshotted");
    });

    let start = tokio::time::Instant::now();
    website.crawl().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    website.unsubscribe();
    let _ = pump.await;

    println!("crawl finished in {:?}", start.elapsed());
}
