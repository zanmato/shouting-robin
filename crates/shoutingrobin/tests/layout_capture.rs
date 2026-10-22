//! Layout capture in Chrome crawls, checked against the shop fixture in
//! `test-site/shop/` (see its README section for what each page holds).

mod common;

use common::*;
use shoutingrobin::crawl::LayoutCapture;
use shoutingrobin::crawl::layout::{LayoutTables, PageLayout};
use shoutingrobin::crawl::layout_groups::Assignment;
use shoutingrobin::crawl::render_mode::RenderMode;
use std::time::Duration;

const SHOP_PAGES: usize = 12;

fn shop_options(layout_capture: LayoutCapture) -> TestCrawlOptions {
    TestCrawlOptions {
        render_mode: RenderMode::Chrome,
        timeout: Duration::from_secs(180),
        check_resources: false,
        layout_capture,
        follow_sitemaps: false,
    }
}

fn crawl_shop(database: &TestDatabase, port: u16, layout_capture: LayoutCapture) -> i64 {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "shoutingrobin=warn".into()),
        )
        .with_test_writer()
        .try_init();
    let (_, crawl_id) = crawl_into(
        database,
        &format!("http://127.0.0.1:{port}/shop/"),
        shop_options(layout_capture),
    );
    crawl_id.expect("the shop crawl should finish")
}

fn load_layout(database: &TestDatabase, crawl_id: i64, url: &str) -> (PageLayout, LayoutTables) {
    database.runtime.block_on(async {
        let tables = shoutingrobin::storage::load_layout_tables(&database.pool, crawl_id)
            .await
            .expect("load layout tables");
        let encoded = shoutingrobin::storage::load_page_layout(&database.pool, crawl_id, url)
            .await
            .expect("load page layout")
            .unwrap_or_else(|| panic!("{url} should have a stored layout"));
        (
            PageLayout::decode(&encoded).expect("decode stored layout"),
            tables,
        )
    })
}

fn count_keys(layout: &PageLayout, tables: &LayoutTables, key: &str) -> usize {
    layout
        .boxes
        .iter()
        .filter(|layout_box| tables.key(layout_box.key) == key)
        .count()
}

#[test]
fn chrome_crawl_stores_the_layout_of_every_shop_page() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }
    let (mut server, port) = spawn_http_server();
    let database = test_database();
    let crawl_id = crawl_shop(&database, port, LayoutCapture::Loaded);

    let stored = database
        .runtime
        .block_on(shoutingrobin::storage::count_page_layouts(
            &database.pool,
            crawl_id,
        ))
        .expect("count layouts");
    assert_eq!(
        stored, SHOP_PAGES as i64,
        "every HTML page of the shop, and not the missing cart page"
    );

    let product_url = format!("http://127.0.0.1:{port}/shop/product/charger-20w.html");
    let (product, tables) = load_layout(&database, crawl_id, &product_url);
    server.kill();

    assert!(product.types.iter().any(|type_name| type_name == "Product"));
    assert_eq!(product.width, 1366, "the viewport the crawl renders at");
    assert_eq!(count_keys(&product, &tables, "h1.buy-box__title"), 1);
    assert_eq!(count_keys(&product, &tables, "li.product-card"), 4);
    assert_eq!(
        count_keys(&product, &tables, "li.review"),
        0,
        "reviews load when scrolled into view, and this capture does not scroll"
    );
}

#[test]
fn scrolled_capture_includes_lazy_reviews() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }
    let (mut server, port) = spawn_http_server();
    let database = test_database();
    let crawl_id = crawl_shop(&database, port, LayoutCapture::Scrolled);

    let (charger, tables) = load_layout(
        &database,
        crawl_id,
        &format!("http://127.0.0.1:{port}/shop/product/charger-20w.html"),
    );
    let (headphones, headphone_tables) = load_layout(
        &database,
        crawl_id,
        &format!("http://127.0.0.1:{port}/shop/product/headphones-anc.html"),
    );
    server.kill();

    assert_eq!(count_keys(&charger, &tables, "li.review"), 3);
    assert_eq!(
        count_keys(&headphones, &headphone_tables, "li.review"),
        0,
        "the headphones have no reviews"
    );
}

#[test]
fn layout_is_not_captured_unless_asked() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }
    let (mut server, port) = spawn_http_server();
    let database = test_database();
    let crawl_id = crawl_shop(&database, port, LayoutCapture::Off);
    server.kill();

    let stored = database
        .runtime
        .block_on(shoutingrobin::storage::count_page_layouts(
            &database.pool,
            crawl_id,
        ))
        .expect("count layouts");
    assert_eq!(stored, 0);
}

/// The shop's pages group into the templates a person would name: the
/// products (with the one lacking Product markup flagged), the categories
/// and the home page, with the free-form info pages kept out of all three.
#[test]
fn shop_pages_group_into_their_templates() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }
    let (mut server, port) = spawn_http_server();
    let database = test_database();
    let crawl_id = crawl_shop(&database, port, LayoutCapture::Loaded);
    let root_url = format!("http://127.0.0.1:{port}/shop/");
    let pages = database
        .runtime
        .block_on(shoutingrobin::storage::load_pages_for_crawl(
            &database.pool,
            crawl_id,
            &root_url,
        ))
        .expect("load pages");
    server.kill();

    let layout_of = |path: &str| {
        pages
            .iter()
            .find(|page| path_of(&page.url) == path)
            .and_then(|page| page.layout.clone())
            .unwrap_or_else(|| panic!("{path} should have a layout"))
    };
    let template_of = |path: &str| {
        layout_of(path)
            .template
            .unwrap_or_else(|| panic!("{path} should be in a template"))
    };

    for product in [
        "/shop/product/charger-20w.html",
        "/shop/product/charger-65w.html",
        "/shop/product/cable-2m.html",
        "/shop/product/headphones-anc.html",
    ] {
        let layout = layout_of(product);
        assert_eq!(layout.template.as_deref(), Some("Product"), "{product}");
        assert_eq!(
            layout.assignment,
            Some(Assignment::StructuredData),
            "{product}"
        );
        assert!(!layout.misses_template_markup());
    }
    let powerbank = layout_of("/shop/product/powerbank-10k.html");
    assert_eq!(
        powerbank.template.as_deref(),
        Some("Product"),
        "its layout is a product page's"
    );
    assert_eq!(powerbank.assignment, Some(Assignment::LayoutWithoutMarkup));
    assert!(powerbank.misses_template_markup());

    let categories: Vec<String> = [
        "/shop/category/chargers.html",
        "/shop/category/cables.html",
        "/shop/category/headphones.html",
    ]
    .iter()
    .map(|path| template_of(path))
    .collect();
    assert!(categories.iter().all(|template| template == &categories[0]));
    let home = template_of("/shop/");
    assert_ne!(home, categories[0]);
    assert_ne!(home, "Product");
    assert_ne!(categories[0], "Product");
    // The info pages are free-form, built from different blocks, so they
    // may each be a layout of their own. They must not join another kind.
    for path in [
        "/shop/info/shipping.html",
        "/shop/info/returns.html",
        "/shop/info/about.html",
    ] {
        let template = template_of(path);
        assert!(
            template != "Product" && template != categories[0] && template != home,
            "{path} joined {template}"
        );
    }

    let products = pages
        .iter()
        .filter(|page| {
            page.layout
                .as_ref()
                .is_some_and(|layout| layout.template.as_deref() == Some("Product"))
        })
        .count();
    assert_eq!(products, 5, "four with Product markup and the power bank");
}

fn change_labels(
    database: &TestDatabase,
    baseline_crawl_id: i64,
    crawl_id: i64,
    url: &str,
) -> Vec<String> {
    let comparison = database
        .runtime
        .block_on(shoutingrobin::crawl::layout::load_page_comparison(
            &database.pool,
            Some(baseline_crawl_id),
            crawl_id,
            url,
        ))
        .expect("load comparison")
        .unwrap_or_else(|| panic!("{url} should have a layout"));
    comparison
        .diff
        .layout_changes()
        .map(|change| format!("{} {}", change.label(), change.kinds_text()))
        .collect()
}

fn has_change(labels: &[String], key: &str, kind: &str) -> bool {
    labels
        .iter()
        .any(|label| label.starts_with(&format!("{key} ")) && label.contains(kind))
}

/// Two plain crawls differ only in content (related products reordered, a
/// new "viewing now" text), which is no layout change. A theme change
/// between crawls is found on the pages it touches, and only there.
#[test]
fn theme_changes_are_found_and_reloads_are_not() {
    let _guard = CRAWL_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    if !chrome_available() {
        eprintln!("skipping: no chrome binary on PATH");
        return;
    }
    let (mut server, port) = spawn_http_server();
    let database = test_database();
    let baseline = crawl_shop(&database, port, LayoutCapture::Loaded);
    let reload = crawl_shop(&database, port, LayoutCapture::Loaded);
    server.set_shop_layout_variant(true);
    let themed = crawl_shop(&database, port, LayoutCapture::Loaded);
    server.kill();

    let unchanged = database
        .runtime
        .block_on(shoutingrobin::crawl::layout::compare_crawls(
            &database.pool,
            baseline,
            reload,
        ))
        .expect("compare plain crawls");
    assert!(
        unchanged.is_empty(),
        "a reload changes no layout: {unchanged:#?}"
    );

    let changed = database
        .runtime
        .block_on(shoutingrobin::crawl::layout::compare_crawls(
            &database.pool,
            reload,
            themed,
        ))
        .expect("compare themed crawl");
    let stored: Vec<i64> = [baseline, reload, themed]
        .iter()
        .map(|crawl_id| {
            database
                .runtime
                .block_on(shoutingrobin::storage::count_page_layouts(
                    &database.pool,
                    *crawl_id,
                ))
                .expect("count layouts")
        })
        .collect();
    assert_eq!(
        stored, [SHOP_PAGES as i64; 3],
        "every crawl stores every page's layout"
    );
    let mut unchanged_pages: Vec<String> = Vec::new();
    for path in [
        "/shop/",
        "/shop/info/shipping.html",
        "/shop/info/returns.html",
        "/shop/info/about.html",
        "/shop/category/chargers.html",
        "/shop/category/cables.html",
        "/shop/category/headphones.html",
        "/shop/product/charger-20w.html",
        "/shop/product/charger-65w.html",
        "/shop/product/cable-2m.html",
        "/shop/product/headphones-anc.html",
        "/shop/product/powerbank-10k.html",
    ] {
        if !changed.contains_key(&format!("http://127.0.0.1:{port}{path}")) {
            unchanged_pages.push(path.to_string());
        }
    }
    assert!(
        unchanged_pages.is_empty(),
        "the theme touches every page, these show no change: {unchanged_pages:?}"
    );

    let url = |path: &str| format!("http://127.0.0.1:{port}{path}");
    let product = change_labels(
        &database,
        reload,
        themed,
        &url("/shop/product/charger-20w.html"),
    );
    assert!(
        has_change(&product, "h1.buy-box__title", "restyled"),
        "{product:?}"
    );
    assert!(
        has_change(&product, "nav.breadcrumbs", "removed"),
        "{product:?}"
    );
    assert!(
        has_change(&product, "ul.trust-badges", "removed"),
        "{product:?}"
    );
    assert!(
        has_change(&product, "footer.site-footer", "restyled"),
        "{product:?}"
    );

    let category = change_labels(
        &database,
        reload,
        themed,
        &url("/shop/category/cables.html"),
    );
    assert!(
        has_change(&category, "nav.breadcrumbs", "removed"),
        "{category:?}"
    );
    assert!(
        has_change(&category, "footer.site-footer", "restyled"),
        "{category:?}"
    );
    assert!(
        !category
            .iter()
            .any(|label| label.starts_with("ul.trust-badges")),
        "category pages have no trust badges: {category:?}"
    );

    let home = change_labels(&database, reload, themed, &url("/shop/"));
    assert!(
        has_change(&home, "footer.site-footer", "restyled"),
        "{home:?}"
    );
    assert!(
        !home
            .iter()
            .any(|label| label.starts_with("nav.breadcrumbs")),
        "the home page has no breadcrumbs: {home:?}"
    );
}
