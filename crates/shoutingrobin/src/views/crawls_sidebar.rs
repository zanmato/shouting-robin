use gpui::prelude::FluentBuilder;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use gpui_component::hover_card::HoverCard;
use gpui_component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_component::{ActiveTheme, IconName, StyledExt as _, v_flex};

use crate::crawl::RenderMode;
use crate::storage::CrawlRow;
use crate::ui::tag::{Tone, tone_tag};

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
                .child(SharedString::from(format!("{count}"))),
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

pub struct CrawlsSidebar {
    focus_handle: FocusHandle,
    crawls: Vec<CrawlRow>,
    selected_id: Option<i64>,
    handle: Entity<Self>,
}

impl CrawlsSidebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            crawls: Vec::new(),
            selected_id: None,
            handle: cx.entity(),
        }
    }

    pub fn set_crawls(&mut self, crawls: Vec<CrawlRow>, cx: &mut Context<Self>) {
        self.crawls = crawls;
        cx.notify();
    }

    pub fn set_selected_id(&mut self, id: i64, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
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
                        let mode_tag = div()
                            .text_xs()
                            .text_color(theme.muted_foreground)
                            .child(SharedString::from(mode_text))
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
                            let total = crawl.page_count;
                            HoverCard::new(("crawl-count", crawl.id as usize))
                                .trigger(
                                    tone_tag(Tone::Neutral, cx)
                                        .rounded_full()
                                        .child(SharedString::from(format!("{total}"))),
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
                                                    "{total} URLs recorded"
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
                            .mx(gpui::px(4.))
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
