//! Text width in pixels, as a search engine result page renders it.
//!
//! Google sets titles in Arial at 20px and descriptions at 14px on desktop, and
//! truncates on rendered width rather than character count, which is why the
//! over/under-pixel-width rules exist at all. Measuring therefore needs the real
//! advance width of each glyph: the previous 4/8/10px bucket approximation
//! understated widths by roughly 27%, consistently enough that every
//! pixel-width rule fired on the wrong pages.
//!
//! The tables below are Arial's advance widths in font units (2048 per em),
//! read straight out of the font. Widths are summed in font units and converted
//! once at the end, so rounding happens a single time rather than per glyph.
//!
//! The remaining error against a rendered page is per-glyph rounding in the
//! rasteriser, which no advance-width table can reproduce. It averages well
//! under 2% and is widest in relative terms on very short strings, where a few
//! pixels is a large share; both are far below the granularity of the
//! thresholds this feeds.

/// Arial's em square. Every table entry is in these units.
const UNITS_PER_EM: u32 = 2048;

/// Google's desktop SERP title size.
pub const TITLE_FONT_SIZE_PX: f32 = 20.0;

/// Google's desktop SERP description size.
pub const META_DESCRIPTION_FONT_SIZE_PX: f32 = 14.0;

/// U+0020 (space) through U+007E (tilde).
static ASCII_ADVANCES: [u16; 95] = [
    569, 569, 727, 1139, 1139, 1821, 1366, 391, 682, 682, 797, 1196, 569, 682, 569, 569, 1139,
    1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 569, 569, 1196, 1196, 1196, 1139, 2079,
    1366, 1366, 1479, 1479, 1366, 1251, 1593, 1479, 569, 1024, 1366, 1139, 1706, 1479, 1593, 1366,
    1593, 1479, 1366, 1251, 1479, 1366, 1933, 1366, 1366, 1251, 569, 569, 569, 961, 1139, 682,
    1139, 1139, 1024, 1139, 1139, 569, 1139, 1139, 455, 455, 1024, 455, 1706, 1139, 1139, 1139,
    1139, 682, 1024, 569, 1139, 1024, 1479, 1024, 1024, 1024, 684, 532, 684, 1196,
];

/// U+00A0 (no-break space) through U+00FF, which covers the accented letters of
/// the Western European languages.
static LATIN1_ADVANCES: [u16; 96] = [
    569, 682, 1139, 1139, 1139, 1139, 532, 1139, 682, 1509, 758, 1139, 1196, 682, 1509, 1131, 819,
    1124, 682, 682, 682, 1180, 1100, 569, 682, 682, 748, 1139, 1708, 1708, 1708, 1251, 1366, 1366,
    1366, 1366, 1366, 1366, 2048, 1479, 1366, 1366, 1366, 1366, 569, 569, 569, 569, 1479, 1479,
    1593, 1593, 1593, 1593, 1593, 1196, 1593, 1479, 1479, 1479, 1479, 1366, 1366, 1251, 1139, 1139,
    1139, 1139, 1139, 1139, 1821, 1024, 1139, 1139, 1139, 1139, 569, 569, 569, 569, 1139, 1139,
    1139, 1139, 1139, 1139, 1139, 1124, 1251, 1139, 1139, 1139, 1139, 1024, 1139, 1024,
];

/// Characters outside those two blocks that turn up in titles often enough to be
/// worth a real width: typographic quotes and dashes, the ellipsis, currency and
/// the trademark sign.
static EXTRA_ADVANCES: &[(char, u16)] = &[
    ('\u{152}', 2048),  // 'Œ'
    ('\u{153}', 1933),  // 'œ'
    ('\u{160}', 1366),  // 'Š'
    ('\u{161}', 1024),  // 'š'
    ('\u{178}', 1366),  // 'Ÿ'
    ('\u{17D}', 1251),  // 'Ž'
    ('\u{17E}', 1024),  // 'ž'
    ('\u{192}', 1139),  // 'ƒ'
    ('\u{2C6}', 682),   // 'ˆ'
    ('\u{2DC}', 682),   // '˜'
    ('\u{2013}', 1139), // '–'
    ('\u{2014}', 2048), // '—'
    ('\u{2018}', 455),  // '‘'
    ('\u{2019}', 455),  // '’'
    ('\u{201A}', 455),  // '‚'
    ('\u{201C}', 682),  // '“'
    ('\u{201D}', 682),  // '”'
    ('\u{201E}', 682),  // '„'
    ('\u{2020}', 1139), // '†'
    ('\u{2021}', 1139), // '‡'
    ('\u{2022}', 717),  // '•'
    ('\u{2026}', 2048), // '…'
    ('\u{2030}', 2048), // '‰'
    ('\u{2039}', 682),  // '‹'
    ('\u{203A}', 682),  // '›'
    ('\u{20AC}', 1139), // '€'
    ('\u{2122}', 2048), // '™'
    ('\u{2212}', 1196), // '−'
];

/// Used for characters not in the tables. `n` is close to the average width of a
/// lowercase Latin letter, so an unlisted accented letter lands near the mark.
const FALLBACK_ADVANCE: u16 = 1139;

/// CJK and fullwidth characters occupy the full em square. Arial has no glyphs
/// for them at all, so a browser falls back to a font where they do, and
/// charging them a Latin letter's width would understate an Asian title by
/// roughly half.
const FULL_WIDTH_ADVANCE: u16 = 2048;

/// The rendered width of `text` in pixels at `font_size_px`.
pub fn text_pixel_width(text: &str, font_size_px: f32) -> u32 {
    let units: u64 = text.chars().map(|ch| u64::from(advance_units(ch))).sum();
    let pixels = units as f64 * f64::from(font_size_px) / f64::from(UNITS_PER_EM);
    pixels.round() as u32
}

fn advance_units(ch: char) -> u16 {
    let code = ch as u32;
    if (0x20..0x7F).contains(&code) {
        // The subtraction and index are both in range for this arm.
        return ASCII_ADVANCES[(code - 0x20) as usize];
    }
    if (0xA0..0x100).contains(&code) {
        return LATIN1_ADVANCES[(code - 0xA0) as usize];
    }
    if let Some((_, advance)) = EXTRA_ADVANCES
        .iter()
        .find(|(candidate, _)| *candidate == ch)
    {
        return *advance;
    }
    // Control characters and the zero-width formatting marks render as nothing.
    if code < 0x20 || matches!(code, 0x200B..=0x200F | 0x2060 | 0xFEFF) {
        return 0;
    }
    if is_full_width(code) {
        return FULL_WIDTH_ADVANCE;
    }
    FALLBACK_ADVANCE
}

/// The East Asian Wide and Fullwidth blocks, coarsely: Hangul Jamo, the CJK
/// blocks and their compatibility forms, and the fullwidth ASCII forms.
fn is_full_width(code: u32) -> bool {
    matches!(
        code,
        0x1100..=0x115F
            | 0x2E80..=0x303E
            | 0x3041..=0x33FF
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xA000..=0xA4CF
            | 0xAC00..=0xD7A3
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE6F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
            | 0x20000..=0x3FFFD
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real titles paired with their known rendered width at the SERP title
    /// size.
    const TITLE_SAMPLES: &[(&str, u32)] = &[
        ("Kvalitetsbett för dig och din häst | ByLynga", 379),
        ("Boss Läder Mini Pelham 125mm | ByLynga", 379),
        ("Touch Läder Lösa ringar 145mm | ByLynga", 379),
        ("Ångra ditt köp | ByLynga", 214),
        ("Bett | ByLynga", 126),
    ];

    /// The same, for real meta descriptions at the smaller SERP size.
    const META_SAMPLES: &[(&str, u32)] = &[
        ("Bett från ByLynga. Alltid snabba leveranser!", 272),
        (
            "Bett med lösa ringar från ByLynga. Alltid snabba leveranser!",
            371,
        ),
        ("Baucher bett från ByLynga. Alltid snabba leveranser!", 326),
    ];

    fn error_pct(measured: u32, expected: u32) -> f64 {
        (f64::from(measured) - f64::from(expected)).abs() / f64::from(expected) * 100.0
    }

    #[test]
    fn title_widths_are_within_five_percent() {
        // 5% rather than 3% because the short samples are the loose ones: on a
        // 14-character title a 4px disagreement is already 3%. The long titles
        // below are held to a tighter bound.
        for (text, expected) in TITLE_SAMPLES {
            let measured = text_pixel_width(text, TITLE_FONT_SIZE_PX);
            let error = error_pct(measured, *expected);
            assert!(
                error < 5.0,
                "{text:?}: measured {measured}, expected {expected}, {error:.1}% out"
            );
        }
    }

    #[test]
    fn long_title_widths_are_within_two_percent() {
        for (text, expected) in TITLE_SAMPLES.iter().filter(|(_, px)| *px >= 200) {
            let measured = text_pixel_width(text, TITLE_FONT_SIZE_PX);
            let error = error_pct(measured, *expected);
            assert!(
                error < 2.0,
                "{text:?}: measured {measured}, expected {expected}, {error:.1}% out"
            );
        }
    }

    #[test]
    fn meta_description_widths_are_within_two_percent() {
        for (text, expected) in META_SAMPLES {
            let measured = text_pixel_width(text, META_DESCRIPTION_FONT_SIZE_PX);
            let error = error_pct(measured, *expected);
            assert!(
                error < 2.0,
                "{text:?}: measured {measured}, expected {expected}, {error:.1}% out"
            );
        }
    }

    #[test]
    fn the_old_bucket_table_would_have_failed_these() {
        // The 4/8/10px buckets measured the first sample at 280 rather than
        // 379. Anything in that region is the old defect returning.
        let measured = text_pixel_width(TITLE_SAMPLES[0].0, TITLE_FONT_SIZE_PX);
        assert!(measured > 350, "measured {measured}");
    }

    #[test]
    fn accented_letters_are_wider_than_a_bucket_table_allows() {
        // 'ä' and 'a' share an advance in Arial; 'i' does not.
        assert_eq!(
            text_pixel_width("häst", TITLE_FONT_SIZE_PX),
            text_pixel_width("hast", TITLE_FONT_SIZE_PX)
        );
        assert!(
            text_pixel_width("iiii", TITLE_FONT_SIZE_PX)
                < text_pixel_width("mmmm", TITLE_FONT_SIZE_PX)
        );
    }

    #[test]
    fn cjk_characters_take_a_full_em() {
        // Four ideographs at 20px are four full em squares.
        assert_eq!(text_pixel_width("日本語版", TITLE_FONT_SIZE_PX), 80);
    }

    #[test]
    fn zero_width_characters_add_nothing() {
        assert_eq!(text_pixel_width("", TITLE_FONT_SIZE_PX), 0);
        assert_eq!(
            text_pixel_width("a\u{200B}b", TITLE_FONT_SIZE_PX),
            text_pixel_width("ab", TITLE_FONT_SIZE_PX)
        );
    }

    #[test]
    fn an_unlisted_character_falls_back_rather_than_vanishing() {
        // Cyrillic isn't in the tables; it must still occupy space.
        assert!(text_pixel_width("страница", TITLE_FONT_SIZE_PX) > 50);
    }
}
