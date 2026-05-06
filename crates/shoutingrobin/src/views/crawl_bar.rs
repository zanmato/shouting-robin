use gpui::{
    Action, Anchor, AppContext, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window, div, prelude::FluentBuilder,
};
use gpui_component::{
    ActiveTheme, Sizable as _,
    button::{Button, ButtonVariants as _, DropdownButton},
    input::{Input, InputEvent, InputState},
};
use serde::Deserialize;

use crate::crawl::RenderMode;

#[derive(Clone, Debug)]
pub enum CrawlBarEvent {
    Start { url: String, mode: RenderMode },
    Stop,
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
    default_mode: RenderMode,
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

        Self {
            focus_handle: cx.focus_handle(),
            url_input,
            running: false,
            default_mode: RenderMode::Http,
            _subscriptions: vec![input_sub],
        }
    }

    fn start_crawl(&mut self, mode: RenderMode, cx: &mut Context<Self>) {
        let url = self.url_input.read(cx).value().to_string();
        if url.trim().is_empty() {
            return;
        }
        self.default_mode = mode;
        self.running = true;
        cx.emit(CrawlBarEvent::Start { url, mode });
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
        let default_label = self.default_label();
        let default_mode = self.default_mode;

        div()
            .id("crawl-bar")
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .on_action(cx.listener(Self::on_crawl_http))
            .on_action(cx.listener(Self::on_crawl_chrome))
            .child(div().flex_1().child(Input::new(&self.url_input).small()))
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
    }
}
