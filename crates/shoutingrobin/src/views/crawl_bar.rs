use gpui::{
    AppContext, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Sizable as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    switch::Switch,
};

use crate::crawl::{CrawlConfig, RenderMode};
use crate::ui::icon::Icon;

#[derive(Clone, Debug)]
// Start inherently carries the full crawl config; Stop is empty. The size
// gap is expected for a low-frequency event emitted once per crawl.
#[allow(clippy::large_enum_variant)]
pub enum CrawlBarEvent {
    Start {
        url: String,
        mode: RenderMode,
        config: CrawlConfig,
    },
    Stop,
}

pub struct CrawlBar {
    focus_handle: gpui::FocusHandle,
    pub url_input: gpui::Entity<InputState>,
    pub running: bool,
    pub has_results: bool,
    default_mode: RenderMode,
    advanced_open: bool,
    headers_input: gpui::Entity<InputState>,
    include_input: gpui::Entity<InputState>,
    exclude_input: gpui::Entity<InputState>,
    crawl_subdomains: bool,
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

        let headers_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Authorization: Bearer token  (one Name: Value per line)")
                .auto_grow(3, 8)
        });
        let include_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("/blog/.*  (one regex per line)")
                .auto_grow(3, 8)
        });
        let exclude_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("/tag/.*  (one regex per line)")
                .auto_grow(3, 8)
        });
        let list_urls_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("https://example.com/page1\nhttps://example.com/page2")
                .auto_grow(4, 10)
        });

        Self {
            focus_handle: cx.focus_handle(),
            url_input,
            running: false,
            has_results: false,
            default_mode: RenderMode::Http,
            advanced_open: false,
            headers_input,
            include_input,
            exclude_input,
            crawl_subdomains: false,
            list_mode: false,
            list_urls_input,
            _subscriptions: vec![input_sub],
        }
    }

    fn build_config(&self, cx: &Context<Self>) -> CrawlConfig {
        let user_agent = {
            let val = crate::app_settings::AppSettings::global(cx)
                .settings
                .crawl
                .user_agent
                .trim()
                .to_string();
            if val.is_empty() { None } else { Some(val) }
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
        let extra_headers = parse_lines(&self.headers_input)
            .into_iter()
            .filter_map(|line| {
                let (name, value) = line.split_once(':')?;
                let name = name.trim();
                if name.is_empty() {
                    return None;
                }
                Some((name.to_string(), value.trim().to_string()))
            })
            .collect();
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
            follow_sitemaps: true,
            block_images: false,
            near_duplicate_threshold: 90,
            content_selector: String::new(),
            user_agent,
            extra_headers,
            include_patterns,
            exclude_patterns,
            crawl_subdomains: self.crawl_subdomains,
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
        let default_mode = self.default_mode;
        let advanced_open = self.advanced_open;
        let list_mode = self.list_mode;
        let crawl_subdomains = self.crawl_subdomains;

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
                        .icon(Icon::Settings)
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
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Switch::new("mode-switch")
                                .small()
                                .checked(matches!(default_mode, RenderMode::Chrome))
                                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                    this.default_mode = if *checked {
                                        RenderMode::Chrome
                                    } else {
                                        RenderMode::Http
                                    };
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_xs()
                                .pl_1()
                                .pr_2()
                                .text_color(if matches!(default_mode, RenderMode::Chrome) {
                                    cx.theme().foreground
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child("Chrome"),
                        )
                        .child(
                            Button::new("crawl-btn")
                                .small()
                                .primary()
                                .icon(Icon::Play)
                                .tooltip("Crawl")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.start_crawl(default_mode, cx);
                                })),
                        ),
                )
            })
            .when(running, |el| {
                el.child(
                    Button::new("stop")
                        .danger()
                        .small()
                        .icon(Icon::Stop)
                        .tooltip("Stop")
                        .on_click(cx.listener(Self::on_stop)),
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
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Subdomains"),
                            )
                            .child(
                                Button::new("subdomains-toggle")
                                    .xsmall()
                                    .when(crawl_subdomains, |b| b.primary())
                                    .when(!crawl_subdomains, |b| b.ghost())
                                    .label(if crawl_subdomains { "On" } else { "Off" })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.crawl_subdomains = !this.crawl_subdomains;
                                        cx.notify();
                                    })),
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
                                    .child("Custom Headers"),
                            )
                            .child(Input::new(&self.headers_input).small()),
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
            .child(main_row)
            .when(advanced_open && !running, |el| el.child(advanced_panel))
    }
}
