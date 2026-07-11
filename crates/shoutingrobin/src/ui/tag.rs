use gpui::{App, Hsla, hsla};
use gpui_component::{ActiveTheme as _, Sizable as _, tag::Tag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Tone {
    Ok,
    Warn,
    Err,
    Info,
    Accent,
    Neutral,
}

/// Opaque (background, foreground, border) for a tone, pre-blended onto a solid
/// base so the chip no longer disappears into a selected tab's background, and
/// chosen for the active theme mode. Hues are kept stable across modes; only the
/// lightness/saturation shifts, giving a dark subdued chip in dark mode and a
/// pale solid tint with a dark foreground in light mode.
fn tone_colors(tone: Tone, cx: &App) -> (Hsla, Hsla, Hsla) {
    if cx.theme().mode.is_dark() {
        // Dark mode: low-lightness, desaturated backgrounds with bright
        // foregrounds (the subdued look the alpha-over-dark base had before).
        match tone {
            Tone::Ok => (
                hsla(150. / 360., 0.30, 0.18, 1.0),
                hsla(152. / 360., 0.50, 0.64, 1.0),
                hsla(150. / 360., 0.30, 0.26, 1.0),
            ),
            Tone::Warn => (
                hsla(38. / 360., 0.40, 0.20, 1.0),
                hsla(43. / 360., 0.70, 0.66, 1.0),
                hsla(38. / 360., 0.40, 0.28, 1.0),
            ),
            Tone::Err => (
                hsla(0. / 360., 0.35, 0.20, 1.0),
                hsla(0. / 360., 0.55, 0.72, 1.0),
                hsla(0. / 360., 0.35, 0.28, 1.0),
            ),
            Tone::Info => (
                hsla(199. / 360., 0.40, 0.20, 1.0),
                hsla(201. / 360., 0.60, 0.80, 1.0),
                hsla(199. / 360., 0.40, 0.28, 1.0),
            ),
            Tone::Accent => (
                hsla(20. / 360., 0.45, 0.22, 1.0),
                hsla(27. / 360., 0.70, 0.70, 1.0),
                hsla(20. / 360., 0.45, 0.30, 1.0),
            ),
            Tone::Neutral => (
                hsla(225. / 360., 0.40, 0.24, 1.0),
                hsla(213. / 360., 0.50, 0.84, 1.0),
                hsla(225. / 360., 0.40, 0.32, 1.0),
            ),
        }
    } else {
        // Light mode: pale solid tints with dark foregrounds.
        match tone {
            Tone::Ok => (
                hsla(150. / 360., 0.50, 0.90, 1.0),
                hsla(152. / 360., 0.55, 0.28, 1.0),
                hsla(150. / 360., 0.45, 0.80, 1.0),
            ),
            Tone::Warn => (
                hsla(38. / 360., 0.80, 0.92, 1.0),
                hsla(35. / 360., 0.70, 0.32, 1.0),
                hsla(38. / 360., 0.70, 0.84, 1.0),
            ),
            Tone::Err => (
                hsla(0. / 360., 0.70, 0.92, 1.0),
                hsla(0. / 360., 0.60, 0.38, 1.0),
                hsla(0. / 360., 0.65, 0.84, 1.0),
            ),
            Tone::Info => (
                hsla(199. / 360., 0.70, 0.90, 1.0),
                hsla(201. / 360., 0.60, 0.34, 1.0),
                hsla(199. / 360., 0.65, 0.82, 1.0),
            ),
            Tone::Accent => (
                hsla(20. / 360., 0.80, 0.92, 1.0),
                hsla(20. / 360., 0.70, 0.34, 1.0),
                hsla(20. / 360., 0.75, 0.84, 1.0),
            ),
            Tone::Neutral => (
                hsla(225. / 360., 0.45, 0.92, 1.0),
                hsla(225. / 360., 0.40, 0.32, 1.0),
                hsla(225. / 360., 0.40, 0.84, 1.0),
            ),
        }
    }
}

pub fn tone_tag(tone: Tone, cx: &App) -> Tag {
    let (bg, fg, border) = tone_colors(tone, cx);
    Tag::custom(bg, fg, border).small()
}

/// The foreground color of a tone, for use as plain colored text instead of a
/// full chip with background and border. Tracks the active theme mode so colored
/// text stays readable in light mode too.
pub fn tone_text_color(tone: Tone, cx: &App) -> Hsla {
    tone_colors(tone, cx).1
}

pub fn status_code_tone(code: u16) -> Tone {
    match code {
        200..=299 => Tone::Ok,
        300..=399 => Tone::Warn,
        400..=599 => Tone::Err,
        _ => Tone::Neutral,
    }
}

pub fn indexability_tone(value: &str) -> Tone {
    let lower = value.to_ascii_lowercase();
    if lower == "-" || lower.is_empty() || lower == "n/a" {
        Tone::Neutral
    } else if lower.contains("non")
        || lower.contains("noindex")
        || lower.contains("canonicalised")
        || lower.contains("canonicalized")
    {
        Tone::Warn
    } else {
        Tone::Ok
    }
}

pub fn count_tone(count: i64, severity: Tone) -> Tone {
    if count > 0 { severity } else { Tone::Ok }
}
