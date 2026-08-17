use std::collections::HashMap;
use std::time::Duration;

use gpui::{
    AppContext, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Task, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Sizable as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState, NumberInput, Textarea, TextareaState},
    switch::Switch,
};

use crate::app_database::AppDatabase;
use crate::app_settings::AppSettings;
use crate::crawl::{CrawlConfig, RenderMode};
use crate::ui::icon::Icon;

const MIN_CONCURRENT: u32 = 1;
const MAX_CONCURRENT: u32 = 100;

#[derive(Clone, Debug)]
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
    headers_input: gpui::Entity<TextareaState>,
    include_input: gpui::Entity<TextareaState>,
    exclude_input: gpui::Entity<TextareaState>,
    crawl_subdomains: bool,
    list_mode: bool,
    list_urls_input: gpui::Entity<TextareaState>,
    concurrency_input: gpui::Entity<InputState>,
    block_images: bool,
    /// One in-flight save per setting key, so a save for one setting cannot
    /// cancel another's. Dropping a task cancels it, which is what debounces a
    /// key being edited repeatedly.
    save_tasks: HashMap<&'static str, Task<()>>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl CrawlBar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://books.toscrape.com"));

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
            TextareaState::new(window, cx)
                .placeholder("Authorization: Bearer token  (one Name: Value per line)")
                .auto_grow(3, 8)
        });
        let include_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("/blog/.*  (one regex per line)")
                .auto_grow(3, 8)
        });
        let exclude_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("/tag/.*  (one regex per line)")
                .auto_grow(3, 8)
        });
        let list_urls_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("https://example.com/page1\nhttps://example.com/page2")
                .auto_grow(4, 10)
        });

        let concurrency_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(
                    AppSettings::global(cx)
                        .settings
                        .crawl
                        .max_concurrent
                        .to_string(),
                )
                .step(1.)
                .min(MIN_CONCURRENT as f64)
                .max(MAX_CONCURRENT as f64)
        });
        let concurrency_sub = cx.subscribe_in(
            &concurrency_input,
            window,
            |this, _state, event: &InputEvent, _window, cx| {
                if matches!(event, InputEvent::Change) {
                    this.persist_concurrency(cx);
                }
            },
        );

        Self {
            focus_handle: cx.focus_handle(),
            url_input,
            running: false,
            has_results: false,
            default_mode: RenderMode::Chrome,
            advanced_open: false,
            headers_input,
            include_input,
            exclude_input,
            crawl_subdomains: false,
            list_mode: false,
            list_urls_input,
            concurrency_input,
            block_images: AppSettings::global(cx).settings.crawl.block_images,
            save_tasks: HashMap::new(),
            _subscriptions: vec![input_sub, concurrency_sub],
        }
    }

    fn concurrency(&self, cx: &Context<Self>) -> Option<u32> {
        self.concurrency_input
            .read(cx)
            .value()
            .trim()
            .parse::<u32>()
            .ok()
            .map(|value| value.clamp(MIN_CONCURRENT, MAX_CONCURRENT))
    }

    /// Mirrors the bar's value into the global setting the settings dialog
    /// shows, so both places stay in sync and the choice survives a restart.
    fn persist_concurrency(&mut self, cx: &mut Context<Self>) {
        let Some(value) = self.concurrency(cx) else {
            return;
        };
        if AppSettings::global(cx).settings.crawl.max_concurrent == value {
            return;
        }
        AppSettings::global_mut(cx).settings.crawl.max_concurrent = value;
        self.save_setting("crawl.max_concurrent", value.to_string(), cx);
    }

    /// Kept in the global settings too, so the settings dialog and the next
    /// launch agree with the panel. `resolve_config` reads the setting rather
    /// than the config's field, a bool having no "unset" to fall back from.
    fn set_block_images(&mut self, block_images: bool, cx: &mut Context<Self>) {
        self.block_images = block_images;
        AppSettings::global_mut(cx).settings.crawl.block_images = block_images;
        self.save_setting("crawl.block_images", block_images.to_string(), cx);
        cx.notify();
    }

    fn save_setting(&mut self, key: &'static str, value: String, cx: &mut Context<Self>) {
        let database = AppDatabase::global(cx).clone();
        let task = cx.spawn(async move |_, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            if let Err(error) = database.save_setting(key, &value).await {
                tracing::error!("Failed to save setting {}: {}", key, error);
            }
        });
        self.save_tasks.insert(key, task);
    }

    pub(crate) fn build_config(&self, cx: &Context<Self>) -> CrawlConfig {
        let user_agent = {
            let val = crate::app_settings::AppSettings::global(cx)
                .settings
                .crawl
                .user_agent
                .trim()
                .to_string();
            if val.is_empty() { None } else { Some(val) }
        };

        let parse_lines = |entity: &gpui::Entity<TextareaState>| -> Vec<String> {
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
            max_concurrent: self.concurrency(cx).unwrap_or(0),
            delay_ms: 0,
            timeout_seconds: 30,
            respect_robots_txt: true,
            follow_sitemaps: true,
            block_images: self.block_images,
            near_duplicate_threshold: 90,
            content_selector: String::new(),
            user_agent,
            extra_headers,
            include_patterns,
            exclude_patterns,
            crawl_subdomains: self.crawl_subdomains,
            list_mode: self.list_mode,
            seed_urls,
            check_resources: true,
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

    /// Puts the bar into the state a recorded crawl ran with, so a recrawl shows
    /// what is actually running rather than whatever was last typed. Does not
    /// emit `Start` - the caller drives the crawl itself.
    pub fn restore_from_config(
        &mut self,
        url: &str,
        mode: RenderMode,
        config: &CrawlConfig,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.url_input
            .update(cx, |state, cx| state.set_value(url, window, cx));
        self.headers_input.update(cx, |state, cx| {
            let headers = config
                .extra_headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}"))
                .collect::<Vec<_>>()
                .join("\n");
            state.set_value(headers, window, cx)
        });
        self.include_input.update(cx, |state, cx| {
            state.set_value(config.include_patterns.join("\n"), window, cx)
        });
        self.exclude_input.update(cx, |state, cx| {
            state.set_value(config.exclude_patterns.join("\n"), window, cx)
        });
        self.list_urls_input.update(cx, |state, cx| {
            state.set_value(config.seed_urls.join("\n"), window, cx)
        });
        if config.max_concurrent > 0 {
            self.concurrency_input.update(cx, |state, cx| {
                state.set_value(config.max_concurrent.to_string(), window, cx)
            });
        }

        self.default_mode = mode;
        self.crawl_subdomains = config.crawl_subdomains;
        self.list_mode = config.list_mode;
        self.block_images = config.block_images;
        self.running = true;
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
        let block_images = self.block_images;

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

        let advanced_panel =
            div()
                .id("crawl-bar-advanced")
                .px_3()
                .pb_2()
                .pt_1()
                .bg(cx.theme().background)
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .flex_wrap()
                        // The switches and the number input are one line tall each,
                        // so they stack in a column of their own rather than
                        // sitting beside the auto-growing textareas, which would
                        // stretch every one of them to the tallest control's height.
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .pr_4()
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(
                                    Switch::new("list-mode-toggle")
                                        .small()
                                        .label("List Mode")
                                        .tooltip(
                                            "Crawl only the URLs you paste below, \
                                         instead of following links from a start URL.",
                                        )
                                        .checked(list_mode)
                                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                            this.list_mode = *checked;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Switch::new("subdomains-toggle")
                                        .small()
                                        .label("Subdomains")
                                        .tooltip(
                                            "Treat subdomains as part of the site, \
                                         so links to them are crawled rather than \
                                         recorded as external.",
                                        )
                                        .checked(crawl_subdomains)
                                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                            this.crawl_subdomains = *checked;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Switch::new("block-images-toggle")
                                        .small()
                                        .label("Block Images")
                                        .tooltip(
                                            "Stop Chrome loading images. Faster, far less \
                                         traffic to the site, but image sizes and \
                                         broken images go unreported.",
                                        )
                                        .checked(block_images)
                                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                            this.set_block_images(*checked, cx);
                                        })),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Concurrency"),
                                        )
                                        .child(div().w(px(96.)).child(
                                            NumberInput::new(&self.concurrency_input).small(),
                                        )),
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
                                .child(Textarea::new(&self.include_input)),
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
                                .child(Textarea::new(&self.exclude_input)),
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
                                .child(Textarea::new(&self.headers_input)),
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
                            .child(div().w_full().child(Textarea::new(&self.list_urls_input))),
                    )
                });

        div()
            .id("crawl-bar")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .child(main_row)
            .when(advanced_open && !running, |el| el.child(advanced_panel))
    }
}
