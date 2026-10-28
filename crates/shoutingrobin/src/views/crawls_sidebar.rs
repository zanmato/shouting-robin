use gpui_kit::component::hover_card::HoverCard;
use gpui_kit::component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    ActiveTheme, Icon as UiIcon, IconName, Sizable as _, StyledExt as _, v_flex,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, AsyncApp, Context, Entity, EventEmitter, FocusHandle, Focusable, Image, ImageFormat,
    InteractiveElement, IntoElement, ObjectFit, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, StyledImage as _, WeakEntity, Window, div, img, px,
};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::crawl::RenderMode;
use crate::crawl::site_icon::{RETRY_MISSING_AFTER_SECONDS, icon_client, icon_host};
use crate::storage::CrawlRow;
use crate::ui::icon::Icon;
use crate::ui::tag::{Tone, compact_count, group_digits, tone_tag};

/// One "label ..... count" line of the crawl badge's hover card.
fn count_line(label: &'static str, count: i64, cx: &App) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .justify_between()
        .gap_2()
        .text_color(cx.theme().muted_foreground)
        .child(label)
        .child(
            div()
                .text_color(cx.theme().foreground)
                .child(SharedString::from(group_digits(
                    usize::try_from(count).unwrap_or_default(),
                ))),
        )
}

#[derive(Clone, Debug)]
pub enum CrawlsSidebarEvent {
    Selected {
        crawl_id: i64,
        root_url: String,
        render_mode: RenderMode,
    },
    Deleted {
        crawl_id: i64,
        was_selected: bool,
    },
    /// Run the same URL again with the settings the crawl was recorded with.
    Recrawl {
        crawl_id: i64,
        root_url: String,
        render_mode: RenderMode,
    },
}

/// A host's icon as the list knows it.
enum SiteIcon {
    Loading,
    Found(Arc<Image>),
    Missing,
}

pub struct CrawlsSidebar {
    focus_handle: FocusHandle,
    crawls: Vec<CrawlRow>,
    selected_id: Option<i64>,
    handle: Entity<Self>,
    /// Keyed by `icon_host`. A host that is absent has not been looked up.
    site_icons: HashMap<String, SiteIcon>,
}

impl CrawlsSidebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            crawls: Vec::new(),
            selected_id: None,
            handle: cx.entity(),
            site_icons: HashMap::new(),
        }
    }

    pub fn set_crawls(&mut self, crawls: Vec<CrawlRow>, cx: &mut Context<Self>) {
        self.crawls = crawls;
        self.load_site_icons(cx);
        cx.notify();
    }

    /// Looks up the icons of hosts the list has not seen yet: stored ones are
    /// shown, and hosts without one are fetched one at a time. A host whose
    /// crawl is still running is skipped, since the crawl fetches its icon and
    /// the list reloads when it finishes.
    fn load_site_icons(&mut self, cx: &mut Context<Self>) {
        // A running crawl may find the icon an earlier fetch missed, so the
        // host is looked up again once the crawl finishes.
        for crawl in self
            .crawls
            .iter()
            .filter(|crawl| crawl.finished_at.is_none())
        {
            if let Some(host) = icon_host(&crawl.root_url)
                && matches!(self.site_icons.get(&host), Some(SiteIcon::Missing))
            {
                self.site_icons.remove(&host);
            }
        }

        let mut page_urls: HashMap<String, String> = HashMap::new();
        let mut running_hosts: HashSet<String> = HashSet::new();
        for crawl in &self.crawls {
            let Some(host) = icon_host(&crawl.root_url) else {
                continue;
            };
            if self.site_icons.contains_key(&host) {
                continue;
            }
            if crawl.finished_at.is_none() {
                running_hosts.insert(host.clone());
            }
            page_urls
                .entry(host)
                .or_insert_with(|| crawl.root_url.clone());
        }
        if page_urls.is_empty() {
            return;
        }
        for host in page_urls.keys() {
            self.site_icons.insert(host.clone(), SiteIcon::Loading);
        }

        let pool = crate::app_database::AppDatabase::global(cx).pool().clone();
        cx.spawn(async move |sidebar, cx| {
            let stored = match crate::storage::load_site_icons(&pool).await {
                Ok(stored) => stored,
                Err(e) => {
                    tracing::warn!(error=%e, "failed to load site icons");
                    Vec::new()
                }
            };
            let now = chrono::Utc::now().timestamp();
            let mut known: HashMap<String, SiteIcon> = HashMap::new();
            for icon in stored {
                if !page_urls.contains_key(&icon.host) {
                    continue;
                }
                match icon.icon_png {
                    Some(png) => {
                        known.insert(icon.host, SiteIcon::Found(icon_image(png)));
                    }
                    None if now - icon.fetched_at < RETRY_MISSING_AFTER_SECONDS => {
                        known.insert(icon.host, SiteIcon::Missing);
                    }
                    None => {}
                }
            }
            let to_fetch: Vec<(String, String)> = page_urls
                .into_iter()
                .filter(|(host, _)| !known.contains_key(host))
                .collect();
            let update = sidebar.update(cx, |sidebar, cx| {
                for (host, icon) in known {
                    sidebar.site_icons.insert(host, icon);
                }
                for (host, _) in &to_fetch {
                    if running_hosts.contains(host) {
                        sidebar.site_icons.remove(host);
                    }
                }
                cx.notify();
            });
            if update.is_err() {
                return;
            }

            let to_fetch: Vec<(String, String)> = to_fetch
                .into_iter()
                .filter(|(host, _)| !running_hosts.contains(host))
                .collect();
            fetch_site_icons(sidebar, pool, to_fetch, cx).await;
        })
        .detach();
    }

    pub fn set_selected_id(&mut self, id: i64, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    /// Opens a crawl from the list, as clicking it does.
    #[cfg(feature = "screenshots")]
    pub(crate) fn open(&mut self, crawl_id: i64, cx: &mut Context<Self>) -> bool {
        let Some(crawl) = self.crawls.iter().find(|crawl| crawl.id == crawl_id) else {
            return false;
        };
        let event = CrawlsSidebarEvent::Selected {
            crawl_id,
            root_url: crawl.root_url.clone(),
            render_mode: RenderMode::from_stored(&crawl.render_mode),
        };
        self.selected_id = Some(crawl_id);
        cx.emit(event);
        cx.notify();
        true
    }
}

fn icon_image(png: Vec<u8>) -> Arc<Image> {
    Arc::new(Image::from_bytes(ImageFormat::Png, png))
}

/// Fetches icons for crawls recorded before icons were captured, one host at
/// a time so opening the app does not fire a burst of requests.
async fn fetch_site_icons(
    sidebar: WeakEntity<CrawlsSidebar>,
    pool: sqlx::SqlitePool,
    hosts: Vec<(String, String)>,
    cx: &mut AsyncApp,
) {
    if hosts.is_empty() {
        return;
    }
    let client = match icon_client() {
        Ok(client) => client,
        Err(e) => {
            tracing::warn!(error=%e, "failed to build the site icon client");
            return;
        }
    };
    for (host, page_url) in hosts {
        let task = cx.update(|cx| {
            let pool = pool.clone();
            let client = client.clone();
            gpui_tokio::Tokio::spawn_result(cx, async move {
                crate::crawl::site_icon::refresh_site_icon(&pool, &client, &page_url).await
            })
        });
        let icon = match task.await {
            Ok(Some(png)) => SiteIcon::Found(icon_image(png)),
            Ok(None) => SiteIcon::Missing,
            Err(e) => {
                tracing::warn!(%host, error=%e, "failed to fetch site icon");
                SiteIcon::Missing
            }
        };
        let update = sidebar.update(cx, |sidebar, cx| {
            sidebar.site_icons.insert(host, icon);
            cx.notify();
        });
        if update.is_err() {
            return;
        }
    }
}

impl EventEmitter<CrawlsSidebarEvent> for CrawlsSidebar {}

impl Focusable for CrawlsSidebar {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CrawlsSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let now = chrono::Utc::now().timestamp();
        let sidebar = self.handle.clone();

        div()
            .id("crawls-sidebar")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .h_full()
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_bold()
                    .text_color(theme.primary)
                    .child("Historic Crawls"),
            )
            .child(
                div()
                    .id("crawls-list")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .py_1()
                    .children(self.crawls.iter().map(|crawl| {
                        let is_selected = self.selected_id == Some(crawl.id);
                        let relative = super::relative_time(now, crawl.started_at);
                        let running = crawl.finished_at.is_none();
                        let mode_text = if crawl.render_mode == "chrome" {
                            "Chrome"
                        } else {
                            "HTTP"
                        };
                        let layout_capture = crawl.layout_capture;
                        let site_icon = icon_host(&crawl.root_url).and_then(|host| {
                            match self.site_icons.get(&host) {
                                Some(SiteIcon::Found(image)) => Some(image.clone()),
                                Some(SiteIcon::Loading | SiteIcon::Missing) | None => None,
                            }
                        });
                        let icon_slot = div()
                            .flex_shrink_0()
                            .size(px(16.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(theme.muted_foreground)
                            .map(|slot| match site_icon {
                                Some(image) => slot
                                    .child(img(image).size(px(16.)).object_fit(ObjectFit::Contain)),
                                None => slot.child(UiIcon::from(Icon::Globe).small()),
                            });
                        let mode_tag = div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(SharedString::from(mode_text))
                            .when(layout_capture.is_enabled(), |el| {
                                el.child(
                                    div()
                                        .id(("crawl-layout", crawl.id as usize))
                                        .child(UiIcon::from(Icon::LayoutPanelLeft).xsmall())
                                        .tooltip(move |window, cx| {
                                            Tooltip::new(layout_capture.describe())
                                                .build(window, cx)
                                        }),
                                )
                            })
                            .into_any_element();

                        let count_tag = if running {
                            tone_tag(Tone::Warn, cx)
                                .rounded_full()
                                .child(SharedString::from("running"))
                                .into_any_element()
                        } else {
                            // The badge counts every URL the crawl recorded,
                            // which is mostly resources on an asset-heavy site
                            // and so reads far above the page count on any tab.
                            // A hover card rather than a tooltip: the three
                            // parts read as a table of figures, which on one
                            // tooltip line is a sentence nobody finishes.
                            let (documents, others, unfetched) = crawl.breakdown();
                            let total = usize::try_from(crawl.page_count).unwrap_or_default();
                            HoverCard::new(("crawl-count", crawl.id as usize))
                                .trigger(
                                    tone_tag(Tone::Neutral, cx)
                                        .rounded_full()
                                        .child(SharedString::from(compact_count(total))),
                                )
                                .content(move |_, _, cx| {
                                    v_flex()
                                        .gap_1()
                                        .w(px(228.))
                                        .text_xs()
                                        .child(
                                            div()
                                                .font_semibold()
                                                .text_color(cx.theme().foreground)
                                                .child(SharedString::from(format!(
                                                    "{} URLs recorded",
                                                    group_digits(total)
                                                ))),
                                        )
                                        .child(count_line("HTML pages", documents, cx))
                                        .child(count_line("Resources and other URLs", others, cx))
                                        .child(count_line("Discovered, not fetched", unfetched, cx))
                                })
                                .into_any_element()
                        };

                        div()
                            .id(SharedString::from(format!("crawl-{}", crawl.id)))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            // Inset and round the rows so a selected or hovered
                            // first/last row can't square the sidebar card's
                            // corners - GPUI's content mask is rectangular.
                            .mx(gpui_kit::px(4.))
                            .rounded(theme.radius)
                            .px_2()
                            .py_2()
                            .when(is_selected, |el| el.bg(theme.accent))
                            .text_color(theme.foreground)
                            .cursor_pointer()
                            .hover(|el| {
                                if is_selected {
                                    el.bg(theme.accent)
                                } else {
                                    el.bg(theme.muted)
                                }
                            })
                            .on_click(cx.listener({
                                let crawl_id = crawl.id;
                                let crawl_root = crawl.root_url.clone();
                                let crawl_mode = RenderMode::from_stored(&crawl.render_mode);
                                move |this, _event, _window, cx| {
                                    this.selected_id = Some(crawl_id);
                                    cx.emit(CrawlsSidebarEvent::Selected {
                                        crawl_id,
                                        root_url: crawl_root.clone(),
                                        render_mode: crawl_mode,
                                    });
                                    cx.notify();
                                }
                            }))
                            .child(icon_slot)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .min_w_0()
                                    .gap_0p5()
                                    .child(div().text_sm().truncate().child(crawl.root_url.clone()))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(theme.muted_foreground)
                                            .child(relative),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_end()
                                    .gap_1()
                                    .child(count_tag)
                                    .child(mode_tag),
                            )
                            .context_menu({
                                let crawl_id = crawl.id;
                                let was_selected = is_selected;
                                let crawl_root = crawl.root_url.clone();
                                let render_mode = RenderMode::from_stored(&crawl.render_mode);
                                let sidebar = sidebar.clone();
                                move |menu, window, _cx| {
                                    let sidebar_for_delete = sidebar.clone();
                                    let crawl_root = crawl_root.clone();
                                    menu.item(
                                        PopupMenuItem::new("Recrawl")
                                            .icon(crate::ui::icon::Icon::RefreshCw)
                                            .on_click(window.listener_for(
                                                &sidebar,
                                                move |_this, _event, _window, cx| {
                                                    cx.emit(CrawlsSidebarEvent::Recrawl {
                                                        crawl_id,
                                                        root_url: crawl_root.clone(),
                                                        render_mode,
                                                    });
                                                },
                                            )),
                                    )
                                    .separator()
                                    .item(
                                        PopupMenuItem::new("Delete")
                                            .icon(IconName::Delete)
                                            .on_click(window.listener_for(
                                                &sidebar_for_delete,
                                                move |this, _event, _window, cx| {
                                                    this.crawls.retain(|c| c.id != crawl_id);
                                                    if was_selected {
                                                        this.selected_id = None;
                                                    }
                                                    cx.emit(CrawlsSidebarEvent::Deleted {
                                                        crawl_id,
                                                        was_selected,
                                                    });
                                                    cx.notify();
                                                },
                                            )),
                                    )
                                }
                            })
                    })),
            )
    }
}
