use crate::app_database::AppDatabase;
use crate::app_settings::AppSettings;
use crate::settings::Settings;
use gpui::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render, SharedString,
    Styled, Subscription, Window, px, rems,
};
use gpui_component::ThemeRegistry;
use gpui_component::{
    ActiveTheme, Sizable, Theme,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    setting::{
        NumberFieldOptions, RenderOptions, SettingField, SettingGroup, SettingItem, SettingPage,
        Settings as GpuiSettings,
    },
};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct NumberFieldState {
    input: Entity<InputState>,
    last_value: f64,
    _subscriptions: Vec<Subscription>,
}

/// Build a number setting field that persists on BOTH typing and +/- stepper clicks.
///
/// gpui-component's built-in `SettingField::number_input` only persists on typed
/// changes: its stepper handler updates the input text via `InputState::set_value`,
/// which suppresses the `Change` event the persistence path listens for. This
/// reimplementation wires the `Step` event straight to the setter so the stepper
/// persists too.
fn number_field<V, S>(
    options: NumberFieldOptions,
    value: V,
    set_value: S,
) -> SettingField<SharedString>
where
    V: Fn(&App) -> f64 + 'static,
    S: Fn(f64, &mut App) + 'static,
{
    let value = Rc::new(value);
    let set_value: Rc<dyn Fn(f64, &mut App)> = Rc::new(set_value);

    SettingField::render(
        move |render_options: &RenderOptions, window: &mut Window, cx: &mut App| {
            let current = value(cx);
            let key = SharedString::from(format!(
                "sr-number-{}-{}-{}",
                render_options.page_ix, render_options.group_ix, render_options.item_ix
            ));
            let options = options.clone();
            let set_value = set_value.clone();

            let state = window.use_keyed_state(key, cx, move |window, cx| {
                let input =
                    cx.new(|cx| InputState::new(window, cx).default_value(current.to_string()));

                let step_sub = cx.subscribe_in(&input, window, {
                    let options = options.clone();
                    let set_value = set_value.clone();
                    move |_state: &mut NumberFieldState,
                          input,
                          event: &NumberInputEvent,
                          window,
                          cx| {
                        let NumberInputEvent::Step(action) = event;
                        input.update(cx, |input, cx| {
                            let Ok(current) = input.value().parse::<f64>() else {
                                return;
                            };
                            let stepped = if *action == StepAction::Increment {
                                current + options.step
                            } else {
                                current - options.step
                            };
                            let clamped = stepped.clamp(options.min, options.max);
                            set_value(clamped, cx);
                            input.set_value(SharedString::from(clamped.to_string()), window, cx);
                        });
                    }
                });

                let change_sub = cx.subscribe_in(&input, window, {
                    let options = options.clone();
                    let set_value = set_value.clone();
                    move |state: &mut NumberFieldState, input, event: &InputEvent, window, cx| {
                        if !matches!(event, InputEvent::Change) {
                            return;
                        }
                        input.update(cx, |input, cx| {
                            let Ok(parsed) = input.value().parse::<f64>() else {
                                return;
                            };
                            let clamped = parsed.clamp(options.min, options.max);
                            if (clamped - state.last_value).abs() < f64::EPSILON {
                                return;
                            }
                            set_value(clamped, cx);
                            state.last_value = clamped;
                            if (clamped - parsed).abs() > f64::EPSILON {
                                input.set_value(
                                    SharedString::from(clamped.to_string()),
                                    window,
                                    cx,
                                );
                            }
                        });
                    }
                });

                NumberFieldState {
                    input,
                    last_value: current,
                    _subscriptions: vec![step_sub, change_sub],
                }
            });

            NumberInput::new(&state.read(cx).input).with_size(render_options.size)
        },
    )
}

#[derive(Debug, Clone)]
struct FontOption {
    value: String,
    label: String,
}

impl SelectItem for FontOption {
    type Value = String;

    fn title(&self) -> SharedString {
        SharedString::from(self.label.clone())
    }

    fn value(&self) -> &String {
        &self.value
    }

    fn matches(&self, query: &str) -> bool {
        self.label.to_lowercase().contains(&query.to_lowercase())
    }
}

pub struct SettingsView {
    focus_handle: FocusHandle,
    save_generations: Arc<Mutex<std::collections::HashMap<String, u64>>>,
    ui_font_select: Option<Entity<SelectState<SearchableVec<FontOption>>>>,
    mono_font_select: Option<Entity<SelectState<SearchableVec<FontOption>>>>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            save_generations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            ui_font_select: None,
            mono_font_select: None,
            _subscriptions: Vec::new(),
        }
    }

    fn ensure_font_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.ui_font_select.is_some() {
            return;
        }

        let font_names = cx.text_system().all_font_names();

        let build_options = |current_value: &str| -> (SearchableVec<FontOption>, Option<usize>) {
            let options: Vec<FontOption> = font_names
                .iter()
                .map(|name| FontOption {
                    value: name.clone(),
                    label: name.clone(),
                })
                .collect();

            let selected_index = if current_value.is_empty() {
                None
            } else {
                options.iter().position(|o| o.value == current_value)
            };

            (SearchableVec::new(options), selected_index)
        };

        let settings = &AppSettings::global(cx).settings.appearance;
        let (ui_items, ui_index) = build_options(&settings.font_family);
        let (mono_items, mono_index) = build_options(&settings.mono_font_family);

        let ui_font_select = cx.new(|cx| {
            SelectState::new(
                ui_items,
                ui_index.map(|i| gpui_component::IndexPath::default().row(i)),
                window,
                cx,
            )
            .searchable(true)
        });

        let mono_font_select = cx.new(|cx| {
            SelectState::new(
                mono_items,
                mono_index.map(|i| gpui_component::IndexPath::default().row(i)),
                window,
                cx,
            )
            .searchable(true)
        });

        let sub1 = cx.subscribe_in(&ui_font_select, window, Self::on_ui_font_selected);
        let sub2 = cx.subscribe_in(&mono_font_select, window, Self::on_mono_font_selected);

        self.ui_font_select = Some(ui_font_select);
        self.mono_font_select = Some(mono_font_select);
        self._subscriptions.push(sub1);
        self._subscriptions.push(sub2);
    }

    fn on_ui_font_selected(
        &mut self,
        _: &Entity<SelectState<SearchableVec<FontOption>>>,
        event: &SelectEvent<SearchableVec<FontOption>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        let font_name = value.clone().unwrap_or_default();
        AppSettings::global_mut(cx).settings.appearance.font_family = font_name.clone();
        crate::settings::apply_font_settings(cx);
        self.save_setting_debounced("appearance.font_family".to_string(), font_name, cx);
    }

    fn on_mono_font_selected(
        &mut self,
        _: &Entity<SelectState<SearchableVec<FontOption>>>,
        event: &SelectEvent<SearchableVec<FontOption>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(value) = event;
        let font_name = value.clone().unwrap_or_default();
        AppSettings::global_mut(cx)
            .settings
            .appearance
            .mono_font_family = font_name.clone();
        crate::settings::apply_font_settings(cx);
        self.save_setting_debounced("appearance.mono_font_family".to_string(), font_name, cx);
    }

    fn save_setting_debounced(&mut self, key: String, value: String, cx: &mut Context<Self>) {
        let generation = {
            let mut gens = self.save_generations.lock().unwrap();
            let generation = gens.entry(key.clone()).or_insert(0);
            *generation += 1;
            *generation
        };

        let db = AppDatabase::global(cx).clone();
        let key_clone = key;
        let generations = self.save_generations.clone();

        let task = cx.spawn(async move |_, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;

            let current_gen = {
                let gens = generations.lock().unwrap();
                gens.get(&key_clone).copied().unwrap_or(0)
            };

            if current_gen == generation
                && let Err(e) = db.save_setting(&key_clone, &value).await
            {
                tracing::error!("Failed to save setting {}: {}", key_clone, e);
            }
        });
        task.detach();
    }

    fn setting_pages(&mut self, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let default_settings = Settings::default();
        let view_handle = cx.entity().downgrade();

        let sorted_themes = ThemeRegistry::global(cx)
            .sorted_themes()
            .into_iter()
            .map(|v| (v.name.clone(), v.name.clone()))
            .collect();

        let ui_font_select = self.ui_font_select.clone();
        let mono_font_select = self.mono_font_select.clone();

        vec![
            SettingPage::new("General").resettable(true).groups(vec![
                SettingGroup::new().title("Updates").items(vec![
                    SettingItem::new(
                        "Update Automatically",
                        SettingField::switch(
                            move |cx: &App| {
                                AppSettings::global(cx).settings.general.check_for_updates
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx)
                                        .settings
                                        .general
                                        .check_for_updates = val;

                                    let key = "general.check_for_updates".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                        .default_value(default_settings.general.check_for_updates),
                    )
                    .description("Automatically check for application updates."),
                ]),
            ]),
            SettingPage::new("Crawl").resettable(true).groups(vec![
                SettingGroup::new().title("Limits").items(vec![
                    SettingItem::new(
                        "Max Pages",
                        number_field(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 1000000.0,
                                step: 100.0,
                            },
                            move |cx: &App| AppSettings::global(cx).settings.crawl.max_pages as f64,
                            {
                                let view_handle = view_handle.clone();
                                move |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.max_pages =
                                        val as u32;

                                    let key = "crawl.max_pages".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                    )
                    .description("Maximum pages to crawl (0 = unlimited)."),
                    SettingItem::new(
                        "Max Concurrent Requests",
                        number_field(
                            NumberFieldOptions {
                                min: 1.0,
                                max: 100.0,
                                step: 1.0,
                            },
                            move |cx: &App| {
                                AppSettings::global(cx).settings.crawl.max_concurrent as f64
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.max_concurrent =
                                        val as u32;

                                    let key = "crawl.max_concurrent".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                    )
                    .description(
                        "Maximum concurrent requests (HTTP) or Chrome tabs (Chrome mode).",
                    ),
                    SettingItem::new(
                        "Request Timeout (seconds)",
                        number_field(
                            NumberFieldOptions {
                                min: 5.0,
                                max: 300.0,
                                step: 5.0,
                            },
                            move |cx: &App| {
                                AppSettings::global(cx).settings.crawl.timeout_seconds as f64
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.timeout_seconds =
                                        val as u32;

                                    let key = "crawl.timeout_seconds".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                    )
                    .description("HTTP request timeout in seconds."),
                ]),
                SettingGroup::new().title("Rate Limiting").items(vec![
                    SettingItem::new(
                        "Delay Between Requests (ms)",
                        number_field(
                            NumberFieldOptions {
                                min: 0.0,
                                max: 60000.0,
                                step: 100.0,
                            },
                            move |cx: &App| AppSettings::global(cx).settings.crawl.delay_ms as f64,
                            {
                                let view_handle = view_handle.clone();
                                move |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.delay_ms =
                                        val as u32;

                                    let key = "crawl.delay_ms".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                    )
                    .description("Delay in milliseconds between requests (0 = no delay)."),
                ]),
                SettingGroup::new().title("Behavior").items(vec![
                    SettingItem::new(
                        "Respect robots.txt",
                        SettingField::switch(
                            move |cx: &App| {
                                AppSettings::global(cx).settings.crawl.respect_robots_txt
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx)
                                        .settings
                                        .crawl
                                        .respect_robots_txt = val;

                                    let key = "crawl.respect_robots_txt".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                        .default_value(default_settings.crawl.respect_robots_txt),
                    )
                    .description("Obey robots.txt directives."),
                    SettingItem::new(
                        "Follow Sitemaps",
                        SettingField::switch(
                            move |cx: &App| AppSettings::global(cx).settings.crawl.follow_sitemaps,
                            {
                                let view_handle = view_handle.clone();
                                move |val: bool, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.follow_sitemaps =
                                        val;

                                    let key = "crawl.follow_sitemaps".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                        .default_value(default_settings.crawl.follow_sitemaps),
                    )
                    .description("Discover URLs from XML sitemaps."),
                    SettingItem::new(
                        "Near Duplicate Threshold (%)",
                        number_field(
                            NumberFieldOptions {
                                min: 50.0,
                                max: 100.0,
                                step: 5.0,
                            },
                            move |cx: &App| {
                                AppSettings::global(cx)
                                    .settings
                                    .crawl
                                    .near_duplicate_threshold as f64
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: f64, cx: &mut App| {
                                    AppSettings::global_mut(cx)
                                        .settings
                                        .crawl
                                        .near_duplicate_threshold = val as u8;

                                    let key = "crawl.near_duplicate_threshold".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                    )
                    .description("Minimum similarity percentage to flag pages as near duplicates."),
                ]),
                SettingGroup::new().title("Content Analysis").items(vec![
                    SettingItem::new(
                        "Content Selector",
                        SettingField::input(
                            move |cx: &App| {
                                SharedString::from(AppSettings::global(cx).settings.crawl.content_selector.clone())
                            },
                            {
                                let view_handle = view_handle.clone();
                                move |val: SharedString, cx: &mut App| {
                                    AppSettings::global_mut(cx).settings.crawl.content_selector = val.to_string();

                                    let key = "crawl.content_selector".to_string();
                                    let value = val.to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, value, cx);
                                        });
                                    }
                                }
                            },
                        )
                        .default_value(SharedString::from("")),
                    )
                    .description("CSS selector to scope word count and duplicate detection. Leave empty to use full page body."),
                ]),
            ]),
            SettingPage::new("Appearance").resettable(true).groups(vec![
                SettingGroup::new().title("Theme").items(vec![
                    SettingItem::new(
                        "Theme",
                        SettingField::dropdown(
                            sorted_themes,
                            move |cx: &App| {
                                SharedString::from(
                                    AppSettings::global(cx).settings.appearance.theme.clone(),
                                )
                            },
                            {
                                let view_handle = view_handle;
                                move |val: SharedString, cx: &mut App| {
                                    let theme_name = val.to_string();
                                    AppSettings::global_mut(cx).settings.appearance.theme =
                                        theme_name.clone();

                                    if let Some(theme_config) =
                                        ThemeRegistry::global(cx).themes().get(&val).cloned()
                                    {
                                        Theme::global_mut(cx).apply_config(&theme_config);
                                        crate::settings::apply_font_settings(cx);
                                    }

                                    let key = "appearance.theme".to_string();
                                    if let Some(view) = view_handle.upgrade() {
                                        view.update(cx, |view, cx| {
                                            view.save_setting_debounced(key, theme_name, cx);
                                        });
                                    }
                                }
                            },
                        )
                        .default_value(SharedString::from(
                            default_settings.appearance.theme,
                        )),
                    )
                    .description("Choose the color theme for the application."),
                ]),
                SettingGroup::new().title("Fonts").items(vec![
                    SettingItem::new(
                        "UI Font",
                        SettingField::render({
                            let ui_font_select = ui_font_select;
                            move |options, _window, cx| {
                                let theme_font = cx.theme().font_family.to_string();
                                if let Some(state) = &ui_font_select {
                                    Select::new(state)
                                        .with_size(options.size)
                                        .placeholder(format!("{theme_font} (theme default)"))
                                        .search_placeholder("Search fonts...")
                                        .cleanable(true)
                                        .max_h(rems(16.))
                                        .w(px(240.))
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            }
                        }),
                    )
                    .description("Font used for the application UI."),
                    SettingItem::new(
                        "Monospace Font",
                        SettingField::render({
                            let mono_font_select = mono_font_select;
                            move |options, _window, cx| {
                                let theme_font = cx.theme().mono_font_family.to_string();
                                if let Some(state) = &mono_font_select {
                                    Select::new(state)
                                        .with_size(options.size)
                                        .placeholder(format!("{theme_font} (theme default)"))
                                        .search_placeholder("Search fonts...")
                                        .cleanable(true)
                                        .max_h(rems(16.))
                                        .w(px(240.))
                                        .into_any_element()
                                } else {
                                    gpui::div().into_any_element()
                                }
                            }
                        }),
                    )
                    .description("Monospace font used for URLs and data."),
                ]),
            ]),
        ]
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_font_selects(window, cx);

        GpuiSettings::new("shoutingrobin-settings").pages(self.setting_pages(cx))
    }
}
