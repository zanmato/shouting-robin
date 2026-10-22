pub mod view;

use crate::app_settings::AppSettings;
use gpui_kit::component::{Theme, ThemeConfig, ThemeRegistry};
use gpui_kit::{App, SharedString};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

/// Applies a theme config together with the user's font overrides.
pub fn apply_theme_config(theme_config: &Rc<ThemeConfig>, cx: &mut App) {
    let appearance = AppSettings::global(cx).settings.appearance.clone();

    // The font overrides are folded into the config rather than assigned to
    // the theme afterwards, because `Theme::change` re-applies the config and
    // would put the theme's own fonts back over them.
    let mut config = (**theme_config).clone();
    if !appearance.font_family.is_empty() {
        config.font_family = Some(SharedString::from(appearance.font_family));
    }
    if !appearance.mono_font_family.is_empty() {
        config.mono_font_family = Some(SharedString::from(appearance.mono_font_family));
    }

    let mode = config.mode;
    Theme::global_mut(cx).apply_config(&Rc::new(config));
    Theme::change(mode, None, cx);
    cx.refresh_windows();
}

/// Make the theme called `name` the app's theme and save the choice, as
/// picking it under Appearance does. Used by the VS Code Themes panel.
pub fn use_theme(name: &str, cx: &mut App) {
    let Some(theme_config) = ThemeRegistry::global(cx)
        .themes()
        .get(&SharedString::from(name.to_string()))
        .cloned()
    else {
        tracing::warn!("Unknown theme: {name}");
        return;
    };
    AppSettings::global_mut(cx).settings.appearance.theme = name.to_string();
    apply_theme_config(&theme_config, cx);

    let db = crate::app_database::AppDatabase::global(cx).clone();
    let name = name.to_string();
    cx.spawn(async move |_| {
        if let Err(error) = db.save_setting("appearance.theme", &name).await {
            tracing::error!("Failed to save setting appearance.theme: {error}");
        }
    })
    .detach();
}

pub fn apply_font_settings(cx: &mut App) {
    let appearance = AppSettings::global(cx).settings.appearance.clone();
    let theme_name = SharedString::from(appearance.theme.clone());

    if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
        apply_theme_config(&theme_config, cx);
        return;
    }

    // The saved theme is not registered (a renamed or deleted theme file), so
    // there is no config to fold the fonts into.
    if !appearance.font_family.is_empty() {
        Theme::global_mut(cx).font_family = SharedString::from(appearance.font_family);
    }
    if !appearance.mono_font_family.is_empty() {
        Theme::global_mut(cx).mono_font_family = SharedString::from(appearance.mono_font_family);
    }
    cx.refresh_windows();
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    pub general: GeneralSettings,
    pub crawl: CrawlSettings,
    pub appearance: AppearanceSettings,
}

impl Settings {
    pub fn from_key_values(values: &[(String, String)]) -> Self {
        let mut settings = Settings::default();

        for (key, value) in values {
            match key.as_str() {
                "general.check_for_updates" => {
                    settings.general.check_for_updates = value.parse().unwrap_or(true);
                }
                "crawl.max_pages" => {
                    settings.crawl.max_pages = value.parse().unwrap_or(0);
                }
                "crawl.max_concurrent" => {
                    settings.crawl.max_concurrent = value.parse().unwrap_or(5);
                }
                "crawl.delay_ms" => {
                    settings.crawl.delay_ms = value.parse().unwrap_or(0);
                }
                "crawl.timeout_seconds" => {
                    settings.crawl.timeout_seconds = value.parse().unwrap_or(30);
                }
                "crawl.respect_robots_txt" => {
                    settings.crawl.respect_robots_txt = value.parse().unwrap_or(true);
                }
                "crawl.follow_sitemaps" => {
                    settings.crawl.follow_sitemaps = value.parse().unwrap_or(true);
                }
                "crawl.block_images" => {
                    settings.crawl.block_images = value.parse().unwrap_or(false);
                }
                "crawl.capture_layout" => {
                    settings.crawl.capture_layout = value.parse().unwrap_or(false);
                }
                "crawl.scroll_for_layout" => {
                    settings.crawl.scroll_for_layout = value.parse().unwrap_or(false);
                }
                "crawl.near_duplicate_threshold" => {
                    settings.crawl.near_duplicate_threshold =
                        value.parse().unwrap_or(90).clamp(50, 100);
                }
                "crawl.content_selector" => {
                    settings.crawl.content_selector = value.clone();
                }
                "crawl.user_agent" => {
                    settings.crawl.user_agent = value.clone();
                }
                "appearance.theme" => {
                    settings.appearance.theme = value.clone();
                }
                "appearance.font_family" => {
                    settings.appearance.font_family = value.clone();
                }
                "appearance.mono_font_family" => {
                    settings.appearance.mono_font_family = value.clone();
                }
                _ => {}
            }
        }

        settings
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub check_for_updates: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            check_for_updates: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlSettings {
    pub max_pages: u32,
    pub max_concurrent: u32,
    pub delay_ms: u32,
    pub timeout_seconds: u32,
    pub respect_robots_txt: bool,
    pub follow_sitemaps: bool,
    #[serde(default)]
    pub block_images: bool,
    /// In Chrome mode, record the boxes each page renders.
    #[serde(default)]
    pub capture_layout: bool,
    /// Scroll through each page before recording its layout. Only read while
    /// `capture_layout` is on.
    #[serde(default)]
    pub scroll_for_layout: bool,
    pub near_duplicate_threshold: u8,
    pub content_selector: String,
    #[serde(default)]
    pub user_agent: String,
}

impl Default for CrawlSettings {
    fn default() -> Self {
        Self {
            max_pages: 0,
            max_concurrent: 5,
            delay_ms: 0,
            timeout_seconds: 30,
            respect_robots_txt: true,
            follow_sitemaps: true,
            block_images: false,
            capture_layout: false,
            scroll_for_layout: false,
            near_duplicate_threshold: 90,
            content_selector: String::new(),
            user_agent: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    pub theme: String,
    pub font_family: String,
    pub mono_font_family: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "Catppuccin Macchiato".to_string(),
            font_family: String::new(),
            mono_font_family: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;

    #[test]
    fn layout_switches_default_off_and_load_from_their_keys() {
        let defaults = Settings::from_key_values(&[]);
        assert!(!defaults.crawl.capture_layout);
        assert!(!defaults.crawl.scroll_for_layout);

        let loaded = Settings::from_key_values(&[
            ("crawl.capture_layout".into(), "true".into()),
            ("crawl.scroll_for_layout".into(), "true".into()),
        ]);
        assert!(loaded.crawl.capture_layout);
        assert!(loaded.crawl.scroll_for_layout);
    }
}
