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
    menu::AppMenuBar,
    resizable::{ResizableState, h_resizable, resizable_panel},
};
use shoutingrobin_ui::{Tab, TabBar};

use crate::crawl::{CrawlConfig, CrawlEngine, CrawlEvent, RenderMode};
use crate::settings::view::SettingsView;
use crate::views::{
    CrawlBar, CrawlsSidebar, DetailsPanel, ResultTab, ResultsGrid, StatusBar,
    crawl_bar::CrawlBarEvent,
    crawls_sidebar::CrawlsSidebarEvent,
    results_grid::{IssueFilter, ResultsGridEvent, filters_for_tab},
};

actions!(shoutingrobin_app, [Quit, OpenSettings]);

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
                    let record = results_for_grid.read(cx).record_at(*row_ix, cx);
                    details_for_grid.update(cx, |panel, cx| {
                        panel.set_selected(record, cx);
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

        let results_grid_clone = results_grid.clone();
        let status_bar_clone = status_bar.clone();
        let sub = cx.subscribe(
            &crawl_bar,
            move |this, _bar, event: &CrawlBarEvent, cx| match event {
                CrawlBarEvent::Start { url, mode, config } => {
                    results_grid_clone.update(cx, |g, cx| g.clear(cx));
                    this.crawl_bar.update(cx, |bar, cx| {
                        bar.has_results = false;
                        cx.notify();
                    });
                    status_bar_clone.update(cx, |s, cx| {
                        s.running = true;
                        s.crawled = 0;
                        s.errors = 0;
                        s.queued = 0;
                        cx.notify();
                    });
                    this.start_crawl(url.clone(), *mode, config.clone(), cx);
                }
                CrawlBarEvent::Stop => {
                    this.stop_crawl(cx);
                    status_bar_clone.update(cx, |s, cx| {
                        s.running = false;
                        cx.notify();
                    });
                }
                CrawlBarEvent::ExportCsv => {
                    this.export_csv(cx);
                }
            },
        );
        subscriptions.push(sub);

        let sidebar_results_grid = results_grid.clone();
        let sidebar_crawl_bar = crawl_bar.clone();
        let sidebar_sub = cx.subscribe(
            &sidebar,
            move |_this, _sidebar, event: &CrawlsSidebarEvent, cx| match event {
                CrawlsSidebarEvent::Selected { crawl_id, root_url } => {
                    let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
                    let crawl_id = *crawl_id;
                    let root_url = root_url.clone();
                    let results_grid = sidebar_results_grid.clone();
                    let crawl_bar = sidebar_crawl_bar.clone();
                    cx.spawn(async move |_, cx| {
                        let pages =
                            crate::storage::load_pages_for_crawl(&pool, crawl_id, &root_url).await;
                        match pages {
                            Ok(pages) => {
                                tracing::info!(count = pages.len(), "loaded pages for crawl");
                                cx.update(|cx| {
                                    results_grid.update(cx, |g, cx| {
                                        g.clear(cx);
                                        g.set_root_url(&root_url, cx);
                                        for record in pages {
                                            g.push(record, cx);
                                        }
                                    });
                                    crawl_bar.update(cx, |bar, cx| {
                                        bar.has_results = true;
                                        cx.notify();
                                    });
                                });
                            }
                            Err(e) => {
                                tracing::error!(error=%e, "failed to load pages for crawl");
                            }
                        }
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

        let mut app = Self {
            focus_handle: cx.focus_handle(),
            crawl_bar,
            app_menu_bar,
            sidebar,
            sidebar_state,
            results_grid,
            details_panel,
            status_bar,
            active_tab: ResultTab::Internal,
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

    fn start_crawl(
        &mut self,
        url: String,
        mode: RenderMode,
        config: crate::crawl::CrawlConfig,
        cx: &mut Context<Self>,
    ) {
        let (tx, rx) = crate::crawl::engine::channel();
        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        let crawl_settings = &crate::app_settings::AppSettings::global(cx).settings.crawl;
        let config = crate::crawl::CrawlConfig {
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
            near_duplicate_threshold: crawl_settings.near_duplicate_threshold,
            content_selector: crawl_settings.content_selector.clone(),
            ..config
        };
        let (cancel, fut) = {
            let engine = cx.global_mut::<CrawlEngine>();
            engine.start(url, tx, pool, mode, config)
        };
        gpui_tokio::Tokio::spawn(cx, fut).detach();
        self._cancel = Some(cancel);
        self.spawn_event_pump(rx, cx);
    }

    fn stop_crawl(&mut self, cx: &mut Context<Self>) {
        cx.global_mut::<CrawlEngine>().stop();
        self._cancel = None;
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
        let filename = format!(
            "shoutingrobin-{}.csv",
            tab_name.to_lowercase().replace(' ', "-")
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
                    CrawlEvent::Finished { total } => {
                        tracing::info!(total, "crawl finished event received");
                    }
                    _ => {}
                }
                cx.update(|cx| match event {
                    CrawlEvent::Started { root_url } => {
                        results_grid.update(cx, |g, cx| {
                            g.set_root_url(root_url.as_str(), cx);
                        });
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |this, cx| this.load_crawl_history(cx));
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
                    CrawlEvent::Finished { total } => {
                        tracing::info!(total, "crawl finished");
                        status_bar.update(cx, |s, cx| {
                            s.running = false;
                            cx.notify();
                        });
                        if let Some(this) = this.upgrade() {
                            this.update(cx, |this, cx| this.load_crawl_history(cx));
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
            dialog.title("Settings").w(gpui::px(850.)).child(
                div()
                    .id("settings-body")
                    .h(gpui::px(600.))
                    .overflow_y_scroll()
                    .child(settings_view.clone()),
            )
        });
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

        let active_ix = ResultTab::ALL
            .iter()
            .position(|t| *t == self.active_tab)
            .unwrap_or(0);

        let mut tab_bar = TabBar::new("results-tabs")
            .underline()
            .pl_2()
            .selected_index(active_ix)
            .on_click(cx.listener(|this, ix: &usize, _w, cx| {
                if let Some(tab) = ResultTab::ALL.get(*ix).copied() {
                    this.active_tab = tab;
                    this.issue_filter = IssueFilter::All;
                    this.results_grid.update(cx, |grid, cx| {
                        grid.switch_tab(tab, cx);
                    });
                }
            }))
            .track_scroll(&self.tabbar_scroll_handle);

        let tab_counts = self.results_grid.read(cx).tab_counts(cx);

        for tab in ResultTab::ALL {
            let mut t = Tab::new().label(tab.label());
            if let Some(icon) = tab.icon() {
                t = t.prefix(UiIcon::from(icon).xsmall());
            }
            if let Some(counts) = tab_counts.get(tab) {
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
                    t = t.suffix(
                        div().flex().items_center().pl_1().child(
                            crate::ui::tag::tone_tag(tone)
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
        let mut filter_bar = div()
            .id("issue-filter-bar")
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background);

        if show_issue_filter {
            for &filter in tab_filters {
                let is_active = current_filter == filter;
                let count = self.results_grid.read(cx).count_for_filter(filter, cx);
                let mut btn = Button::new(SharedString::from(format!("filter-{:?}", filter)))
                    .label(SharedString::from(filter.label()))
                    .xsmall()
                    .when(is_active, |btn| btn.primary())
                    .when(!is_active, |btn| btn.ghost())
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        this.issue_filter = filter;
                        this.results_grid.update(cx, |grid, cx| {
                            grid.set_issue_filter(filter, cx);
                        });
                    }));
                if count > 0 {
                    btn = btn.child(
                        crate::ui::tag::tone_tag(filter.tone())
                            .rounded_full()
                            .child(SharedString::from(count.to_string())),
                    );
                }
                filter_bar = filter_bar.child(btn);
            }
        } else {
            filter_bar = filter_bar.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{} results",
                        self.results_grid.read(cx).row_count(cx)
                    )),
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
                        ),
                ),
            )
            .child(self.crawl_bar.clone())
            .child(
                div().flex().flex_1().min_h_0().child(
                    h_resizable("sidebar-main")
                        .with_state(&self.sidebar_state)
                        .child(
                            resizable_panel()
                                .size(px(240.))
                                .size_range(px(180.)..px(400.))
                                .child(
                                    div()
                                        .size_full()
                                        .overflow_hidden()
                                        .border_r_1()
                                        .border_color(cx.theme().border)
                                        .child(self.sidebar.clone()),
                                ),
                        )
                        .child(
                            resizable_panel().child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .size_full()
                                    .overflow_hidden()
                                    .child(tab_bar)
                                    .child(filter_bar)
                                    .child(
                                        div()
                                            .flex()
                                            .flex_1()
                                            .min_h_0()
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .child(self.results_grid.clone()),
                                            )
                                            .child(self.details_panel.clone()),
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
