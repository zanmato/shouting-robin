use gpui_kit::component as gpui_component;
use gpui_kit::{App, Global};
use std::sync::Once;

/// The locales the app is translated to, with each one's name in its own
/// language, which is how the Language setting lists them.
pub const LOCALES: &[(&str, &str)] = &[("en", "English"), ("sv", "Svenska")];

const FALLBACK: &str = "en";

/// Adds the app's translations of gpui-component's own strings, kept under
/// `gpui_component` in `locales/gpui_component.yml`. gpui-component ships no
/// Swedish, so without them its buttons and placeholders stay in English.
pub fn init() {
    static EXTEND: Once = Once::new();
    EXTEND.call_once(|| rust_i18n::extend!(gpui_component));
}

/// The locale a `general.locale` setting stands for: the setting itself when
/// it names a shipped locale, otherwise the system's.
pub fn resolve(setting: &str) -> &'static str {
    match_locale(setting).unwrap_or_else(system_locale)
}

/// The first of the system's preferred languages the app is translated to.
pub fn system_locale() -> &'static str {
    sys_locale::get_locales()
        .find_map(|locale| match_locale(&locale))
        .unwrap_or(FALLBACK)
}

/// The locale the app shows. Text is translated as it is rendered, so a
/// change shows at the next frame, but a view that keeps translated text in
/// its state, such as an input's placeholder, observes this global to make it
/// again.
pub struct Locale;

impl Global for Locale {}

/// Makes `setting` the locale of every string, the app's and gpui-component's.
pub fn apply(setting: &str, cx: &mut App) {
    let locale = resolve(setting);
    rust_i18n::set_locale(locale);
    cx.set_global(Locale);
}

/// Switches the locale while the app runs and redraws the windows in it.
pub fn change(setting: &str, cx: &mut App) {
    apply(setting, cx);
    cx.refresh_windows();
}

/// The shipped locale for a language tag such as `sv-SE`, `sv_SE.UTF-8` or
/// `en`, matching the whole tag first and then its language alone, so a
/// regional variant gets its language's translation.
fn match_locale(tag: &str) -> Option<&'static str> {
    let tag = tag
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('_', "-");
    if tag.is_empty() {
        return None;
    }
    let language = tag.split('-').next().unwrap_or_default();
    LOCALES
        .iter()
        .find(|(code, _)| code.eq_ignore_ascii_case(&tag))
        .or_else(|| {
            LOCALES
                .iter()
                .find(|(code, _)| code.eq_ignore_ascii_case(language))
        })
        .map(|(code, _)| *code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn language_tags_match_their_language() {
        assert_eq!(match_locale("sv-SE"), Some("sv"));
        assert_eq!(match_locale("sv_FI.UTF-8"), Some("sv"));
        assert_eq!(match_locale("en-US"), Some("en"));
        assert_eq!(match_locale("SV"), Some("sv"));
        assert_eq!(match_locale("de-DE"), None);
        assert_eq!(match_locale(""), None);
        assert_eq!(resolve("sv"), "sv");
        assert_eq!(resolve("en"), "en");
    }

    fn keys(backend: &dyn rust_i18n::Backend, locale: &str) -> BTreeSet<String> {
        backend
            .messages_for_locale(locale)
            .unwrap_or_default()
            .into_iter()
            .map(|(key, _)| key.into_owned())
            .collect()
    }

    /// A key missing from one locale shows in English, or as its key, there.
    #[test]
    fn every_locale_translates_every_key() {
        for (name, backend) in [
            ("shoutingrobin", crate::_rust_i18n_backend()),
            ("shoutingrobin-ui", shoutingrobin_ui::_rust_i18n_backend()),
            ("vsx-themes", vsx_themes::_rust_i18n_backend()),
        ] {
            // gpui-component's English is its own, see the test below.
            let ours = |locale| -> BTreeSet<String> {
                keys(backend, locale)
                    .into_iter()
                    .filter(|key| !key.starts_with("gpui_component."))
                    .collect()
            };
            let english = ours("en");
            assert!(!english.is_empty(), "{name} has no English strings");
            for (locale, _) in LOCALES {
                let translated = ours(locale);
                let missing: Vec<_> = english.difference(&translated).collect();
                let extra: Vec<_> = translated.difference(&english).collect();
                assert!(missing.is_empty(), "{name} lacks {locale} for {missing:?}");
                assert!(
                    extra.is_empty(),
                    "{name} has {locale} without English for {extra:?}"
                );
            }
        }
    }

    /// Every string gpui-component shows in English has a Swedish one here.
    #[test]
    fn gpui_component_strings_are_translated() {
        let english = keys(gpui_component::_rust_i18n_backend(), "en");
        let prefix = "gpui_component.";
        let ours: BTreeSet<String> = keys(crate::_rust_i18n_backend(), "sv")
            .into_iter()
            .filter_map(|key| key.strip_prefix(prefix).map(str::to_string))
            .collect();
        let missing: Vec<_> = english.difference(&ours).collect();
        assert!(
            missing.is_empty(),
            "gpui-component lacks sv for {missing:?}"
        );
    }
}
