pub mod view;

use crate::app_settings::AppSettings;
use gpui::{App, SharedString};
use gpui_component::Theme;
use serde::{Deserialize, Serialize};

pub fn apply_font_settings(cx: &mut App) {
    let appearance = AppSettings::global(cx).settings.appearance.clone();
    if !appearance.font_family.is_empty() {
        Theme::global_mut(cx).font_family = SharedString::from(appearance.font_family);
    }
    if !appearance.mono_font_family.is_empty() {
        Theme::global_mut(cx).mono_font_family = SharedString::from(appearance.mono_font_family);
    }
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
                    settings.crawl.max_concurrent = value.parse().unwrap_or(10);
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
    pub near_duplicate_threshold: u8,
    pub content_selector: String,
    #[serde(default)]
    pub user_agent: String,
}

impl Default for CrawlSettings {
    fn default() -> Self {
        Self {
            max_pages: 0,
            max_concurrent: 10,
            delay_ms: 0,
            timeout_seconds: 30,
            respect_robots_txt: true,
            follow_sitemaps: true,
            block_images: false,
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
