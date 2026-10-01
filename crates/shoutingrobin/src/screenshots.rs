//! Stages the app for the website screenshots. Built only with the
//! `screenshots` feature and driven by `scripts/screenshots/capture`, which
//! runs Shouting Robin on a virtual display against a throwaway data
//! directory.
//!
//! `SR_SCENE` names the scene to stage, `SR_THEME` the theme and
//! `SR_SCENE_READY` the file written once the window shows the scene.
//! `SR_CRAWL_FIXTURE` is the SQL file of recorded crawls the scenes show. The
//! `record` scene makes that file's data instead: it crawls `SR_RECORD_URL`
//! over HTTP and then in Chrome, and the script dumps the result.

use std::time::Duration;

use gpui_kit::{App, AsyncWindowContext, Bounds, Entity, Pixels, Window, point, px, size};

use crate::app::ShoutingRobinApp;
use crate::app_database::AppDatabase;
use crate::crawl::RenderMode;
use crate::crawl::event::PageRecord;
use crate::views::ResultTab;

/// The size of the virtual display the script starts.
const WINDOW_WIDTH: f32 = 1600.;
const WINDOW_HEIGHT: f32 = 1000.;

/// How long before the capture each recorded crawl started, oldest first, so
/// the history reads like crawls run over a few days rather than seconds
/// apart on the day the fixture was recorded.
const CRAWL_AGES: [i64; 2] = [3 * 86_400 + 4 * 3600, 2 * 3600];

/// What the window shows when the screenshot is taken.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scene {
    /// Crawls the test site twice to make the fixture the other scenes show.
    Record,
    /// The Overview tab of the latest crawl, its findings sorted by priority.
    Overview,
    /// The Page Titles tab with a page selected and its details beside the
    /// grid.
    Details,
    /// Accessibility violations from the Chrome crawl.
    Accessibility,
    /// The Ecommerce tab with the product page selected.
    Ecommerce,
    /// What changed since the previous crawl of the same site.
    Changes,
}

impl Scene {
    pub(crate) fn from_env() -> Option<Self> {
        let scene = std::env::var("SR_SCENE").ok()?;
        match scene.as_str() {
            "record" => Some(Self::Record),
            "overview" => Some(Self::Overview),
            "details" => Some(Self::Details),
            "ecommerce" => Some(Self::Ecommerce),
            "accessibility" => Some(Self::Accessibility),
            "changes" => Some(Self::Changes),
            _ => {
                tracing::error!("Unknown SR_SCENE {scene:?}");
                None
            }
        }
    }

    fn tab(self) -> ResultTab {
        match self {
            Self::Record | Self::Overview => ResultTab::Overview,
            Self::Details => ResultTab::PageTitles,
            Self::Accessibility => ResultTab::Accessibility,
            Self::Ecommerce => ResultTab::Ecommerce,
            Self::Changes => ResultTab::Changes,
        }
    }

    /// The path of the page the scene selects, for the details panel.
    fn selected_path(self) -> Option<&'static str> {
        match self {
            Self::Details => Some("/redirect-target.html"),
            Self::Ecommerce => Some("/about.html"),
            Self::Accessibility => Some("/a11y.html"),
            Self::Changes => Some("/spa.html"),
            Self::Record | Self::Overview => None,
        }
    }
}

/// The window fills the virtual display from its top left corner.
pub(crate) fn window_bounds() -> Option<Bounds<Pixels>> {
    Scene::from_env()?;
    Some(Bounds::new(
        point(px(0.), px(0.)),
        size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)),
    ))
}

/// Fill a fresh app database with the settings and, for every scene but
/// `record`, the recorded crawls. Does nothing when the database already
/// holds a crawl, so a data directory that was not thrown away is never
/// written to twice.
pub(crate) fn seed(database: &AppDatabase) {
    if let Err(error) = smol::block_on(seed_database(database)) {
        tracing::error!("Failed to seed the screenshot database: {error:#}");
    }
}

async fn seed_database(database: &AppDatabase) -> anyhow::Result<()> {
    let pool = database.pool();
    let crawls: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM crawls")
        .fetch_one(pool)
        .await?;
    if crawls > 0 {
        return Ok(());
    }

    let theme = std::env::var("SR_THEME").unwrap_or_else(|_| "Catppuccin Latte".into());
    for (key, value) in [
        ("appearance.theme", theme.as_str()),
        // The update check would reach GitHub and could put an update button
        // in the title bar.
        ("general.check_for_updates", "false"),
    ] {
        database.save_setting(key, value).await?;
    }

    if Scene::from_env() == Some(Scene::Record) {
        return Ok(());
    }

    let fixture_path = std::env::var("SR_CRAWL_FIXTURE")?;
    let fixture = std::fs::read_to_string(&fixture_path)?;
    let mut transaction = pool.begin().await?;
    sqlx::raw_sql(&fixture).execute(&mut *transaction).await?;

    let started: Vec<(i64, i64)> = sqlx::query_as("SELECT id, started_at FROM crawls ORDER BY id")
        .fetch_all(&mut *transaction)
        .await?;
    let now = chrono::Utc::now().timestamp();
    for ((crawl_id, started_at), age) in started.into_iter().zip(CRAWL_AGES) {
        let shift = now - age - started_at;
        sqlx::query(
            "UPDATE crawls SET started_at = started_at + ?1, finished_at = finished_at + ?1 \
             WHERE id = ?2",
        )
        .bind(shift)
        .bind(crawl_id)
        .execute(&mut *transaction)
        .await?;
        sqlx::query("UPDATE pages SET crawled_at = crawled_at + ?1 WHERE crawl_id = ?2")
            .bind(shift)
            .bind(crawl_id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

/// Stage `scene` in the window that holds `app`, then write the ready file.
pub(crate) fn stage(scene: Scene, app: Entity<ShoutingRobinApp>, window: &Window, cx: &App) {
    window
        .spawn(cx, async move |cx| {
            let staged = match scene {
                Scene::Record => record(&app, cx).await,
                _ => stage_scene(scene, &app, cx).await,
            };
            if let Err(error) = staged {
                tracing::error!("Failed to stage the screenshot scene: {error:#}");
                return;
            }
            if let Ok(path) = std::env::var("SR_SCENE_READY")
                && let Err(error) = std::fs::write(&path, "ready")
            {
                tracing::error!("Failed to write {path}: {error}");
            }
        })
        .detach();
}

/// Crawl the site over HTTP and then in Chrome, the way a user starts a crawl
/// from the crawl bar. The Chrome crawl is the newer one, so the scenes show
/// it with the HTTP crawl as its baseline.
async fn record(app: &Entity<ShoutingRobinApp>, cx: &mut AsyncWindowContext) -> anyhow::Result<()> {
    let url = std::env::var("SR_RECORD_URL")?;
    let (crawl_bar, status_bar) = app.read_with(cx, |app, _| {
        (app.crawl_bar().clone(), app.status_bar().clone())
    });
    settle(cx, 1000).await;

    for mode in [RenderMode::Http, RenderMode::Chrome] {
        crawl_bar.update_in(cx, |bar, window, cx| {
            bar.url_input
                .update(cx, |input, cx| input.set_value(url.clone(), window, cx));
            bar.start_crawl(mode, cx);
        })?;
        settle(cx, 500).await;
        let mut finished = false;
        // The Chrome crawl waits for each page to go idle, which takes a
        // few seconds a page.
        for _ in 0..6000 {
            settle(cx, 100).await;
            if !status_bar.read_with(cx, |status, _| status.running) {
                finished = true;
                break;
            }
        }
        if !finished {
            anyhow::bail!("the {mode:?} crawl did not finish");
        }
    }
    Ok(())
}

async fn stage_scene(
    scene: Scene,
    app: &Entity<ShoutingRobinApp>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let (crawl_bar, sidebar, results_grid, status_bar) = app.read_with(cx, |app, _| {
        (
            app.crawl_bar().clone(),
            app.sidebar().clone(),
            app.results_grid().clone(),
            app.status_bar().clone(),
        )
    });

    let pool = cx.update(|_, cx| AppDatabase::global(cx).pool().clone())?;
    let (crawl_id, root_url): (i64, String) =
        sqlx::query_as("SELECT id, root_url FROM crawls ORDER BY id DESC LIMIT 1")
            .fetch_one(&pool)
            .await?;

    // The history loads in the background after the window opens.
    let mut opened = false;
    for _ in 0..100 {
        settle(cx, 50).await;
        opened = sidebar.update(cx, |sidebar, cx| sidebar.open(crawl_id, cx));
        if opened {
            break;
        }
    }
    if !opened {
        anyhow::bail!("crawl {crawl_id} is not in the history");
    }

    // Opening a crawl leaves the bar as it was, but after a crawl it holds
    // the URL that was crawled.
    crawl_bar.update_in(cx, |bar, window, cx| {
        bar.url_input.update(cx, |input, cx| {
            input.set_value(root_url.clone(), window, cx)
        });
    })?;

    let mut loaded = false;
    for _ in 0..200 {
        settle(cx, 50).await;
        if results_grid.read_with(cx, |grid, cx| !grid.is_loading() && grid.has_results(cx)) {
            loaded = true;
            break;
        }
    }
    if !loaded {
        anyhow::bail!("crawl {crawl_id} did not load");
    }

    // The status bar counts what a crawl streams in, so after a crawl it
    // shows these totals. Opening one from the history leaves it at zero.
    let (pages, resources) =
        results_grid.read_with(cx, |grid, cx| grid.page_and_resource_counts(cx));
    status_bar.update(cx, |status, cx| {
        status.pages = pages;
        status.resources = resources;
        cx.notify();
    });

    app.update(cx, |app, cx| app.select_tab(scene.tab(), cx));
    settle(cx, 300).await;

    if let Some(path) = scene.selected_path() {
        let selected = results_grid.update(cx, |grid, cx| {
            grid.select_row_where(|page: &PageRecord| url_path(&page.url) == path, cx)
        });
        if !selected {
            anyhow::bail!("no row for {path}");
        }
    }

    settle(cx, 1500).await;
    Ok(())
}

fn url_path(url: &str) -> &str {
    let after_scheme = url.split_once("://").map_or(url, |(_, rest)| rest);
    after_scheme
        .find('/')
        .map_or("/", |slash| &after_scheme[slash..])
}

async fn settle(cx: &mut AsyncWindowContext, millis: u64) {
    cx.background_executor()
        .timer(Duration::from_millis(millis))
        .await;
}
