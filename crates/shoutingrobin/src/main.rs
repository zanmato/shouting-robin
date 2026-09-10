#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

mod a11y_rules;
mod app;
mod app_database;
mod app_settings;
mod assets;
mod crawl;
mod report;
mod result_ext;
mod settings;
mod storage;
mod themes_manager;
mod ui;
mod update_manager;
mod views;

#[cfg(test)]
mod filter_coverage;
#[cfg(test)]
mod parity_export;

use assets::Assets;
use gpui_kit::application;
use gpui_kit::component::ThemeRegistry;
use gpui_kit::{AppContext, SharedString, WindowBounds, WindowOptions, px, size};

use app::ShoutingRobinApp;
use app_settings::AppSettings;
use crawl::CrawlEngine;
use settings::Settings;
use themes_manager::ThemesManager;

fn main() {
    tracing_subscriber::fmt::init();

    let app = application()
        .with_quit_mode(gpui_kit::QuitMode::LastWindowClosed)
        .with_assets(Assets);

    app.run(move |cx| {
        gpui_kit::init(cx);
        gpui_tokio::init(cx);
        ui::fonts::register(cx);

        if let Err(e) = ThemesManager::init() {
            tracing::error!("Failed to initialize themes: {}", e);
        }

        let db = match smol::block_on(async { app_database::AppDatabase::new().await }) {
            Ok(db) => db,
            Err(e) => {
                tracing::error!("Fatal: failed to init database: {}", e);
                std::process::exit(1);
            }
        };

        let kv = smol::block_on(async { db.load_all_settings().await }).unwrap_or_default();
        let user_settings = smol::block_on(async { db.get_user_settings().await })
            .ok()
            .flatten();

        let mut app_settings = Settings::from_key_values(&kv);
        if app_settings.appearance.theme == Settings::default().appearance.theme
            && let Some(ref u) = user_settings
            && !kv.iter().any(|(k, _)| k == "appearance.theme")
        {
            app_settings.appearance.theme = u.theme.clone();
        }

        let theme_name = SharedString::from(app_settings.appearance.theme.clone());

        let app_settings_global = AppSettings::new(cx, app_settings);
        cx.set_global(app_settings_global);

        if let Err(err) =
            ThemeRegistry::watch_dir(ThemesManager::themes_directory(), cx, move |cx| {
                if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
                    settings::apply_theme_config(&theme, cx);
                    tracing::info!("Applying theme {}", theme_name);
                }
            })
        {
            tracing::error!("Failed to watch themes directory: {}", err);
        }

        cx.set_global(db);
        cx.set_global(CrawlEngine::new());

        let update_manager = update_manager::UpdateManager::new(cx);
        cx.set_global(update_manager);
        update_manager::UpdateManager::start_polling(cx);

        let bounds = gpui_kit::Bounds::centered(None, size(px(1280.), px(900.)), cx);
        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui_kit::TitlebarOptions {
                title: Some("shouting robin".into()),
                appears_transparent: true,
                traffic_light_position: Some(gpui_kit::Point {
                    x: px(8.0),
                    y: px(6.0),
                }),
            }),
            app_owns_titlebar_drag: true,
            inactive_frame_interval: Some(std::time::Duration::from_micros(33_333)),
            window_decorations: Some(gpui_kit::WindowDecorations::Client),
            window_min_size: Some(size(px(900.), px(600.))),
            focus: true,
            show: true,
            kind: gpui_kit::WindowKind::Normal,
            is_movable: true,
            is_minimizable: true,
            is_resizable: true,
            tabbing_identifier: None,
            display_id: None,
            window_background: gpui_kit::WindowBackgroundAppearance::Opaque,
            app_id: Some("se.zanmato.shoutingrobin".into()),
            icon: None,
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let app_entity = cx.new(|cx| ShoutingRobinApp::new(window, cx));
                cx.new(|cx| gpui_kit::component::Root::new(app_entity, window, cx))
            })?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();

        cx.activate(true);
    });
}
