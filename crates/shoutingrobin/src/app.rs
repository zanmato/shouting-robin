use std::sync::{Arc, atomic::AtomicBool};

use flume::Receiver;
use gpui::prelude::FluentBuilder as _;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    Menu, MenuItem, ParentElement, Render, ScrollHandle, SharedString, StatefulInteractiveElement,
    Styled, Subscription, Window, actions, div, px, svg,
};
use gpui_component::{
    ActiveTheme, Icon as UiIcon, Root, Sizable as _, TitleBar, WindowExt,
    button::{Button, ButtonVariants as _},
    global_state::GlobalState,
    h_flex,
    menu::AppMenuBar,
    notification::{Notification, NotificationType},
    v_flex,
};
use shoutingrobin_ui::{Tab, TabBar};

use crate::ui::resizable::{ResizableState, h_resizable, resizable_panel};

use crate::crawl::{CrawlEngine, CrawlEvent, RenderMode};
use crate::settings::view::SettingsView;
use crate::update_manager::UpdateManager;
use crate::views::{
    CrawlBar, CrawlsSidebar, DetailsPanel, ResultTab, ResultsGrid, StatusBar,
    crawl_bar::CrawlBarEvent,
    crawls_sidebar::CrawlsSidebarEvent,
    results_grid::{IssueFilter, ResultsGridEvent, filters_for_tab},
};

actions!(shoutingrobin_app, [Quit, OpenSettings]);

/// Corner radius of the sidebar and main cards. Deliberately separate from
/// `theme.radius`, which stays smaller for the controls inside the cards.
pub(crate) const PANEL_RADIUS: gpui::Pixels = px(8.);

/// Gutter between the cards and the window edges. Half of it sits on each side
/// of the split, so the gap between the two cards is the same as the outer one.
const PANEL_GAP: gpui::Pixels = px(6.);

/// Padding inside a segmented trough, around its segments.
const SEGMENT_TROUGH_PADDING: gpui::Pixels = px(3.);

/// Vertical padding inside a single segment of a segmented trough. Content that
/// replaces a trough in the same row carries this plus `SEGMENT_TROUGH_PADDING`,
/// so the row keeps its height and the layout below it doesn't shift.
const SEGMENT_PADDING_Y: gpui::Pixels = px(2.);

pub struct ShoutingRobinApp {
    focus_handle: FocusHandle,
    crawl_bar: Entity<CrawlBar>,
    app_menu_bar: Entity<AppMenuBar>,
    sidebar: Entity<CrawlsSidebar>,
    sidebar_state: Entity<ResizableState>,
    results_grid: Entity<ResultsGrid>,
    details_panel: Entity<DetailsPanel>,
    status_bar: Entity<StatusBar>,
    active_tab: ResultTab,
    /// The render mode of the crawl currently on screen. Accessibility and
    /// Core Web Vitals only exist for a crawl that ran a browser, so the tabs
    /// and panel sections reporting them are hidden for one that did not.
    active_render_mode: RenderMode,
    issue_filter: IssueFilter,
    tabbar_scroll_handle: ScrollHandle,
    _cancel: Option<Arc<AtomicBool>>,
    _subscriptions: Vec<Subscription>,
}

impl ShoutingRobinApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        init_keys(cx);
        init_menus(cx);

        let crawl_bar = cx.new(|cx| CrawlBar::new(window, cx));
        let app_menu_bar = AppMenuBar::new(cx);
        let results_grid = cx.new(|cx| ResultsGrid::new(window, cx));
        let details_panel = cx.new(|_| DetailsPanel::new());
        let status_bar = cx.new(|_| StatusBar::new());
        let sidebar = cx.new(CrawlsSidebar::new);
        let sidebar_state = cx.new(|_| ResizableState::default());

        let mut subscriptions = Vec::new();

        let details_for_grid = details_panel.clone();
        let results_for_grid = results_grid.clone();
        let grid_sub = cx.subscribe(
            &results_grid,
            move |this, _grid, event: &ResultsGridEvent, cx| match event {
                ResultsGridEvent::Selected(row_ix) => {
                    let selection = results_for_grid.read(cx).selection_at(*row_ix, cx);
                    details_for_grid.update(cx, |panel, cx| {
                        panel.set_selected(selection, cx);
                    });
                }
                ResultsGridEvent::OverviewDrillDown { tab, filter } => {
                    this.active_tab = *tab;
                    this.issue_filter = *filter;
                    this.results_grid.update(cx, |grid, cx| {
                        grid.switch_tab(*tab, cx);
                        grid.set_issue_filter(*filter, cx);
                    });
                }
            },
        );
        subscriptions.push(grid_sub);

        let status_bar_clone = status_bar.clone();
        let sub = cx.subscribe(
            &crawl_bar,
            move |this, _bar, event: &CrawlBarEvent, cx| match event {
                CrawlBarEvent::Start { url, mode, config } => {
                    this.reset_for_new_crawl(cx);
                    this.start_crawl(url.clone(), *mode, config.clone(), cx);
                }
                CrawlBarEvent::Stop => {
                    this.stop_crawl(cx);
                    status_bar_clone.update(cx, |s, cx| {
                        s.running = false;
                        cx.notify();
                    });
                }
            },
        );
        subscriptions.push(sub);

        let sidebar_results_grid = results_grid.clone();
        let sidebar_crawl_bar = crawl_bar.clone();
        let sidebar_sub = cx.subscribe_in(
            &sidebar,
            window,
            move |this, _sidebar, event: &CrawlsSidebarEvent, window, cx| match event {
                CrawlsSidebarEvent::Recrawl {
                    crawl_id,
                    root_url,
                    render_mode,
                } => {
                    let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
                    let crawl_id = *crawl_id;
                    let root_url = root_url.clone();
                    let render_mode = *render_mode;
                    cx.spawn_in(window, async move |this, cx| {
                        let stored = match crate::storage::load_crawl_config(&pool, crawl_id).await {
                            Ok(config) => config,
                            Err(e) => {
                                tracing::error!(error=%e, crawl_id, "failed to load crawl config");
                                None
                            }
                        };

                        let updated = this.update_in(cx, |this, window, cx| {
                            let had_stored_config = stored.is_some();
                            // Crawls recorded before the config was written back
                            // have nothing to replay, so fall back to whatever the
                            // crawl bar and settings currently say.
                            let config = match stored {
                                Some(config) => config,
                                None => {
                                    let current =
                                        this.crawl_bar.update(cx, |bar, cx| bar.build_config(cx));
                                    Self::resolve_config(current, cx)
                                }
                            };

                            this.crawl_bar.update(cx, |bar, cx| {
                                bar.restore_from_config(
                                    &root_url,
                                    render_mode,
                                    &config,
                                    window,
                                    cx,
                                );
                            });
                            this.reset_for_new_crawl(cx);
                            this.spawn_crawl(root_url.clone(), render_mode, config, cx);

                            if !had_stored_config {
                                window.push_notification(
                                    Notification::new()
                                        .message(
                                            "This crawl predates saved settings, using the current ones",
                                        )
                                        .with_type(NotificationType::Warning),
                                    cx,
                                );
                            }
                        });

                        if let Err(e) = updated {
                            tracing::error!(error=%e, crawl_id, "failed to start recrawl");
                        }
                    })
                    .detach();
                }
                CrawlsSidebarEvent::Selected {
                    crawl_id,
                    root_url,
                    render_mode,
                } => {
                    this.set_render_mode(*render_mode, cx);
                    let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
                    let crawl_id = *crawl_id;
                    let root_url = root_url.clone();
                    let results_grid = sidebar_results_grid.clone();
                    let crawl_bar = sidebar_crawl_bar.clone();
                    cx.spawn(async move |_, cx| {
                        let pages =
                            crate::storage::load_pages_for_crawl(&pool, crawl_id, &root_url).await;
                        let pages = match pages {
                            Ok(pages) => pages,
                            Err(e) => {
                                tracing::error!(error=%e, "failed to load pages for crawl");
                                return;
                            }
                        };
                        tracing::info!(count = pages.len(), "loaded pages for crawl");

                        let baseline = match crate::storage::find_previous_crawl(
                            &pool,
                            &root_url,
                            Some(crawl_id),
                        )
                        .await
                        {
                            Ok(Some(previous)) => match crate::storage::load_pages_for_crawl(
                                &pool,
                                previous.id,
                                &previous.root_url,
                            )
                            .await
                            {
                                Ok(baseline_pages) => Some((baseline_pages, previous.started_at)),
                                Err(e) => {
                                    tracing::error!(error=%e, "failed to load baseline pages");
                                    None
                                }
                            },
                            Ok(None) => None,
                            Err(e) => {
                                tracing::error!(error=%e, "failed to find previous crawl");
                                None
                            }
                        };

                        cx.update(|cx| {
                            results_grid.update(cx, |g, cx| {
                                g.clear(cx);
                                g.set_root_url(&root_url, cx);
                                for record in pages {
                                    g.push(record, cx);
                                }
                                match baseline {
                                    Some((baseline_pages, started_at)) => {
                                        g.set_baseline(baseline_pages, started_at, cx)
                                    }
                                    None => g.clear_baseline(cx),
                                }
                            });
                            crawl_bar.update(cx, |bar, cx| {
                                bar.has_results = true;
                                cx.notify();
                            });
                        });
                    })
                    .detach();
                }
                CrawlsSidebarEvent::Deleted {
                    crawl_id,
                    was_selected,
                } => {
                    let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
                    let crawl_id = *crawl_id;
                    let was_selected = *was_selected;
                    let results_grid = sidebar_results_grid.clone();
                    cx.spawn(async move |this, cx| {
                        if let Err(e) = crate::storage::delete_crawl(&pool, crawl_id).await {
                            tracing::error!(error=%e, crawl_id, "failed to delete crawl");
                        }
                        cx.update(|cx| {
                            if was_selected {
                                results_grid.update(cx, |g, cx| g.clear(cx));
                            }
                            if let Some(app) = this.upgrade() {
                                app.update(cx, |app, cx| {
                                    if was_selected {
                                        app.crawl_bar.update(cx, |bar, cx| {
                                            bar.has_results = false;
                                            cx.notify();
                                        });
                                    }
                                    app.load_crawl_history(cx);
                                });
                            }
                        });
                    })
                    .detach();
                }
            },
        );
        subscriptions.push(sidebar_sub);

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        // Show a success notification if we relaunched into a freshly applied update.
        if UpdateManager::global(cx)
            .just_updated_from
            .read(cx)
            .is_some()
        {
            let current = env!("CARGO_PKG_VERSION");
            let app_entity = cx.entity().downgrade();
            window.defer(cx, move |window, cx| {
                if let Some(app_entity) = app_entity.upgrade() {
                    window.push_notification(
                        Notification::new()
                            .message(format!("Updated to v{}, click to view changelog", current))
                            .with_type(NotificationType::Success)
                            .on_click(window.listener_for(&app_entity, move |_, _, _, cx| {
                                let changelog_url =
                                    UpdateManager::changelog_url(&format!("v{}", current));
                                cx.open_url(&changelog_url);
                            })),
                        cx,
                    );
                }
            });
        }

        let mut app = Self {
            focus_handle,
            crawl_bar,
            app_menu_bar,
            sidebar,
            sidebar_state,
            results_grid,
            details_panel,
            status_bar,
            active_tab: ResultTab::Internal,
            // Nothing is loaded yet. Chrome is the permissive starting point:
            // it hides no tab, and the first crawl or reopened crawl replaces
            // it with the truth.
            active_render_mode: RenderMode::Chrome,
            issue_filter: IssueFilter::All,
            tabbar_scroll_handle: ScrollHandle::default(),
            _cancel: None,
            _subscriptions: subscriptions,
        };

        app.load_crawl_history(cx);
        app
    }

    fn load_crawl_history(&mut self, cx: &mut Context<Self>) {
        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        let sidebar = self.sidebar.clone();
        cx.spawn(async move |_, cx| {
            let crawls = crate::storage::list_crawls(&pool).await;
            match crawls {
                Ok(crawls) => {
                    tracing::info!(count = crawls.len(), "loaded crawl history");
                    cx.update(|cx| {
                        sidebar.update(cx, |sidebar, cx| sidebar.set_crawls(crawls, cx));
                    });
                }
                Err(e) => {
                    tracing::error!(error=%e, "failed to load crawl history");
                }
            }
        })
        .detach();
    }

    /// Starts a crawl from the crawl bar, filling in the fields the bar leaves
    /// unset from the app settings.
    fn start_crawl(
        &mut self,
        url: String,
        mode: RenderMode,
        config: crate::crawl::CrawlConfig,
        cx: &mut Context<Self>,
    ) {
        let config = Self::resolve_config(config, cx);
        self.spawn_crawl(url, mode, config, cx);
    }

    /// Fills in the config fields the crawl bar leaves at their defaults from the
    /// app settings. A recrawl skips this: it replays the config the earlier
    /// crawl was recorded with, which is already fully resolved.
    fn resolve_config(
        config: crate::crawl::CrawlConfig,
        cx: &Context<Self>,
    ) -> crate::crawl::CrawlConfig {
        let crawl_settings = &crate::app_settings::AppSettings::global(cx).settings.crawl;
        crate::crawl::CrawlConfig {
            max_pages: if config.max_pages > 0 {
                config.max_pages
            } else {
                crawl_settings.max_pages
            },
            max_concurrent: if config.max_concurrent > 0 {
                config.max_concurrent
            } else {
                crawl_settings.max_concurrent
            },
            delay_ms: if config.delay_ms > 0 {
                config.delay_ms
            } else {
                crawl_settings.delay_ms as u64
            },
            timeout_seconds: if config.timeout_seconds > 0 {
                config.timeout_seconds
            } else {
                crawl_settings.timeout_seconds
            },
            respect_robots_txt: crawl_settings.respect_robots_txt,
            follow_sitemaps: crawl_settings.follow_sitemaps,
            block_images: crawl_settings.block_images,
            near_duplicate_threshold: crawl_settings.near_duplicate_threshold,
            content_selector: crawl_settings.content_selector.clone(),
            user_agent: config.user_agent.clone().or_else(|| {
                let ua = crawl_settings.user_agent.trim();
                if ua.is_empty() {
                    None
                } else {
                    Some(ua.to_string())
                }
            }),
            ..config
        }
    }

    /// Records which mode the crawl on screen ran in, and steps off a tab that
    /// mode has nothing to say about.
    fn set_render_mode(&mut self, mode: RenderMode, cx: &mut Context<Self>) {
        if self.active_render_mode == mode {
            return;
        }
        self.active_render_mode = mode;
        self.details_panel
            .update(cx, |panel, cx| panel.set_render_mode(mode, cx));
        if !mode.renders_javascript() && Self::needs_rendering(self.active_tab) {
            self.active_tab = ResultTab::Overview;
            self.issue_filter = IssueFilter::All;
            self.results_grid.update(cx, |grid, cx| {
                grid.switch_tab(ResultTab::Overview, cx);
            });
        }
        cx.notify();
    }

    /// True for the tabs whose every column is measured in a browser. Without
    /// JavaScript rendering they can only ever be empty, so they are hidden
    /// rather than left to look broken.
    fn needs_rendering(tab: ResultTab) -> bool {
        matches!(tab, ResultTab::Accessibility | ResultTab::Performance)
    }

    /// Hands a fully resolved config to the engine and starts pumping its events
    /// into the UI.
    fn spawn_crawl(
        &mut self,
        url: String,
        mode: RenderMode,
        config: crate::crawl::CrawlConfig,
        cx: &mut Context<Self>,
    ) {
        self.set_render_mode(mode, cx);
        let (tx, rx) = crate::crawl::engine::channel();
        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        let (cancel, fut) = {
            let engine = cx.global_mut::<CrawlEngine>();
            engine.start(url, tx, pool, mode, config)
        };
        gpui_tokio::Tokio::spawn(cx, fut).detach();
        self._cancel = Some(cancel);
        self.spawn_event_pump(rx, cx);
    }

    /// Resets the results, status bar and crawl bar for a crawl that is about to
    /// start. Shared by the crawl bar's own Start and by Recrawl.
    fn reset_for_new_crawl(&mut self, cx: &mut Context<Self>) {
        // Recrawl can be triggered from the context menu while a crawl is still
        // in flight, and two crawls writing at once would interleave in the grid.
        // A no-op when nothing is running.
        self.stop_crawl(cx);
        self.results_grid.update(cx, |grid, cx| grid.clear(cx));
        self.crawl_bar.update(cx, |bar, cx| {
            bar.has_results = false;
            cx.notify();
        });
        self.status_bar.update(cx, |status, cx| {
            status.running = true;
            status.crawled = 0;
            status.errors = 0;
            status.queued = 0;
            cx.notify();
        });
    }

    fn stop_crawl(&mut self, cx: &mut Context<Self>) {
        cx.global_mut::<CrawlEngine>().stop();
        self._cancel = None;
    }

    /// Loads the previous crawl of `root_url` (if any) and installs it as the
    /// comparison baseline on the results grid, or clears the baseline when no
    /// earlier crawl exists.
    /// Re-reads the finished crawl's pages from the database and swaps them
    /// into the grid. The records streamed during the crawl predate the
    /// post-crawl passes (link aggregation, PageRank, near-duplicate detection,
    /// hreflang validation), which write their results straight to the
    /// database. Without this the live session shows empty link scores,
    /// similarity and hreflang issues until the crawl is reopened from history.
    fn reload_finished_crawl(&mut self, crawl_id: i64, root_url: String, cx: &mut Context<Self>) {
        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        let results_grid = self.results_grid.clone();
        cx.spawn(async move |_, cx| {
            let pages = match crate::storage::load_pages_for_crawl(&pool, crawl_id, &root_url).await
            {
                Ok(pages) => pages,
                Err(e) => {
                    tracing::error!(error=%e, crawl_id, "failed to reload pages after crawl");
                    return;
                }
            };
            tracing::info!(count = pages.len(), crawl_id, "reloaded pages after crawl");
            cx.update(|cx| {
                results_grid.update(cx, |g, cx| {
                    g.replace_records(pages, cx);
                });
            });
        })
        .detach();
    }

    fn apply_baseline(
        &mut self,
        root_url: String,
        current_crawl_id: Option<i64>,
        cx: &mut Context<Self>,
    ) {
        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        let results_grid = self.results_grid.clone();
        cx.spawn(async move |_, cx| {
            let baseline =
                match crate::storage::find_previous_crawl(&pool, &root_url, current_crawl_id).await
                {
                    Ok(Some(previous)) => match crate::storage::load_pages_for_crawl(
                        &pool,
                        previous.id,
                        &previous.root_url,
                    )
                    .await
                    {
                        Ok(baseline_pages) => Some((baseline_pages, previous.started_at)),
                        Err(e) => {
                            tracing::error!(error=%e, "failed to load baseline pages");
                            None
                        }
                    },
                    Ok(None) => None,
                    Err(e) => {
                        tracing::error!(error=%e, "failed to find previous crawl");
                        None
                    }
                };
            cx.update(|cx| {
                results_grid.update(cx, |g, cx| match baseline {
                    Some((baseline_pages, started_at)) => {
                        g.set_baseline(baseline_pages, started_at, cx)
                    }
                    None => g.clear_baseline(cx),
                });
            });
        })
        .detach();
    }

    fn export_csv(&mut self, cx: &mut Context<Self>) {
        let csv_result = self.results_grid.read(cx).export_csv(cx);
        let csv_content = match csv_result {
            Ok(content) => content,
            Err(e) => {
                tracing::error!(error=%e, "failed to generate CSV");
                return;
            }
        };

        let tab_name = self.active_tab.label();
        let hostname_part = self
            .results_grid
            .read(cx)
            .root_url(cx)
            .and_then(|url| url::Url::parse(&url).ok())
            .and_then(|parsed| parsed.host_str().map(|h| h.to_owned()))
            .unwrap_or_else(|| "unknown".to_owned())
            .replace('.', "-");
        let date_part = chrono::Local::now().format("%Y-%m-%d").to_string();
        let filename = format!(
            "{}-{}-{}.csv",
            hostname_part,
            tab_name.to_lowercase().replace(' ', "-"),
            date_part
        );
        let dir = dirs::download_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let path = cx.prompt_for_new_path(&dir, Some(&filename));

        cx.spawn(async move |_, cx| {
            let file_path = match path.await {
                Ok(Ok(Some(p))) => p,
                Ok(Ok(None)) => return,
                Ok(Err(e)) => {
                    tracing::error!(error=%e, "file dialog error");
                    return;
                }
                Err(_) => return,
            };
            cx.update(
                |_: &mut App| match std::fs::write(&file_path, &csv_content) {
                    Ok(()) => tracing::info!(path = %file_path.display(), "CSV exported"),
                    Err(e) => {
                        tracing::error!(error=%e, path=%file_path.display(), "failed to write CSV")
                    }
                },
            );
        })
        .detach();
    }

    fn spawn_event_pump(&mut self, rx: Receiver<CrawlEvent>, cx: &mut Context<Self>) {
        let results_grid = self.results_grid.clone();
        let status_bar = self.status_bar.clone();
        cx.spawn(async move |this, cx| {
            tracing::info!("UI event pump started");
            while let Ok(event) = rx.recv_async().await {
                match &event {
                    CrawlEvent::Started { .. } => {
                        tracing::info!("crawl started event received");
                    }
                    CrawlEvent::Page(record) => {
                        tracing::info!(url = %record.url, "UI received page event");
                    }
                    CrawlEvent::Finished { total, .. } => {
                        tracing::info!(total, "crawl finished event received");
                    }
                    _ => {}
                }
                cx.update(|cx| match event {
                    CrawlEvent::Started { crawl_id, root_url } => {
                        results_grid.update(cx, |g, cx| {
                            g.set_root_url(root_url.as_str(), cx);
                        });
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |this, cx| {
                                this.load_crawl_history(cx);
                                this.sidebar.update(cx, |sidebar, cx| {
                                    sidebar.set_selected_id(crawl_id, cx);
                                });
                            });
                        }
                    }
                    CrawlEvent::Page(boxed_record) => {
                        let record = *boxed_record;
                        results_grid.update(cx, |g, cx| {
                            g.push(record, cx);
                        });
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |app, cx| {
                                app.crawl_bar.update(cx, |bar, cx| {
                                    if !bar.has_results {
                                        bar.has_results = true;
                                        cx.notify();
                                    }
                                });
                            });
                        }
                        status_bar.update(cx, |s, cx| {
                            s.crawled = s.crawled.saturating_add(1);
                            cx.notify();
                        });
                    }
                    CrawlEvent::Progress { crawled, queued } => {
                        status_bar.update(cx, |s, cx| {
                            s.crawled = crawled;
                            s.queued = queued;
                            cx.notify();
                        });
                    }
                    CrawlEvent::Error { url, message } => {
                        tracing::warn!(%url, %message, "crawl error");
                        status_bar.update(cx, |s, cx| {
                            s.errors = s.errors.saturating_add(1);
                            cx.notify();
                        });
                    }
                    CrawlEvent::Finished { crawl_id, total } => {
                        tracing::info!(total, "crawl finished");
                        status_bar.update(cx, |s, cx| {
                            s.running = false;
                            cx.notify();
                        });
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |this, cx| {
                                this.crawl_bar.update(cx, |bar, cx| {
                                    bar.running = false;
                                    cx.notify();
                                });
                                this.load_crawl_history(cx);
                                if let Some(root_url) = this.results_grid.read(cx).root_url(cx) {
                                    this.reload_finished_crawl(crawl_id, root_url.clone(), cx);
                                    this.apply_baseline(root_url, Some(crawl_id), cx);
                                }
                            });
                        }
                    }
                });
            }
            tracing::info!("UI event pump ended (channel closed)");
        })
        .detach();
    }

    fn on_quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn on_settings(&mut self, _: &OpenSettings, window: &mut Window, cx: &mut Context<Self>) {
        let settings_view = cx.new(SettingsView::new);
        window.open_dialog(cx, move |dialog, _window, _cx| {
            dialog.title("Settings").w(gpui::px(960.)).child(
                div()
                    .id("settings-body")
                    .h(gpui::px(600.))
                    .overflow_y_scroll()
                    .child(settings_view.clone()),
            )
        });
    }
}

impl ShoutingRobinApp {
    fn render_update_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let has_update = UpdateManager::global(cx).pending_update.read(cx).is_some();

        if has_update {
            div().child(
                Button::new("update-available")
                    .ghost()
                    .compact()
                    .small()
                    .label("Update available, click to restart")
                    .on_click(|_, window, cx| {
                        if let Err(e) = UpdateManager::apply_pending_update() {
                            window.push_notification(
                                Notification::new()
                                    .message(format!("Failed to apply update: {e}"))
                                    .with_type(NotificationType::Error),
                                cx,
                            );
                        }
                    }),
            )
        } else {
            div()
        }
    }
}

impl Focusable for ShoutingRobinApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for ShoutingRobinApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let bg = cx.theme().background;
        let fg = cx.theme().foreground;

        let has_baseline = self.results_grid.read(cx).has_baseline(cx);

        // The Changes tab only exists while a comparison baseline is active. If
        // the baseline went away (e.g. a new crawl started) while it was open,
        // fall back to the Overview tab so the bar and grid stay in sync.
        if !has_baseline && self.active_tab == ResultTab::Changes {
            self.active_tab = ResultTab::Overview;
            self.issue_filter = IssueFilter::All;
            self.results_grid.update(cx, |grid, cx| {
                grid.switch_tab(ResultTab::Overview, cx);
            });
        }

        let visible_tabs: Vec<ResultTab> = ResultTab::ALL
            .iter()
            .copied()
            .filter(|tab| *tab != ResultTab::Changes || has_baseline)
            .filter(|tab| {
                !Self::needs_rendering(*tab) || self.active_render_mode.renders_javascript()
            })
            .collect();

        let active_ix = visible_tabs
            .iter()
            .position(|t| *t == self.active_tab)
            .unwrap_or(0);

        let mut tab_bar = TabBar::new("results-tabs")
            .segmented()
            .selected_index(active_ix)
            .on_click(cx.listener({
                let visible_tabs = visible_tabs.clone();
                move |this, ix: &usize, _w, cx| {
                    if let Some(tab) = visible_tabs.get(*ix).copied() {
                        this.active_tab = tab;
                        this.issue_filter = IssueFilter::All;
                        this.results_grid.update(cx, |grid, cx| {
                            grid.switch_tab(tab, cx);
                        });
                    }
                }
            }))
            .track_scroll(&self.tabbar_scroll_handle);

        let tab_badges = self.results_grid.update(cx, |grid, cx| grid.tab_badges(cx));

        for tab in &visible_tabs {
            let tab = *tab;
            let mut t = Tab::new().label(tab.label());
            if let Some(icon) = tab.icon() {
                // Prefixes are rendered as a sibling of the padded label, so
                // they need their own inset to line up with it.
                t = t.prefix(
                    h_flex()
                        .items_center()
                        .pl_2()
                        .child(UiIcon::from(icon).xsmall()),
                );
            }
            if let Some(counts) = tab_badges.get(&tab) {
                let (badge_count, tone) = if counts.errors > 0 {
                    (counts.errors, crate::ui::tag::Tone::Err)
                } else if counts.warnings > 0 {
                    (counts.warnings, crate::ui::tag::Tone::Warn)
                } else if matches!(tab, ResultTab::Ecommerce) {
                    (counts.total, crate::ui::tag::Tone::Accent)
                } else {
                    (counts.total, crate::ui::tag::Tone::Neutral)
                };
                if badge_count > 0 {
                    // The suffix is a sibling of the padded label, so the label's
                    // own 12px right padding is the gap on the badge's left. Pull
                    // some of that back and spend it on the right instead, so the
                    // badge isn't flush against the next tab.
                    t = t.suffix(
                        div().flex().items_center().ml(px(-4.)).pr_2().child(
                            crate::ui::tag::tone_tag(tone, cx)
                                .rounded_full()
                                .child(SharedString::from(badge_count.to_string())),
                        ),
                    );
                }
            }
            tab_bar = tab_bar.child(t);
        }

        let tab_filters = filters_for_tab(self.active_tab);
        let show_issue_filter = tab_filters.len() > 1;

        let current_filter = self.issue_filter;
        let has_results = self.results_grid.read(cx).has_results(cx);
        let mut filter_bar = div()
            .id("issue-filter-bar")
            .flex()
            .items_center()
            .justify_between()
            .px(PANEL_GAP)
            .pb(PANEL_GAP)
            .gap_2();

        // `flex_1` + `min_w_0` so this side yields space to the export button
        // instead of overlapping it when the tab list and the baseline note are
        // both long.
        let mut filter_left = h_flex().items_center().gap_2().flex_1().min_w_0();

        let active_filter_counts = self
            .results_grid
            .update(cx, |grid, cx| grid.active_filter_counts(cx));

        if show_issue_filter {
            // Hand-rolled rather than a segmented `TabBar`, so each filter can
            // carry its own count badge. The styling deliberately reuses the
            // tokens `TabVariant::Segmented` uses, so it stays in step with the
            // real tab bar above it when the theme changes.
            let mut trough = h_flex()
                .gap(px(2.))
                .p(SEGMENT_TROUGH_PADDING)
                .rounded(cx.theme().radius)
                .bg(cx.theme().tab_bar_segmented);

            for &filter in tab_filters {
                let is_active = current_filter == filter;
                let count = active_filter_counts
                    .iter()
                    .find(|(f, _)| *f == filter)
                    .map(|(_, c)| *c)
                    .unwrap_or(0);
                let segment = h_flex()
                    .id(SharedString::from(format!("filter-{:?}", filter)))
                    .items_center()
                    .gap_1()
                    .px_2()
                    .py(SEGMENT_PADDING_Y)
                    .rounded(cx.theme().radius)
                    .text_xs()
                    .cursor_pointer()
                    .when(is_active, |el| {
                        el.bg(cx.theme().background)
                            .shadow_sm()
                            .text_color(cx.theme().tab_active_foreground)
                    })
                    .when(!is_active, |el| {
                        el.text_color(cx.theme().tab_foreground)
                            .hover(|el| el.text_color(cx.theme().tab_active_foreground))
                    })
                    .child(SharedString::from(filter.label()))
                    .when(count > 0, |el| {
                        el.child(
                            crate::ui::tag::tone_tag(filter.tone(), cx)
                                .rounded_full()
                                .child(SharedString::from(count.to_string())),
                        )
                    })
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.issue_filter = filter;
                        this.results_grid.update(cx, |grid, cx| {
                            grid.set_issue_filter(filter, cx);
                        });
                    }));
                trough = trough.child(segment);
            }

            filter_left = filter_left.child(trough);
        } else {
            // No trough on this tab, so carry the trough's and a segment's
            // vertical padding here instead, or the row shrinks and everything
            // below it shifts up when switching tabs.
            filter_left = filter_left.child(
                div()
                    .my(SEGMENT_TROUGH_PADDING)
                    .py(SEGMENT_PADDING_Y)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} results",
                        self.results_grid.read(cx).row_count(cx)
                    )),
            );
        }

        if let Some(note) = self.active_tab.note() {
            let note = SharedString::from(note);
            filter_left = filter_left.child(
                h_flex()
                    .id("tab-note")
                    .items_center()
                    .gap_1()
                    .min_w_0()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(UiIcon::from(crate::ui::icon::Icon::Info).xsmall())
                    // The line rarely fits beside the filters, so the truncated
                    // form is the affordance and the tooltip is the text.
                    .child(div().min_w_0().truncate().child(note.clone()))
                    .tooltip(move |window, cx| {
                        gpui_component::tooltip::Tooltip::new(note.clone()).build(window, cx)
                    }),
            );
        }

        if has_baseline && let Some(started_at) = self.results_grid.read(cx).baseline_started_at(cx)
        {
            let now = chrono::Utc::now().timestamp();
            filter_left = filter_left.child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "Comparing to crawl from {}",
                        crate::views::relative_time(now, started_at)
                    )),
            );
        }

        filter_bar = filter_bar.child(filter_left);

        if has_results {
            filter_bar = filter_bar.child(
                Button::new("export-csv")
                    .xsmall()
                    .outline()
                    .flex_shrink_0()
                    .label("Export CSV")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.export_csv(cx);
                    })),
            );
        }

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg)
            .text_color(fg)
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_settings))
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .pr_3()
                                .child(
                                    svg()
                                        .h(px(30.))
                                        .w(px(30. * 7.95))
                                        .text_color(window.text_style().color)
                                        .path("img/shouting-robin.svg"),
                                )
                                .child(self.app_menu_bar.clone()),
                        )
                        .child(self.render_update_button(cx)),
                ),
            )
            .child(self.crawl_bar.clone())
            .child(
                // Inset the cards from the window edges. The top inset is kept
                // minimal so they sit close under the crawl bar.
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .min_h_0()
                    .overflow_hidden()
                    .px(PANEL_GAP)
                    // The status bar centres its text in its own 28px height, so
                    // a full gutter below the cards reads as too much space.
                    .pb(PANEL_GAP / 2.)
                    .pt(px(1.))
                    .child(
                        h_resizable("sidebar-main")
                            .with_state(&self.sidebar_state)
                            // The cards draw their own borders, so the handle
                            // only needs to stay draggable - see the vendored
                            // `ui::resizable`.
                            .invisible_handles()
                            .child(
                                resizable_panel()
                                    .size(px(240.))
                                    .size_range(px(180.)..px(400.))
                                    .child(
                                        div().size_full().pr(PANEL_GAP / 2.).child(
                                            v_flex()
                                                .size_full()
                                                .overflow_hidden()
                                                .bg(cx.theme().sidebar)
                                                .rounded(PANEL_RADIUS)
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .child(self.sidebar.clone()),
                                        ),
                                    ),
                            )
                            .child(
                                resizable_panel().child(
                                    div().size_full().pl(PANEL_GAP / 2.).child(
                                        v_flex()
                                            .flex_1()
                                            // Allow this flex item to shrink below
                                            // its content width, or the tab bar
                                            // never overflows and scrolls.
                                            .min_w_0()
                                            .h_full()
                                            .overflow_hidden()
                                            // The main card shares the shell
                                            // colour, so the 1px border is what
                                            // makes its rounded outline readable.
                                            .bg(cx.theme().background)
                                            .rounded(PANEL_RADIUS)
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            // The segmented trough paints its own
                                            // background, so inset it rather than
                                            // letting it square the top corners.
                                            .child(div().p(PANEL_GAP).min_w_0().child(tab_bar))
                                            .child(filter_bar)
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_1()
                                                    .min_h_0()
                                                    // One border on the shared
                                                    // wrapper, so it runs across
                                                    // the table and the details
                                                    // panel without a seam.
                                                    .border_t_1()
                                                    .border_color(cx.theme().border)
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .min_w_0()
                                                            .child(self.results_grid.clone()),
                                                    )
                                                    // No panel on the Overview:
                                                    // clicking a row there
                                                    // navigates to another tab
                                                    // rather than selecting a
                                                    // URL, so the panel could
                                                    // only ever show the last
                                                    // thing looked at
                                                    // elsewhere.
                                                    .when(
                                                        self.active_tab != ResultTab::Overview,
                                                        |el| el.child(self.details_panel.clone()),
                                                    ),
                                            ),
                                    ),
                                ),
                            ),
                    ),
            )
            .child(self.status_bar.clone())
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

fn build_menu() -> Vec<Menu> {
    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("Settings", OpenSettings),
                MenuItem::Separator,
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", gpui_component::input::Undo),
                MenuItem::action("Redo", gpui_component::input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", gpui_component::input::Cut),
                MenuItem::action("Copy", gpui_component::input::Copy),
                MenuItem::action("Paste", gpui_component::input::Paste),
                MenuItem::separator(),
                MenuItem::action("Select All", gpui_component::input::SelectAll),
            ],
            disabled: false,
        },
    ]
}

fn init_menus(cx: &mut App) {
    cx.set_menus(build_menu());
    let menu = build_menu().into_iter().map(|menu| menu.owned()).collect();
    GlobalState::global_mut(cx).set_app_menus(menu);
}

fn init_keys(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        gpui::KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        gpui::KeyBinding::new("ctrl-q", Quit, None),
        #[cfg(target_os = "macos")]
        gpui::KeyBinding::new("cmd-,", OpenSettings, None),
        #[cfg(not(target_os = "macos"))]
        gpui::KeyBinding::new("ctrl-,", OpenSettings, None),
    ]);
}
