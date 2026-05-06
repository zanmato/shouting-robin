use gpui::{Hsla, hsla};
use gpui_component::{Sizable as _, tag::Tag};

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

fn tone_colors(tone: Tone) -> (Hsla, Hsla, Hsla) {
    // (background, foreground, border) — colors mirror the mockup palette:
    // dark, low-opacity background with a bright foreground for readability.
    match tone {
        // green-500 / green-300
        Tone::Ok => (
            hsla(160. / 360., 0.84, 0.39, 0.20),
            hsla(152. / 360., 0.76, 0.64, 1.0),
            hsla(160. / 360., 0.84, 0.39, 0.35),
        ),
        // amber-500 / amber-300
        Tone::Warn => (
            hsla(38. / 360., 0.92, 0.50, 0.20),
            hsla(43. / 360., 0.96, 0.66, 1.0),
            hsla(38. / 360., 0.92, 0.50, 0.35),
        ),
        // red-500 / red-300
        Tone::Err => (
            hsla(0. / 360., 0.84, 0.60, 0.20),
            hsla(0. / 360., 0.93, 0.78, 1.0),
            hsla(0. / 360., 0.84, 0.60, 0.35),
        ),
        // sky-400 / sky-200
        Tone::Info => (
            hsla(199. / 360., 0.92, 0.60, 0.20),
            hsla(201. / 360., 0.94, 0.86, 1.0),
            hsla(199. / 360., 0.92, 0.60, 0.35),
        ),
        // orange-500 / orange-300
        Tone::Accent => (
            hsla(20. / 360., 0.94, 0.53, 0.30),
            hsla(27. / 360., 0.96, 0.70, 1.0),
            hsla(20. / 360., 0.94, 0.53, 0.45),
        ),
        // blue chip: blue-800 bg / blue-200 fg, like mockup's `bg-chip text-chipfg`
        Tone::Neutral => (
            hsla(225. / 360., 0.64, 0.33, 1.0),
            hsla(213. / 360., 0.96, 0.87, 1.0),
            hsla(225. / 360., 0.64, 0.33, 1.0),
        ),
    }
}

pub fn tone_tag(tone: Tone) -> Tag {
    let (bg, fg, border) = tone_colors(tone);
    Tag::custom(bg, fg, border).small()
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
