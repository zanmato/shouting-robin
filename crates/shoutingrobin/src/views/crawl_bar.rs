use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gpui_kit::component::{
    ActiveTheme, Disableable as _, IconName, Selectable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState, NumberInput, Textarea, TextareaState},
    scroll::ScrollableElement as _,
    switch::Switch,
    tooltip::Tooltip,
    v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext, Bounds, Context, ElementId, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, KeyBinding, ParentElement, Pixels, Render,
    SharedString, StatefulInteractiveElement, Styled, Subscription, Task, Window, actions,
    anchored, canvas, deferred, div, point, prelude::FluentBuilder, px,
};

use crate::app_database::AppDatabase;
use crate::app_settings::AppSettings;
use crate::crawl::site_settings::{
    InputProblem, PatternList, SiteSettings, UrlVerdict, header_problem, site_host, url_verdict,
};
use crate::crawl::{CrawlConfig, LayoutCapture, RenderMode};
use crate::ui::icon::Icon;
use crate::ui::tag::{Tone, tone_tag};

const MIN_CONCURRENT: u32 = 1;
const MAX_CONCURRENT: u32 = 100;
const MAX_PAGES: u32 = 1_000_000;

/// Typing an address waits this long before looking up what is remembered
/// for its site, so each keystroke does not read the database.
const SITE_LOOKUP_DELAY: Duration = Duration::from_millis(300);

const CONTEXT: &str = "CrawlSettings";

/// How many setting tags fit at the end of the address field before the rest
/// are counted in one.
const SHOWN_TAGS: usize = 4;

actions!(
    crawl_settings,
    [
        /// Close the crawl settings.
        CloseCrawlSettings,
    ]
);

pub fn key_bindings() -> Vec<KeyBinding> {
    vec![KeyBinding::new("escape", CloseCrawlSettings, Some(CONTEXT))]
}

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

struct HeaderRow {
    id: usize,
    name: Entity<InputState>,
    value: Entity<InputState>,
    _subscriptions: [Subscription; 2],
}

pub struct CrawlBar {
    focus_handle: FocusHandle,
    pub url_input: Entity<InputState>,
    pub running: bool,
    pub has_results: bool,
    default_mode: RenderMode,
    settings_open: bool,
    list_mode: bool,
    list_urls_input: Entity<TextareaState>,
    crawl_subdomains: bool,
    include_input: Entity<TextareaState>,
    exclude_input: Entity<TextareaState>,
    include: PatternList,
    exclude: PatternList,
    test_url_input: Entity<InputState>,
    max_pages_input: Entity<InputState>,
    concurrency_input: Entity<InputState>,
    block_images: bool,
    capture_layout: bool,
    scroll_for_layout: bool,
    headers: Vec<HeaderRow>,
    next_header_id: usize,
    /// The site of the address in the bar, as it was last looked up.
    site: Option<String>,
    /// When the panel holds settings remembered for `site`, when they were
    /// saved.
    remembered_at: Option<i64>,
    site_lookup: Option<Task<()>>,
    /// Where the bar was last painted, which the panel opens under and takes
    /// its width from.
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    _subscriptions: Vec<Subscription>,
}

impl CrawlBar {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let defaults = site_defaults(cx);

        let url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://books.toscrape.com"));
        let url_subscription = cx.subscribe_in(
            &url_input,
            window,
            |this, _state, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } if !this.running => {
                    this.start_crawl(this.default_mode, cx);
                }
                InputEvent::Change => this.look_up_site(window, cx),
                _ => {}
            },
        );

        let include_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Leave empty to crawl every page")
                .auto_grow(2, 8)
        });
        let exclude_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Leave empty to skip nothing")
                .auto_grow(2, 8)
        });
        let list_urls_input = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("https://example.com/page-1\nhttps://example.com/page-2")
                .auto_grow(6, 16)
        });
        let test_url_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Paste a URL from the site"));
        let max_pages_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(defaults.max_pages.to_string())
                .step(100.)
                .min(0.)
                .max(MAX_PAGES as f64)
        });
        let concurrency_input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(defaults.max_concurrent.to_string())
                .step(1.)
                .min(MIN_CONCURRENT as f64)
                .max(MAX_CONCURRENT as f64)
        });

        let mut subscriptions = vec![url_subscription];
        for input in [&include_input, &exclude_input] {
            subscriptions.push(cx.subscribe(input, |this, _state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.check_patterns(cx);
                }
            }));
        }
        subscriptions.push(cx.subscribe(
            &list_urls_input,
            |_this, _state, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    cx.notify();
                }
            },
        ));
        for input in [&test_url_input, &max_pages_input, &concurrency_input] {
            subscriptions.push(
                cx.subscribe(input, |_this, _state, event: &InputEvent, cx| {
                    if matches!(event, InputEvent::Change) {
                        cx.notify();
                    }
                }),
            );
        }

        Self {
            focus_handle: cx.focus_handle(),
            url_input,
            running: false,
            has_results: false,
            default_mode: RenderMode::Chrome,
            settings_open: false,
            list_mode: false,
            list_urls_input,
            crawl_subdomains: defaults.crawl_subdomains,
            include_input,
            exclude_input,
            include: PatternList::default(),
            exclude: PatternList::default(),
            test_url_input,
            max_pages_input,
            concurrency_input,
            block_images: defaults.block_images,
            capture_layout: defaults.capture_layout,
            scroll_for_layout: defaults.scroll_for_layout,
            headers: Vec::new(),
            next_header_id: 0,
            site: None,
            remembered_at: None,
            site_lookup: None,
            bounds: Rc::new(Cell::new(None)),
            _subscriptions: subscriptions,
        }
    }

    fn check_patterns(&mut self, cx: &mut Context<Self>) {
        self.include = PatternList::parse(&self.include_input.read(cx).value());
        self.exclude = PatternList::parse(&self.exclude_input.read(cx).value());
        cx.notify();
    }

    fn number(input: &Entity<InputState>, range: (u32, u32), cx: &App) -> Option<u32> {
        input
            .read(cx)
            .value()
            .trim()
            .parse::<u32>()
            .ok()
            .map(|value| value.clamp(range.0, range.1))
    }

    fn concurrency(&self, cx: &App) -> Option<u32> {
        Self::number(
            &self.concurrency_input,
            (MIN_CONCURRENT, MAX_CONCURRENT),
            cx,
        )
    }

    fn max_pages(&self, cx: &App) -> u32 {
        Self::number(&self.max_pages_input, (0, MAX_PAGES), cx).unwrap_or(0)
    }

    pub(crate) fn set_capture_layout(&mut self, capture_layout: bool, cx: &mut Context<Self>) {
        self.capture_layout = capture_layout;
        cx.notify();
    }

    fn add_header(&mut self, name: &str, value: &str, window: &mut Window, cx: &mut Context<Self>) {
        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Name")
                .default_value(name.to_string())
        });
        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Value")
                .masked(true)
                .default_value(value.to_string())
        });
        let notify = |_this: &mut Self,
                      _state: Entity<InputState>,
                      event: &InputEvent,
                      cx: &mut Context<Self>| {
            if matches!(event, InputEvent::Change) {
                cx.notify();
            }
        };
        let subscriptions = [
            cx.subscribe(&name_input, notify),
            cx.subscribe(&value_input, notify),
        ];
        self.headers.push(HeaderRow {
            id: self.next_header_id,
            name: name_input,
            value: value_input,
            _subscriptions: subscriptions,
        });
        self.next_header_id += 1;
        cx.notify();
    }

    /// The header rows as typed, leaving out the empty ones.
    fn header_entries(&self, cx: &App) -> Vec<(usize, String, String)> {
        self.headers
            .iter()
            .map(|row| {
                (
                    row.id,
                    row.name.read(cx).value().trim().to_string(),
                    row.value.read(cx).value().trim().to_string(),
                )
            })
            .filter(|(_, name, value)| !name.is_empty() || !value.is_empty())
            .collect()
    }

    fn header_problems(&self, cx: &App) -> Vec<(usize, String)> {
        self.header_entries(cx)
            .into_iter()
            .filter_map(|(id, name, value)| {
                header_problem(&name, &value).map(|problem| (id, problem))
            })
            .collect()
    }

    /// Patterns are only read in a site crawl, so a list crawl is not held
    /// up by one.
    fn problem_count(&self, cx: &App) -> usize {
        let pattern_problems = if self.list_mode {
            0
        } else {
            self.include.problems.len() + self.exclude.problems.len()
        };
        pattern_problems + self.header_problems(cx).len()
    }

    fn list_urls(&self, cx: &App) -> Vec<String> {
        lines(&self.list_urls_input.read(cx).value())
    }

    /// What the panel holds, as it is remembered for a site.
    fn site_settings(&self, cx: &App) -> SiteSettings {
        SiteSettings {
            crawl_subdomains: self.crawl_subdomains,
            include_patterns: lines(&self.include_input.read(cx).value()),
            exclude_patterns: lines(&self.exclude_input.read(cx).value()),
            max_pages: self.max_pages(cx),
            max_concurrent: self.concurrency(cx).unwrap_or(MIN_CONCURRENT),
            block_images: self.block_images,
            capture_layout: self.capture_layout,
            scroll_for_layout: self.scroll_for_layout,
            header_names: self
                .header_entries(cx)
                .into_iter()
                .map(|(_, name, _)| name)
                .filter(|name| !name.is_empty())
                .collect(),
        }
    }

    fn apply_site_settings(
        &mut self,
        settings: &SiteSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.crawl_subdomains = settings.crawl_subdomains;
        self.block_images = settings.block_images;
        self.capture_layout = settings.capture_layout;
        self.scroll_for_layout = settings.scroll_for_layout;
        set_text(
            &self.include_input,
            settings.include_patterns.join("\n"),
            window,
            cx,
        );
        set_text(
            &self.exclude_input,
            settings.exclude_patterns.join("\n"),
            window,
            cx,
        );
        self.max_pages_input.update(cx, |state, cx| {
            state.set_value(settings.max_pages.to_string(), window, cx)
        });
        if settings.max_concurrent > 0 {
            self.concurrency_input.update(cx, |state, cx| {
                state.set_value(settings.max_concurrent.to_string(), window, cx)
            });
        }
        self.headers.clear();
        for name in &settings.header_names {
            self.add_header(name, "", window, cx);
        }
        self.check_patterns(cx);
    }

    /// Reads what is remembered for the site of the address being typed. A
    /// site with nothing remembered keeps what the panel holds, unless that
    /// came from another site, which it then does not belong to.
    fn look_up_site(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let site = site_host(&self.url_input.read(cx).value());
        if site == self.site {
            return;
        }
        self.site = site.clone();
        let pool = AppDatabase::global(cx).pool().clone();
        self.site_lookup = Some(cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(SITE_LOOKUP_DELAY).await;
            let remembered = match &site {
                Some(site) => match crate::storage::load_site_settings(&pool, site).await {
                    Ok(remembered) => remembered,
                    Err(error) => {
                        tracing::error!(%error, site, "failed to read the site's settings");
                        None
                    }
                },
                None => None,
            };
            let updated = this.update_in(cx, |this, window, cx| {
                match remembered {
                    Some((settings, saved_at)) => {
                        this.apply_site_settings(&settings, window, cx);
                        this.remembered_at = Some(saved_at);
                    }
                    None => {
                        if this.remembered_at.take().is_some() {
                            let defaults = site_defaults(cx);
                            this.apply_site_settings(&defaults, window, cx);
                        }
                    }
                }
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!(%error, "the crawl bar closed before the site was looked up");
            }
        }));
    }

    fn remember_site(&mut self, cx: &mut Context<Self>) {
        let Some(site) = site_host(&self.url_input.read(cx).value()) else {
            return;
        };
        let settings = self.site_settings(cx);
        let saved_at = chrono::Utc::now().timestamp();
        self.site = Some(site.clone());
        self.remembered_at = Some(saved_at);
        let pool = AppDatabase::global(cx).pool().clone();
        cx.background_spawn(async move {
            if let Err(error) =
                crate::storage::save_site_settings(&pool, &site, &settings, saved_at).await
            {
                tracing::error!(%error, site, "failed to remember the site's settings");
            }
        })
        .detach();
    }

    fn forget_site(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(site) = self.site.clone() else {
            return;
        };
        self.remembered_at = None;
        let defaults = site_defaults(cx);
        self.apply_site_settings(&defaults, window, cx);
        let pool = AppDatabase::global(cx).pool().clone();
        cx.background_spawn(async move {
            if let Err(error) = crate::storage::forget_site_settings(&pool, &site).await {
                tracing::error!(%error, site, "failed to forget the site's settings");
            }
        })
        .detach();
        cx.notify();
    }

    pub(crate) fn build_config(&self, cx: &App) -> CrawlConfig {
        let user_agent = {
            let value = AppSettings::global(cx).settings.crawl.user_agent.trim();
            (!value.is_empty()).then(|| value.to_string())
        };
        let extra_headers = self
            .header_entries(cx)
            .into_iter()
            .filter(|(_, name, value)| header_problem(name, value).is_none())
            .map(|(_, name, value)| (name, value))
            .collect();
        let list_mode = self.list_mode;
        let seed_urls = if list_mode {
            self.list_urls(cx)
        } else {
            Vec::new()
        };
        // A list crawl fetches the listed URLs and nothing else, so the site
        // options it hides are not sent either.
        let site_only = |value: Vec<String>| if list_mode { Vec::new() } else { value };

        CrawlConfig {
            max_pages: if list_mode { 0 } else { self.max_pages(cx) },
            max_concurrent: self.concurrency(cx).unwrap_or(0),
            delay_ms: 0,
            // Zero means "not set here": `ShoutingRobinApp::resolve_config`
            // fills every unset value from the Settings dialog, so a number
            // written here would silently override what the user configured.
            timeout_seconds: 0,
            respect_robots_txt: true,
            follow_sitemaps: true,
            block_images: self.block_images,
            near_duplicate_threshold: 0,
            content_selector: String::new(),
            user_agent,
            extra_headers,
            include_patterns: site_only(lines(&self.include_input.read(cx).value())),
            exclude_patterns: site_only(lines(&self.exclude_input.read(cx).value())),
            crawl_subdomains: self.crawl_subdomains && !list_mode,
            list_mode,
            seed_urls,
            check_resources: true,
            layout_capture: LayoutCapture::from_switches(
                self.capture_layout,
                self.scroll_for_layout,
            ),
        }
    }

    /// A crawl with a pattern or header that would not apply as typed does
    /// not start. The panel opens on it instead.
    pub(crate) fn start_crawl(&mut self, mode: RenderMode, cx: &mut Context<Self>) {
        if self.problem_count(cx) > 0 {
            self.settings_open = true;
            cx.notify();
            return;
        }
        let url = if self.list_mode {
            let first = self.list_urls(cx).into_iter().next();
            match first {
                Some(url) => url,
                None => {
                    self.settings_open = true;
                    cx.notify();
                    return;
                }
            }
        } else {
            self.url_input.read(cx).value().to_string()
        };
        if url.trim().is_empty() {
            return;
        }
        let config = self.build_config(cx);
        if !self.list_mode {
            self.remember_site(cx);
        }
        self.default_mode = mode;
        self.mark_running();
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
        // The panel shows what this crawl ran with, not what is remembered
        // for its site.
        self.site = site_host(url);
        self.remembered_at = None;
        self.site_lookup = None;
        set_text(
            &self.include_input,
            config.include_patterns.join("\n"),
            window,
            cx,
        );
        set_text(
            &self.exclude_input,
            config.exclude_patterns.join("\n"),
            window,
            cx,
        );
        set_text(
            &self.list_urls_input,
            config.seed_urls.join("\n"),
            window,
            cx,
        );
        self.max_pages_input.update(cx, |state, cx| {
            state.set_value(config.max_pages.to_string(), window, cx)
        });
        if config.max_concurrent > 0 {
            self.concurrency_input.update(cx, |state, cx| {
                state.set_value(config.max_concurrent.to_string(), window, cx)
            });
        }
        self.headers.clear();
        for (name, value) in &config.extra_headers {
            self.add_header(name, value, window, cx);
        }

        self.default_mode = mode;
        self.crawl_subdomains = config.crawl_subdomains;
        self.list_mode = config.list_mode;
        self.block_images = config.block_images;
        self.capture_layout = config.layout_capture.is_enabled();
        self.scroll_for_layout = config.layout_capture.scrolls();
        self.check_patterns(cx);
        self.mark_running();
        cx.notify();
    }

    /// The settings are hidden while a crawl runs. Closing them here keeps a
    /// panel left open at Start from reappearing when the crawl ends, long
    /// after it was last seen.
    fn mark_running(&mut self) {
        self.running = true;
        self.settings_open = false;
    }

    fn on_stop(&mut self, _: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.running = false;
        cx.emit(CrawlBarEvent::Stop);
        cx.notify();
    }

    fn on_close_settings(
        &mut self,
        _: &CloseCrawlSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.settings_open = false;
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// The settings that differ from what a new site starts with, as short
    /// labels for the bar to show while the panel is closed.
    fn summary(&self, cx: &App) -> Vec<(SharedString, Tone)> {
        let defaults = site_defaults(cx);
        let current = self.site_settings(cx);
        let mut summary = Vec::new();
        let problems = self.problem_count(cx);
        if problems > 0 {
            summary.push((
                format!("Fix {}", plural(problems, "setting", "settings")).into(),
                Tone::Err,
            ));
        }
        if !self.list_mode && current.max_pages != defaults.max_pages {
            let label = if current.max_pages == 0 {
                "No page limit".to_string()
            } else {
                format!(
                    "Max {}",
                    plural(current.max_pages as usize, "page", "pages")
                )
            };
            summary.push((label.into(), Tone::Neutral));
        }
        if !self.list_mode && !self.include.is_empty() {
            summary.push((
                format!("Only {}", plural(self.include.len(), "pattern", "patterns")).into(),
                Tone::Neutral,
            ));
        }
        if !self.list_mode && !self.exclude.is_empty() {
            summary.push((
                format!(
                    "Skipping {}",
                    plural(self.exclude.len(), "pattern", "patterns")
                )
                .into(),
                Tone::Neutral,
            ));
        }
        if !self.list_mode && current.crawl_subdomains {
            summary.push(("Subdomains".into(), Tone::Neutral));
        }
        if self.default_mode.renders_javascript() {
            if current.block_images != defaults.block_images {
                let label = if current.block_images {
                    "Images blocked"
                } else {
                    "Images loaded"
                };
                summary.push((label.into(), Tone::Neutral));
            }
            if current.capture_layout != defaults.capture_layout
                || (current.capture_layout
                    && current.scroll_for_layout != defaults.scroll_for_layout)
            {
                let label = match (current.capture_layout, current.scroll_for_layout) {
                    (false, _) => "No layout",
                    (true, true) => "Layout with scrolling",
                    (true, false) => "Layout captured",
                };
                summary.push((label.into(), Tone::Neutral));
            }
        }
        if current.max_concurrent != defaults.max_concurrent {
            summary.push((
                format!("{} at a time", current.max_concurrent).into(),
                Tone::Neutral,
            ));
        }
        let headers = self.header_entries(cx).len();
        if headers > 0 {
            summary.push((plural(headers, "header", "headers").into(), Tone::Neutral));
        }
        summary
    }
}

/// What a site nothing is remembered for starts from.
fn site_defaults(cx: &App) -> SiteSettings {
    let crawl = &AppSettings::global(cx).settings.crawl;
    SiteSettings {
        max_pages: crawl.max_pages,
        max_concurrent: crawl.max_concurrent,
        block_images: crawl.block_images,
        capture_layout: crawl.capture_layout,
        scroll_for_layout: crawl.scroll_for_layout,
        ..SiteSettings::default()
    }
}

fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

fn set_text(
    input: &Entity<TextareaState>,
    text: String,
    window: &mut Window,
    cx: &mut Context<CrawlBar>,
) {
    input.update(cx, |state, cx| state.set_value(text, window, cx));
}

impl EventEmitter<CrawlBarEvent> for CrawlBar {}

impl Focusable for CrawlBar {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

fn group(title: &'static str, trailing: Option<AnyElement>, cx: &App) -> gpui_kit::Div {
    v_flex()
        .flex_1()
        .min_w(px(0.))
        .gap_3()
        .p_3()
        .rounded(cx.theme().radius_lg)
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().group_box)
        .child(
            h_flex()
                .justify_between()
                .gap_2()
                .child(div().text_sm().font_semibold().child(title))
                .children(trailing),
        )
}

fn help(text: impl Into<SharedString>, cx: &App) -> gpui_kit::Div {
    div()
        .text_xs()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
}

fn field_label(label: &'static str, hint: &'static str, cx: &App) -> gpui_kit::Div {
    h_flex()
        .justify_between()
        .gap_2()
        .child(div().text_sm().child(label))
        .child(help(hint, cx))
}

fn problem_lines(problems: &[InputProblem], cx: &App) -> Option<AnyElement> {
    if problems.is_empty() {
        return None;
    }
    Some(
        v_flex()
            .gap_0p5()
            .text_xs()
            .text_color(cx.theme().danger)
            .children(problems.iter().map(|problem| {
                div().child(SharedString::from(format!(
                    "{}  {}",
                    problem.line, problem.message
                )))
            }))
            .into_any_element(),
    )
}

/// A switch with its help text under the label, so what it does is read
/// without hovering.
fn switch_setting(
    switch: Switch,
    help_text: &'static str,
    indented: bool,
    cx: &App,
) -> gpui_kit::Div {
    v_flex()
        .gap_0p5()
        .when(indented, |el| el.pl(px(36.)))
        .child(switch)
        .child(help(help_text, cx).pl(px(36.)))
}

impl CrawlBar {
    fn render_main_row(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let running = self.running;
        let default_mode = self.default_mode;
        let list_mode = self.list_mode;
        let show_dot = !self.settings_open && !running && !self.summary(cx).is_empty();

        h_flex()
            .id("crawl-bar-main")
            .gap_2()
            .px_3()
            .py_2()
            .when(!running, |el| {
                el.child(
                    div()
                        .relative()
                        .child(
                            Button::new("toggle-crawl-settings")
                                .small()
                                .ghost()
                                .icon(Icon::Sliders)
                                .tooltip("Crawl settings")
                                .selected(self.settings_open)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.settings_open = !this.settings_open;
                                    cx.notify();
                                })),
                        )
                        .when(show_dot, |el| {
                            el.child(
                                div()
                                    .absolute()
                                    .top(px(3.))
                                    .right(px(3.))
                                    .size(px(6.))
                                    .rounded_full()
                                    .bg(cx.theme().primary),
                            )
                        }),
                )
            })
            // The same input in both modes, so switching does not move the
            // bar. A list crawl starts from its list, so the address is
            // disabled meanwhile.
            .child(
                div().flex_1().child(
                    Input::new(&self.url_input)
                        .small()
                        .disabled(list_mode && !running)
                        .when_some(self.render_summary(cx), |input, summary| {
                            input.suffix(summary)
                        }),
                ),
            )
            .when(!running, |el| {
                el.child(
                    h_flex()
                        .gap_1()
                        .child(
                            Switch::new("mode-switch")
                                .small()
                                .checked(default_mode.renders_javascript())
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
                                .text_color(if default_mode.renders_javascript() {
                                    cx.theme().foreground
                                } else {
                                    cx.theme().muted_foreground
                                })
                                .child("Chrome"),
                        ),
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
            })
    }

    /// The settings that differ from a new site's, as tags at the end of the
    /// address field. They sit in the bar's one row, so they come and go
    /// without moving the results, and the panel opens on a click.
    fn render_summary(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.running {
            return None;
        }
        let summary = self.summary(cx);
        if summary.is_empty() {
            return None;
        }
        let remembered = (self.remembered_at.is_some() && !self.list_mode)
            .then(|| site_host(&self.url_input.read(cx).value()))
            .flatten();
        let mut tooltip: Vec<String> = Vec::new();
        if let Some(site) = &remembered {
            tooltip.push(format!("Remembered for {site}"));
        }
        tooltip.extend(summary.iter().map(|(label, _)| label.to_string()));
        let tooltip = SharedString::from(tooltip.join("\n"));
        let hidden = summary.len().saturating_sub(SHOWN_TAGS);

        Some(
            h_flex()
                .id("crawl-settings-summary")
                .flex_shrink_0()
                .gap_1()
                .cursor_pointer()
                .children(
                    summary
                        .into_iter()
                        .take(SHOWN_TAGS)
                        .map(|(label, tone)| tone_tag(tone, cx).child(label)),
                )
                .when(hidden > 0, |el| {
                    el.child(
                        tone_tag(Tone::Neutral, cx).child(SharedString::from(format!("+{hidden}"))),
                    )
                })
                .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx))
                .on_click(cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.settings_open = true;
                    cx.notify();
                }))
                .into_any_element(),
        )
    }

    fn render_scope_group(&self, cx: &mut Context<Self>) -> AnyElement {
        let verdict = {
            let url = self.test_url_input.read(cx).value().trim().to_string();
            (!url.is_empty()).then(|| {
                if !self.include.problems.is_empty() || !self.exclude.problems.is_empty() {
                    return h_flex().child(help("Fix the patterns above to test a URL", cx));
                }
                let (tone, label, reason) = match url_verdict(&url, &self.include, &self.exclude) {
                    UrlVerdict::Crawled { included_by } => (
                        Tone::Ok,
                        "Crawled",
                        match included_by {
                            Some(pattern) => format!("matches {pattern}"),
                            None => "no pattern skips it".to_string(),
                        },
                    ),
                    UrlVerdict::Excluded { by } => (Tone::Err, "Skipped", format!("matches {by}")),
                    UrlVerdict::NotIncluded => (
                        Tone::Err,
                        "Skipped",
                        "matches no include pattern".to_string(),
                    ),
                };
                h_flex()
                    .gap_2()
                    .text_xs()
                    .child(tone_tag(tone, cx).child(label))
                    .child(
                        div()
                            .text_color(cx.theme().muted_foreground)
                            .child(SharedString::from(reason)),
                    )
            })
        };

        let list_mode = self.list_mode;
        let start_from = ButtonGroup::new("crawl-start-from")
            .outline()
            .xsmall()
            .child(
                Button::new("start-from-site")
                    .label("Site")
                    .selected(!list_mode),
            )
            .child(
                Button::new("start-from-list")
                    .label("List")
                    .selected(list_mode),
            )
            .on_click(cx.listener(|this, selected: &Vec<usize>, _, cx| {
                this.list_mode = selected.first() == Some(&1);
                cx.notify();
            }))
            .into_any_element();
        let scope = group("What to crawl", Some(start_from), cx)
            .flex_grow(1.6)
            .flex_basis(px(360.));
        if list_mode {
            let count = self.list_urls(cx).len();
            return scope
                .child(
                    v_flex()
                        .gap_1()
                        .child(field_label("URLs to crawl", "One per line", cx))
                        .child(Textarea::new(&self.list_urls_input))
                        .child(help(
                            if count == 0 {
                                "Only these URLs are fetched. Links on them aren't followed."
                                    .to_string()
                            } else {
                                format!(
                                    "{}. Only these are fetched, and links on them aren't followed.",
                                    plural(count, "URL", "URLs")
                                )
                            },
                            cx,
                        )),
                )
                .into_any_element();
        }
        scope
            .child(switch_setting(
                Switch::new("subdomains-toggle")
                    .small()
                    .label("Include subdomains")
                    .checked(self.crawl_subdomains)
                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                        this.crawl_subdomains = *checked;
                        cx.notify();
                    })),
                "Crawl links to other subdomains of the site as part of it.",
                false,
                cx,
            ))
            .child(
                v_flex()
                    .gap_1()
                    .child(field_label(
                        "Only crawl URLs matching",
                        "Regex, one per line",
                        cx,
                    ))
                    .child(
                        div()
                            .font_family(cx.theme().mono_font_family.clone())
                            .child(Textarea::new(&self.include_input)),
                    )
                    .children(problem_lines(&self.include.problems, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(field_label("Skip URLs matching", "Regex, one per line", cx))
                    .child(
                        div()
                            .font_family(cx.theme().mono_font_family.clone())
                            .child(Textarea::new(&self.exclude_input)),
                    )
                    .children(problem_lines(&self.exclude.problems, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .pt_2()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(help("Test a URL against the patterns", cx))
                    .child(Input::new(&self.test_url_input).small())
                    .children(verdict),
            )
            .child(
                h_flex()
                    .justify_between()
                    .gap_3()
                    .child(
                        v_flex()
                            .child(div().text_sm().child("Stop after"))
                            .child(help("0 crawls every page", cx)),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(110.))
                                    .child(NumberInput::new(&self.max_pages_input).small()),
                            )
                            .child(help("pages", cx)),
                    ),
            )
            .into_any_element()
    }

    fn render_chrome_group(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let chrome = self.default_mode.renders_javascript();
        let capture_layout = self.capture_layout;
        group(
            "Chrome rendering",
            (!chrome).then(|| help("Not used with HTTP", cx).into_any_element()),
            cx,
        )
        .flex_basis(px(240.))
        .when(!chrome, |el| {
            el.child(
                h_flex()
                    .gap_1()
                    .flex_wrap()
                    .p_2()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(cx.theme().border)
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("These need Chrome.")
                    .child(
                        Button::new("switch-to-chrome")
                            .link()
                            .xsmall()
                            .label("Switch to Chrome")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.default_mode = RenderMode::Chrome;
                                cx.notify();
                            })),
                    ),
            )
        })
        .child(
            v_flex()
                .gap_3()
                .when(!chrome, |el| el.opacity(0.5))
                .child(switch_setting(
                    Switch::new("block-images-toggle")
                        .small()
                        .label("Block images")
                        .disabled(!chrome)
                        .checked(self.block_images)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.block_images = *checked;
                            cx.notify();
                        })),
                    "Faster and lighter on the site. Image sizes and broken images go unreported.",
                    false,
                    cx,
                ))
                .child(switch_setting(
                    Switch::new("capture-layout-toggle")
                        .small()
                        .label("Capture layout")
                        .disabled(!chrome)
                        .checked(capture_layout)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.set_capture_layout(*checked, cx);
                        })),
                    "Groups pages by template and finds layout changes between crawls.",
                    false,
                    cx,
                ))
                .child(switch_setting(
                    Switch::new("scroll-for-layout-toggle")
                        .small()
                        .label("Scroll each page first")
                        .disabled(!chrome || !capture_layout)
                        .checked(capture_layout && self.scroll_for_layout)
                        .on_click(cx.listener(|this, checked: &bool, _, cx| {
                            this.scroll_for_layout = *checked;
                            cx.notify();
                        })),
                    "Includes content that loads on scroll. Adds a few seconds a page.",
                    true,
                    cx,
                )),
        )
    }

    fn render_requests_group(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let problems = self.header_problems(cx);
        group("Requests", None, cx)
            .flex_basis(px(280.))
            .child(
                h_flex()
                    .justify_between()
                    .gap_3()
                    .child(
                        v_flex()
                            .child(div().text_sm().child("Pages at a time"))
                            .child(help("Chrome tabs in a Chrome crawl", cx)),
                    )
                    .child(
                        div()
                            .w(px(96.))
                            .flex_shrink_0()
                            .child(NumberInput::new(&self.concurrency_input).small()),
                    ),
            )
            .child(
                v_flex()
                    .gap_1p5()
                    .child(field_label(
                        "Custom headers",
                        "Values aren't remembered",
                        cx,
                    ))
                    .children(self.headers.iter().map(|row| {
                        let id = row.id;
                        let problem = problems
                            .iter()
                            .find(|(problem_id, _)| *problem_id == id)
                            .map(|(_, problem)| SharedString::from(problem.clone()));
                        v_flex()
                            .gap_0p5()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .child(Input::new(&row.name).small()),
                                    )
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(0.))
                                            .child(Input::new(&row.value).small().mask_toggle()),
                                    )
                                    .child(
                                        Button::new(ElementId::from(("remove-header", id)))
                                            .ghost()
                                            .xsmall()
                                            .icon(IconName::Close)
                                            .tooltip("Remove header")
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.headers.retain(|row| row.id != id);
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .when_some(problem, |el, problem| {
                                el.child(
                                    div().text_xs().text_color(cx.theme().danger).child(problem),
                                )
                            })
                    }))
                    .child(
                        div().child(
                            Button::new("add-header")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .label("Add header")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.add_header("", "", window, cx);
                                })),
                        ),
                    ),
            )
    }

    fn render_panel(
        &self,
        bar: Bounds<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let width = bar.size.width - px(24.);
        let max_height = (window.viewport_size().height - bar.bottom() - px(48.)).max(px(240.));
        let site = site_host(&self.url_input.read(cx).value());
        let site_line: SharedString = match (&site, self.list_mode) {
            (_, true) => "Applies to this list crawl. List crawls aren't remembered.".into(),
            (None, false) => "Applies to the next crawl, and is remembered for its site.".into(),
            (Some(site), false) => {
                let restored = self
                    .remembered_at
                    .and_then(|saved_at| chrono::DateTime::from_timestamp(saved_at, 0))
                    .map(|saved_at| {
                        format!(
                            " Restored from the crawl on {}.",
                            saved_at.with_timezone(&chrono::Local).format("%-d %b %Y")
                        )
                    })
                    .unwrap_or_default();
                format!("Applies to {site} and is remembered for its next crawl.{restored}").into()
            }
        };
        let can_forget = self.remembered_at.is_some() && !self.list_mode;

        let body = v_flex()
            .gap_3()
            .p_3()
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(help(site_line, cx))
                    .when(can_forget, |el| {
                        el.child(
                            Button::new("forget-site")
                                .link()
                                .xsmall()
                                .label("Forget for this site")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.forget_site(window, cx);
                                })),
                        )
                    }),
            )
            .child(
                h_flex()
                    .items_start()
                    .flex_wrap()
                    .gap_3()
                    .child(self.render_scope_group(cx))
                    .child(self.render_chrome_group(cx))
                    .child(self.render_requests_group(cx)),
            )
            .child(
                h_flex()
                    .justify_between()
                    .gap_2()
                    .child(help(
                        "Defaults for new sites, robots.txt, timeouts, delays and the user agent are in Settings.",
                        cx,
                    ))
                    .child(
                        Button::new("open-crawl-defaults")
                            .link()
                            .xsmall()
                            .label("Open Settings")
                            .on_click(|_, window, cx| {
                                window.dispatch_action(Box::new(crate::app::OpenSettings), cx);
                            }),
                    ),
            );

        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::on_close_settings))
            .occlude()
            .w(width)
            .rounded(cx.theme().radius_lg)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .shadow_lg()
            .overflow_hidden()
            .child(
                div()
                    .id("crawl-settings")
                    .max_h(max_height)
                    .overflow_y_scrollbar()
                    .child(body),
            )
    }
}

impl Render for CrawlBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open = self.settings_open && !self.running;
        div()
            .id("crawl-bar")
            .track_focus(&self.focus_handle)
            .relative()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .child(self.render_main_row(cx))
            // Painted over the results rather than pushing them down, so the
            // grid does not jump. It stays open on a click outside, so a URL
            // can be copied from the grid into the tester.
            .child(
                canvas(
                    {
                        let bounds = self.bounds.clone();
                        move |painted, _, _| bounds.set(Some(painted))
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .when_some(self.bounds.get().filter(|_| open), |el, bar| {
                el.child(
                    deferred(
                        anchored()
                            .position(point(bar.left() + px(12.), bar.bottom()))
                            .child(self.render_panel(bar, window, cx)),
                    )
                    .with_priority(1),
                )
            })
    }
}
