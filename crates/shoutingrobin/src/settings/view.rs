use crate::app_database::AppDatabase;
use crate::app_settings::AppSettings;
use crate::settings::Settings;
use gpui_kit::component::ThemeRegistry;
use gpui_kit::component::{
    ActiveTheme, Sizable,
    select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState},
    setting::{
        NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage,
        Settings as GpuiSettings,
    },
};
use gpui_kit::{
    App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, px, rems,
};
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
    theme_browser: Option<Entity<vsx_themes::ThemeBrowser>>,
    _subscriptions: Vec<gpui_kit::Subscription>,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            save_generations: Arc::new(Mutex::new(std::collections::HashMap::new())),
            ui_font_select: None,
            mono_font_select: None,
            theme_browser: None,
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
                ui_index.map(|i| gpui_kit::component::IndexPath::default().row(i)),
                window,
                cx,
            )
            .searchable(true)
        });

        let mono_font_select = cx.new(|cx| {
            SelectState::new(
                mono_items,
                mono_index.map(|i| gpui_kit::component::IndexPath::default().row(i)),
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
            let mut gens = self
                .save_generations
                .lock()
                .unwrap_or_else(|e| e.into_inner());
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
                let gens = generations.lock().unwrap_or_else(|e| e.into_inner());
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
        let system_name = crate::locale::LOCALES
            .iter()
            .find(|(code, _)| *code == crate::locale::system_locale())
            .map(|(_, name)| *name)
            .unwrap_or_default();
        let locale_options = std::iter::once((
            SharedString::default(),
            SharedString::from(t!("settings.language_system", language = system_name)),
        ))
        .chain(
            crate::locale::LOCALES
                .iter()
                .map(|(code, name)| (SharedString::from(*code), SharedString::from(*name))),
        )
        .collect();
        let mono_font_select = self.mono_font_select.clone();

        vec![
            SettingPage::new(t!("settings.page.general"))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(t!("settings.group.language"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.language"),
                                SettingField::dropdown(
                                    locale_options,
                                    move |cx: &App| {
                                        SharedString::from(
                                            AppSettings::global(cx).settings.general.locale.clone(),
                                        )
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: SharedString, cx: &mut App| {
                                            let locale = val.to_string();
                                            AppSettings::global_mut(cx).settings.general.locale =
                                                locale.clone();
                                            crate::locale::change(&locale, cx);

                                            let key = "general.locale".to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, locale, cx);
                                                });
                                            }
                                        }
                                    },
                                )
                                .default_value(SharedString::from(
                                    default_settings.general.locale.clone(),
                                )),
                            )
                            .description(SharedString::from(t!("settings.language_description"))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.updates"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.check_for_updates"),
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
                            .description(SharedString::from(t!(
                                "settings.check_for_updates_description"
                            ))),
                        ]),
                ]),
            SettingPage::new(t!("settings.page.crawl"))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(t!("settings.group.limits"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.max_pages"),
                                SettingField::number_input(
                                    NumberFieldOptions {
                                        min: 0.0,
                                        max: 1000000.0,
                                        step: 100.0,
                                    },
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.max_pages as f64
                                    },
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
                                ),
                            )
                            .description(SharedString::from(t!("settings.max_pages_description"))),
                            SettingItem::new(
                                t!("settings.max_concurrent"),
                                SettingField::number_input(
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
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .max_concurrent = val as u32;

                                            let key = "crawl.max_concurrent".to_string();
                                            let value = val.to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, value, cx);
                                                });
                                            }
                                        }
                                    },
                                ),
                            )
                            .description(SharedString::from(t!(
                                "settings.max_concurrent_description"
                            ))),
                            SettingItem::new(
                                t!("settings.timeout"),
                                SettingField::number_input(
                                    NumberFieldOptions {
                                        min: 5.0,
                                        max: 300.0,
                                        step: 5.0,
                                    },
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.timeout_seconds
                                            as f64
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: f64, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .timeout_seconds = val as u32;

                                            let key = "crawl.timeout_seconds".to_string();
                                            let value = val.to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, value, cx);
                                                });
                                            }
                                        }
                                    },
                                ),
                            )
                            .description(SharedString::from(t!("settings.timeout_description"))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.rate_limiting"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.delay"),
                                SettingField::number_input(
                                    NumberFieldOptions {
                                        min: 0.0,
                                        max: 60000.0,
                                        step: 100.0,
                                    },
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.delay_ms as f64
                                    },
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
                                ),
                            )
                            .description(SharedString::from(t!("settings.delay_description"))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.behavior"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.respect_robots_txt"),
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
                            .description(SharedString::from(t!(
                                "settings.respect_robots_txt_description"
                            ))),
                            SettingItem::new(
                                t!("settings.follow_sitemaps"),
                                SettingField::switch(
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.follow_sitemaps
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: bool, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .follow_sitemaps = val;

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
                            .description(SharedString::from(t!(
                                "settings.follow_sitemaps_description"
                            ))),
                            SettingItem::new(
                                t!("settings.block_images"),
                                SettingField::switch(
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.block_images
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: bool, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .block_images = val;

                                            let key = "crawl.block_images".to_string();
                                            let value = val.to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, value, cx);
                                                });
                                            }
                                        }
                                    },
                                )
                                .default_value(default_settings.crawl.block_images),
                            )
                            .description(SharedString::from(t!(
                                "settings.block_images_description"
                            ))),
                            SettingItem::new(
                                t!("settings.capture_layout"),
                                SettingField::switch(
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.capture_layout
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: bool, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .capture_layout = val;

                                            let key = "crawl.capture_layout".to_string();
                                            let value = val.to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, value, cx);
                                                });
                                            }
                                        }
                                    },
                                )
                                .default_value(default_settings.crawl.capture_layout),
                            )
                            .description(SharedString::from(t!(
                                "settings.capture_layout_description"
                            ))),
                            SettingItem::new(
                                t!("settings.scroll_for_layout"),
                                SettingField::switch(
                                    move |cx: &App| {
                                        AppSettings::global(cx).settings.crawl.scroll_for_layout
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: bool, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .scroll_for_layout = val;

                                            let key = "crawl.scroll_for_layout".to_string();
                                            let value = val.to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(key, value, cx);
                                                });
                                            }
                                        }
                                    },
                                )
                                .default_value(default_settings.crawl.scroll_for_layout),
                            )
                            .description(SharedString::from(t!(
                                "settings.scroll_for_layout_description"
                            ))),
                            SettingItem::new(
                                t!("settings.near_duplicate_threshold"),
                                SettingField::number_input(
                                    NumberFieldOptions {
                                        min: 50.0,
                                        max: 100.0,
                                        step: 5.0,
                                    },
                                    move |cx: &App| {
                                        AppSettings::global(cx)
                                            .settings
                                            .crawl
                                            .near_duplicate_threshold
                                            as f64
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
                                ),
                            )
                            .description(SharedString::from(t!(
                                "settings.near_duplicate_threshold_description"
                            ))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.content_analysis"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.content_selector"),
                                SettingField::input(
                                    move |cx: &App| {
                                        SharedString::from(
                                            AppSettings::global(cx)
                                                .settings
                                                .crawl
                                                .content_selector
                                                .clone(),
                                        )
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: SharedString, cx: &mut App| {
                                            AppSettings::global_mut(cx)
                                                .settings
                                                .crawl
                                                .content_selector = val.to_string();

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
                            .description(SharedString::from(t!(
                                "settings.content_selector_description"
                            ))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.identification"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.user_agent"),
                                SettingField::input(
                                    move |cx: &App| {
                                        SharedString::from(
                                            AppSettings::global(cx)
                                                .settings
                                                .crawl
                                                .user_agent
                                                .clone(),
                                        )
                                    },
                                    {
                                        let view_handle = view_handle.clone();
                                        move |val: SharedString, cx: &mut App| {
                                            AppSettings::global_mut(cx).settings.crawl.user_agent =
                                                val.to_string();

                                            let key = "crawl.user_agent".to_string();
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
                            .description(SharedString::from(t!("settings.user_agent_description"))),
                        ]),
                ]),
            SettingPage::new(t!("settings.page.appearance"))
                .resettable(true)
                .groups(vec![
                    SettingGroup::new()
                        .title(t!("settings.group.theme"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.theme"),
                                SettingField::dropdown(
                                    sorted_themes,
                                    move |cx: &App| {
                                        SharedString::from(
                                            AppSettings::global(cx)
                                                .settings
                                                .appearance
                                                .theme
                                                .clone(),
                                        )
                                    },
                                    {
                                        let view_handle = view_handle;
                                        move |val: SharedString, cx: &mut App| {
                                            let theme_name = val.to_string();
                                            AppSettings::global_mut(cx).settings.appearance.theme =
                                                theme_name.clone();

                                            if let Some(theme_config) = ThemeRegistry::global(cx)
                                                .themes()
                                                .get(&val)
                                                .cloned()
                                            {
                                                crate::settings::apply_theme_config(
                                                    &theme_config,
                                                    cx,
                                                );
                                            }

                                            let key = "appearance.theme".to_string();
                                            if let Some(view) = view_handle.upgrade() {
                                                view.update(cx, |view, cx| {
                                                    view.save_setting_debounced(
                                                        key, theme_name, cx,
                                                    );
                                                });
                                            }
                                        }
                                    },
                                )
                                .default_value(SharedString::from(
                                    default_settings.appearance.theme,
                                )),
                            )
                            .description(SharedString::from(t!("settings.theme_description"))),
                        ]),
                    SettingGroup::new()
                        .title(t!("settings.group.fonts"))
                        .items(vec![
                            SettingItem::new(
                                t!("settings.ui_font"),
                                SettingField::render({
                                    let ui_font_select = ui_font_select;
                                    move |options, _window, cx| {
                                        let theme_font = cx.theme().font_family.to_string();
                                        if let Some(state) = &ui_font_select {
                                            Select::new(state)
                                                .with_size(options.size())
                                                .placeholder(t!(
                                                    "settings.font_theme_default",
                                                    font = theme_font
                                                ))
                                                .search_placeholder(t!("settings.search_fonts"))
                                                .cleanable(true)
                                                .max_h(rems(16.))
                                                .w(px(240.))
                                                .into_any_element()
                                        } else {
                                            gpui_kit::div().into_any_element()
                                        }
                                    }
                                }),
                            )
                            .description(SharedString::from(t!("settings.ui_font_description"))),
                            SettingItem::new(
                                t!("settings.mono_font"),
                                SettingField::render({
                                    let mono_font_select = mono_font_select;
                                    move |options, _window, cx| {
                                        let theme_font = cx.theme().mono_font_family.to_string();
                                        if let Some(state) = &mono_font_select {
                                            Select::new(state)
                                                .with_size(options.size())
                                                .placeholder(t!(
                                                    "settings.font_theme_default",
                                                    font = theme_font
                                                ))
                                                .search_placeholder(t!("settings.search_fonts"))
                                                .cleanable(true)
                                                .max_h(rems(16.))
                                                .w(px(240.))
                                                .into_any_element()
                                        } else {
                                            gpui_kit::div().into_any_element()
                                        }
                                    }
                                }),
                            )
                            .description(SharedString::from(t!("settings.mono_font_description"))),
                        ]),
                    vsx_themes::browser::setting_group(
                        self.theme_browser
                            .clone()
                            .expect("created before the pages"),
                    ),
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
        if self.theme_browser.is_none() {
            let options = vsx_themes::ThemeBrowserOptions {
                // The folder `ThemeRegistry::watch_dir` loads in main.rs.
                themes_dir: crate::themes_manager::ThemesManager::themes_directory(),
                convert: vsx_themes::ConvertOptions::default(),
                use_theme: Some(std::rc::Rc::new(
                    |name: &str, _: &mut Window, cx: &mut App| {
                        crate::settings::use_theme(name, cx);
                    },
                )),
                watches_themes_dir: true,
            };
            self.theme_browser =
                Some(cx.new(|cx| vsx_themes::ThemeBrowser::new(options, window, cx)));
        }

        gpui_kit::div()
            .track_focus(&self.focus_handle)
            .size_full()
            .child(GpuiSettings::new("shoutingrobin-settings").pages(self.setting_pages(cx)))
    }
}
