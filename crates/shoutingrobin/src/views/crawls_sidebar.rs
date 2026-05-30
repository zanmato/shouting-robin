use gpui::prelude::FluentBuilder;
use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, div,
    transparent_black,
};
use gpui_component::menu::{ContextMenuExt, PopupMenuItem};
use gpui_component::{ActiveTheme, IconName, StyledExt as _};

use crate::storage::CrawlRow;
use crate::ui::tag::{Tone, tone_tag};

#[derive(Clone, Debug)]
pub enum CrawlsSidebarEvent {
    Selected { crawl_id: i64, root_url: String },
    Deleted { crawl_id: i64, was_selected: bool },
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
            .flex()
            .flex_col()
            .h_full()
            .bg(theme.background)
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .border_b_1()
                    .border_color(theme.border)
                    .child("HISTORIC CRAWLS"),
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
                        let relative = relative_time(now, crawl.started_at);
                        let running = crawl.finished_at.is_none();
                        let is_chrome = crawl.render_mode == "chrome";
                        let mode_tag = if is_chrome {
                            tone_tag(Tone::Accent)
                                .rounded_full()
                                .child(SharedString::from("Chrome"))
                                .into_any_element()
                        } else {
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(SharedString::from("HTTP"))
                                .into_any_element()
                        };
                        let count_tag = if running {
                            tone_tag(Tone::Warn)
                                .rounded_full()
                                .child(SharedString::from("running"))
                                .into_any_element()
                        } else {
                            tone_tag(Tone::Neutral)
                                .rounded_full()
                                .child(SharedString::from(format!("{}", crawl.page_count)))
                                .into_any_element()
                        };

                        div()
                            .id(SharedString::from(format!("crawl-{}", crawl.id)))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .px_3()
                            .py_2()
                            .border_l_2()
                            .border_color(if is_selected {
                                theme.primary
                            } else {
                                transparent_black()
                            })
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
                                move |this, _event, _window, cx| {
                                    this.selected_id = Some(crawl_id);
                                    cx.emit(CrawlsSidebarEvent::Selected {
                                        crawl_id,
                                        root_url: crawl_root.clone(),
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
                                let sidebar = sidebar.clone();
                                move |menu, window, _cx| {
                                    menu.item(
                                        PopupMenuItem::new("Delete")
                                            .icon(IconName::Delete)
                                            .on_click(window.listener_for(
                                                &sidebar,
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

fn relative_time(now: i64, ts: i64) -> String {
    let delta = now.saturating_sub(ts);
    if delta < 60 {
        return "just now".into();
    }
    if delta < 3600 {
        return format!("{}m ago", delta / 60);
    }
    if delta < 86_400 {
        return format!("{}h ago", delta / 3600);
    }
    if delta < 7 * 86_400 {
        return format!("{}d ago", delta / 86_400);
    }
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%b %-d").to_string())
        .unwrap_or_else(|| "unknown".into())
}
