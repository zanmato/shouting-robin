use gpui::{
    Action, Anchor, AppContext, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_component::{
    ActiveTheme, Sizable as _,
    button::{Button, ButtonVariants as _, DropdownButton},
    input::{Input, InputEvent, InputState},
};
use serde::Deserialize;

use crate::crawl::{CrawlConfig, RenderMode};

#[derive(Clone, Debug)]
pub enum CrawlBarEvent {
    Start {
        url: String,
        mode: RenderMode,
        config: CrawlConfig,
    },
    Stop,
    ExportCsv,
}

#[derive(Clone, Action, PartialEq, Eq, Deserialize)]
#[action(namespace = crawl_bar, no_json)]
struct CrawlHttp;

#[derive(Clone, Action, PartialEq, Eq, Deserialize)]
#[action(namespace = crawl_bar, no_json)]
struct CrawlChrome;

pub struct CrawlBar {
    focus_handle: gpui::FocusHandle,
    pub url_input: gpui::Entity<InputState>,
    pub running: bool,
    pub has_results: bool,
    default_mode: RenderMode,
    advanced_open: bool,
    user_agent_input: gpui::Entity<InputState>,
    include_input: gpui::Entity<InputState>,
    exclude_input: gpui::Entity<InputState>,
    list_mode: bool,
    list_urls_input: gpui::Entity<InputState>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl CrawlBar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("https://books.toscrape.com")
                .default_value("https://books.toscrape.com")
        });

        let input_sub = cx.subscribe_in(
            &url_input,
            window,
            |this, _state, event: &InputEvent, _window, cx| {
                if let InputEvent::PressEnter { .. } = event
                    && !this.running
                {
                    this.start_crawl(this.default_mode, cx);
                }
            },
        );

        let user_agent_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Default User-Agent"));
        let include_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("/blog/.*  (one regex per line)"));
        let exclude_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("/tag/.*  (one regex per line)"));
        let list_urls_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("https://example.com/page1\nhttps://example.com/page2")
        });

        Self {
            focus_handle: cx.focus_handle(),
            url_input,
            running: false,
            has_results: false,
            default_mode: RenderMode::Http,
            advanced_open: false,
            user_agent_input,
            include_input,
            exclude_input,
            list_mode: false,
            list_urls_input,
            _subscriptions: vec![input_sub],
        }
    }

    fn build_config(&self, cx: &Context<Self>) -> CrawlConfig {
        let user_agent = {
            let val = self.user_agent_input.read(cx).value().to_string();
            let trimmed = val.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        };

        let parse_lines = |entity: &gpui::Entity<InputState>| -> Vec<String> {
            entity
                .read(cx)
                .value()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        };

        let include_patterns = parse_lines(&self.include_input);
        let exclude_patterns = parse_lines(&self.exclude_input);
        let seed_urls = if self.list_mode {
            parse_lines(&self.list_urls_input)
        } else {
            Vec::new()
        };

        CrawlConfig {
            max_pages: 0,
            max_concurrent: 0,
            delay_ms: 0,
            timeout_seconds: 30,
            respect_robots_txt: true,
            near_duplicate_threshold: 90,
            content_selector: String::new(),
            user_agent,
            extra_headers: Vec::new(),
            include_patterns,
            exclude_patterns,
            crawl_subdomains: false,
            list_mode: self.list_mode,
            seed_urls,
        }
    }

    fn start_crawl(&mut self, mode: RenderMode, cx: &mut Context<Self>) {
        let url = if self.list_mode {
            let urls = self.list_urls_input.read(cx).value().to_string();
            urls.lines()
                .next()
                .map(|l| l.trim().to_string())
                .unwrap_or_default()
        } else {
            self.url_input.read(cx).value().to_string()
        };
        if url.trim().is_empty() {
            return;
        }
        let config = self.build_config(cx);
        self.default_mode = mode;
        self.running = true;
        cx.emit(CrawlBarEvent::Start { url, mode, config });
        cx.notify();
    }

    fn on_stop(&mut self, _: &gpui::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.running = false;
        cx.emit(CrawlBarEvent::Stop);
        cx.notify();
    }

    fn on_crawl_http(&mut self, _: &CrawlHttp, _: &mut Window, cx: &mut Context<Self>) {
        self.start_crawl(RenderMode::Http, cx);
    }

    fn on_crawl_chrome(&mut self, _: &CrawlChrome, _: &mut Window, cx: &mut Context<Self>) {
        self.start_crawl(RenderMode::Chrome, cx);
    }

    fn default_label(&self) -> SharedString {
        SharedString::from(self.default_mode.label().to_string())
    }
}

impl EventEmitter<CrawlBarEvent> for CrawlBar {}

impl Focusable for CrawlBar {
    fn focus_handle(&self, _: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CrawlBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let running = self.running;
        let has_results = self.has_results;
        let default_label = self.default_label();
        let default_mode = self.default_mode;
        let advanced_open = self.advanced_open;
        let list_mode = self.list_mode;

        let main_row = div()
            .id("crawl-bar-main")
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .when(!running, |el| {
                el.child(
                    Button::new("toggle-advanced")
                        .small()
                        .ghost()
                        .label("Settings")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.advanced_open = !this.advanced_open;
                            cx.notify();
                        })),
                )
            })
            .when(!list_mode, |el| {
                el.child(div().flex_1().child(Input::new(&self.url_input).small()))
            })
            .when(list_mode, |el| {
                el.child(
                    div()
                        .flex_1()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("List Mode: enter URLs below"),
                )
            })
            .when(!running, |el| {
                el.child(
                    DropdownButton::new("crawl-dropdown")
                        .small()
                        .primary()
                        .button(Button::new("crawl-btn").label(default_label).on_click(
                            cx.listener(move |this, _, _, cx| {
                                this.start_crawl(default_mode, cx);
                            }),
                        ))
                        .dropdown_menu_with_anchor(
                            Anchor::BottomRight,
                            move |menu, _window, _cx| {
                                menu.menu_with_check(
                                    "Crawl (HTTP)",
                                    default_mode == RenderMode::Http,
                                    Box::new(CrawlHttp),
                                )
                                .menu_with_check(
                                    "Crawl (Chrome)",
                                    default_mode == RenderMode::Chrome,
                                    Box::new(CrawlChrome),
                                )
                            },
                        ),
                )
            })
            .when(running, |el| {
                el.child(
                    Button::new("stop")
                        .danger()
                        .small()
                        .label("Stop")
                        .on_click(cx.listener(Self::on_stop)),
                )
            })
            .when(has_results && !running, |el| {
                el.child(
                    Button::new("export-csv")
                        .small()
                        .ghost()
                        .label("Export CSV")
                        .on_click(cx.listener(|this, _, _, cx| {
                            cx.emit(CrawlBarEvent::ExportCsv);
                        })),
                )
            });

        let advanced_panel = div()
            .id("crawl-bar-advanced")
            .px_3()
            .pb_2()
            .pt_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_wrap()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .w(px(200.))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("User-Agent"),
                            )
                            .child(Input::new(&self.user_agent_input).small()),
                    )
                    .child(
                        div().flex().flex_col().gap_1().child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("List Mode"),
                                )
                                .child(
                                    Button::new("list-mode-toggle")
                                        .xsmall()
                                        .when(list_mode, |b| b.primary())
                                        .when(!list_mode, |b| b.ghost())
                                        .label(if list_mode { "On" } else { "Off" })
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.list_mode = !this.list_mode;
                                            cx.notify();
                                        })),
                                ),
                        ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .w(px(200.))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Include (regex)"),
                            )
                            .child(Input::new(&self.include_input).small()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .w(px(200.))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Exclude (regex)"),
                            )
                            .child(Input::new(&self.exclude_input).small()),
                    ),
            )
            .when(list_mode, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .mt_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("URLs to crawl (one per line)"),
                        )
                        .child(
                            div()
                                .w_full()
                                .h(px(80.))
                                .child(Input::new(&self.list_urls_input).small()),
                        ),
                )
            });

        div()
            .id("crawl-bar")
            .flex()
            .flex_col()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .on_action(cx.listener(Self::on_crawl_http))
            .on_action(cx.listener(Self::on_crawl_chrome))
            .child(main_row)
            .when(advanced_open && !running, |el| el.child(advanced_panel))
    }
}
