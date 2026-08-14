//! Text width in pixels, as a search engine result page renders it.
//!
//! Google truncates a title or description on rendered width rather than
//! character count, which is why the over/under-pixel-width rules exist at all.
//! Measuring therefore needs the real advance width of each glyph: the 4/8/10px
//! bucket approximation this replaced understated widths by roughly 27%.
//!
//! The two surfaces use different fonts, so this holds two tables:
//!
//! * **Titles**: Google Sans at 22px. Google sets its own font on the result
//!   heading now, not Arial.
//! * **Descriptions**: `sans-serif` at 14px, which resolves to Arial on the
//!   desktop platforms this matters for.
//!
//! Both tables are advance widths in font units, read straight out of the font
//! (Google Sans has 1000 to the em, Arial 2048). Widths are summed in font
//! units and converted once at the end, so rounding happens a single time
//! rather than per glyph.
//!
//! The Google Sans table was re-checked against `assets/fonts/GoogleSans-Regular.ttf`,
//! which the app now bundles, and the Arial table against the platform's
//! `arial.ttf`: all 191 codepoints of each match the font's `hmtx` exactly. The
//! bundled file makes that audit repeatable — read `hmtx` for U+0020..U+007E and
//! U+00A0..U+00FF and compare.
//!
//! Note this deliberately parts company with the tool this crawler is compared
//! against, which models titles as 20px: our title widths run about 11% wider
//! than its. The pixel thresholds are container widths and do not move with the
//! font, so a larger heading font legitimately means fewer characters fit.
//!
//! The remaining error against a rendered page is per-glyph rounding in the
//! rasteriser, which no advance-width table can reproduce.

/// One font's advance widths.
struct FontMetrics {
    /// Font units per em, the divisor that turns an advance into a fraction of
    /// the font size.
    units_per_em: u32,
    /// U+0020 (space) through U+007E (tilde).
    ascii: &'static [u16; 95],
    /// U+00A0 through U+00FF, the accented letters of the Western European
    /// languages.
    latin1: &'static [u16; 96],
    /// Characters outside those blocks that turn up often enough to be worth a
    /// real width: typographic quotes and dashes, the ellipsis, currency.
    extra: &'static [(char, u16)],
    /// Used for characters in none of the tables. Close to the width of a
    /// lowercase Latin letter, so an unlisted accented letter lands near the
    /// mark.
    fallback: u16,
}

impl FontMetrics {
    fn advance(&self, ch: char) -> u16 {
        let code = ch as u32;
        if (0x20..0x7F).contains(&code) {
            return self.ascii[(code - 0x20) as usize];
        }
        if (0xA0..0x100).contains(&code) {
            return self.latin1[(code - 0xA0) as usize];
        }
        if let Some((_, advance)) = self.extra.iter().find(|(candidate, _)| *candidate == ch) {
            return *advance;
        }
        // Control characters and the zero-width formatting marks render as
        // nothing.
        if code < 0x20 || matches!(code, 0x200B..=0x200F | 0x2060 | 0xFEFF) {
            return 0;
        }
        // CJK and fullwidth characters occupy the full em square. Neither font
        // has glyphs for them, so a browser falls back to one that does, and
        // charging them a Latin letter's width would understate an Asian title
        // by roughly half.
        if is_full_width(code) {
            return self.units_per_em as u16;
        }
        self.fallback
    }

    fn width(&self, text: &str, font_size_px: f32) -> u32 {
        let units: u64 = text.chars().map(|ch| u64::from(self.advance(ch))).sum();
        let pixels = units as f64 * f64::from(font_size_px) / f64::from(self.units_per_em);
        pixels.round() as u32
    }
}

/// Google's desktop SERP title size.
pub const TITLE_FONT_SIZE_PX: f32 = 22.0;

/// Google's desktop SERP description size.
pub const META_DESCRIPTION_FONT_SIZE_PX: f32 = 14.0;

static GOOGLE_SANS: FontMetrics = FontMetrics {
    units_per_em: 1000,
    ascii: &GOOGLE_SANS_ASCII,
    latin1: &GOOGLE_SANS_LATIN1,
    extra: GOOGLE_SANS_EXTRA,
    // 'n'.
    fallback: 559,
};

static ARIAL: FontMetrics = FontMetrics {
    units_per_em: 2048,
    ascii: &ARIAL_ASCII,
    latin1: &ARIAL_LATIN1,
    extra: ARIAL_EXTRA,
    // 'n'.
    fallback: 1139,
};

/// The rendered width of a page title in the search results.
pub fn title_pixel_width(text: &str) -> u32 {
    GOOGLE_SANS.width(text, TITLE_FONT_SIZE_PX)
}

/// The rendered width of a meta description in the search results.
pub fn meta_description_pixel_width(text: &str) -> u32 {
    ARIAL.width(text, META_DESCRIPTION_FONT_SIZE_PX)
}

static GOOGLE_SANS_ASCII: [u16; 95] = [
    232, 236, 320, 640, 540, 828, 623, 177, 321, 321, 425, 556, 236, 439, 236, 300, 643, 430, 524,
    533, 591, 559, 555, 527, 550, 555, 236, 236, 495, 566, 495, 487, 882, 670, 600, 736, 701, 544,
    529, 806, 696, 243, 530, 627, 505, 864, 706, 822, 574, 822, 591, 555, 530, 667, 633, 944, 619,
    585, 570, 316, 300, 316, 463, 476, 397, 531, 600, 540, 600, 561, 362, 595, 562, 209, 209, 504,
    209, 873, 559, 594, 600, 600, 370, 471, 364, 559, 508, 775, 473, 508, 486, 311, 235, 311, 534,
];

static GOOGLE_SANS_LATIN1: [u16; 96] = [
    232, 236, 529, 523, 609, 570, 235, 544, 392, 530, 472, 546, 571, 0, 530, 399, 322, 542, 298,
    318, 397, 629, 612, 236, 403, 233, 513, 546, 677, 718, 770, 462, 670, 670, 670, 670, 670, 670,
    953, 736, 544, 544, 544, 544, 243, 243, 243, 243, 713, 706, 822, 822, 822, 822, 822, 544, 822,
    667, 667, 667, 667, 585, 574, 553, 531, 531, 531, 531, 531, 531, 919, 540, 561, 561, 561, 561,
    209, 209, 209, 209, 566, 559, 594, 594, 594, 594, 594, 542, 594, 559, 559, 559, 559, 508, 600,
    508,
];

static GOOGLE_SANS_EXTRA: &[(char, u16)] = &[
    ('\u{152}', 1158),  // 'Œ'
    ('\u{153}', 1001),  // 'œ'
    ('\u{160}', 555),   // 'Š'
    ('\u{161}', 471),   // 'š'
    ('\u{178}', 585),   // 'Ÿ'
    ('\u{17D}', 570),   // 'Ž'
    ('\u{17E}', 486),   // 'ž'
    ('\u{192}', 573),   // 'ƒ'
    ('\u{2C6}', 395),   // 'ˆ'
    ('\u{2DC}', 398),   // '˜'
    ('\u{2013}', 600),  // '–'
    ('\u{2014}', 900),  // '—'
    ('\u{2018}', 222),  // '‘'
    ('\u{2019}', 222),  // '’'
    ('\u{201A}', 222),  // '‚'
    ('\u{201C}', 391),  // '“'
    ('\u{201D}', 391),  // '”'
    ('\u{201E}', 391),  // '„'
    ('\u{2020}', 432),  // '†'
    ('\u{2021}', 432),  // '‡'
    ('\u{2022}', 379),  // '•'
    ('\u{2026}', 696),  // '…'
    ('\u{2030}', 1093), // '‰'
    ('\u{2039}', 346),  // '‹'
    ('\u{203A}', 346),  // '›'
    ('\u{20AC}', 634),  // '€'
    ('\u{2122}', 497),  // '™'
    ('\u{2212}', 556),  // '−'
];

/// U+0020 (space) through U+007E (tilde).
static ARIAL_ASCII: [u16; 95] = [
    569, 569, 727, 1139, 1139, 1821, 1366, 391, 682, 682, 797, 1196, 569, 682, 569, 569, 1139,
    1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 1139, 569, 569, 1196, 1196, 1196, 1139, 2079,
    1366, 1366, 1479, 1479, 1366, 1251, 1593, 1479, 569, 1024, 1366, 1139, 1706, 1479, 1593, 1366,
    1593, 1479, 1366, 1251, 1479, 1366, 1933, 1366, 1366, 1251, 569, 569, 569, 961, 1139, 682,
    1139, 1139, 1024, 1139, 1139, 569, 1139, 1139, 455, 455, 1024, 455, 1706, 1139, 1139, 1139,
    1139, 682, 1024, 569, 1139, 1024, 1479, 1024, 1024, 1024, 684, 532, 684, 1196,
];

/// U+00A0 (no-break space) through U+00FF, which covers the accented letters of
/// the Western European languages.
static ARIAL_LATIN1: [u16; 96] = [
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
static ARIAL_EXTRA: &[(char, u16)] = &[
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

    /// Titles measured in a browser on a real result page, which is the only
    /// ground truth there is for this: the width a rendering engine gives the
    /// string in the font and size Google actually sets.
    const BROWSER_MEASURED_TITLES: &[(&str, f64)] = &[
        ("Rust (programming language)", 293.58),
        ("wikipedia - crates.io: Rust Package Registry", 427.08),
        ("ByLynga: Kvalitetsbett för dig och din häst", 414.59),
    ];

    /// Real meta descriptions with their known rendered width. Descriptions are
    /// Arial at 14px for us and for the compared tool alike.
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

    /// The whole model in one assertion: font, size and table, against widths a
    /// browser produced. 1% is tight enough that changing the font or moving
    /// the size by a single pixel fails it (20px is 9% out, Arial at 22px is
    /// 0.7% and would pass here but is three times further off on average).
    #[test]
    fn titles_match_widths_measured_in_a_browser() {
        for (text, expected) in BROWSER_MEASURED_TITLES {
            let measured = f64::from(title_pixel_width(text));
            let error = (measured - expected).abs() / expected * 100.0;
            assert!(
                error < 1.0,
                "{text:?}: measured {measured}, browser says {expected}, {error:.2}% out"
            );
        }
    }

    #[test]
    fn meta_description_widths_are_within_two_percent() {
        for (text, expected) in META_SAMPLES {
            let measured = meta_description_pixel_width(text);
            let error = error_pct(measured, *expected);
            assert!(
                error < 2.0,
                "{text:?}: measured {measured}, expected {expected}, {error:.1}% out"
            );
        }
    }

    #[test]
    fn the_old_bucket_table_would_have_failed_these() {
        // The 4/8/10px buckets measured this at 280. Anything in that region
        // is the old defect returning.
        let measured = title_pixel_width("Kvalitetsbett för dig och din häst | ByLynga");
        assert!(measured > 350, "measured {measured}");
    }

    #[test]
    fn the_two_surfaces_use_different_fonts() {
        // The same string at the same size measured with each table: Google
        // Sans is the narrower face, so this would only be equal if both
        // surfaces were reading one table.
        let text = "Kvalitetsbett för dig och din häst";
        assert_ne!(
            GOOGLE_SANS.width(text, 20.0),
            ARIAL.width(text, 20.0),
            "titles and descriptions must not share a table"
        );
    }

    #[test]
    fn accented_letters_carry_their_own_width() {
        // 'ä' and 'a' share an advance in both faces; 'i' does not.
        assert_eq!(title_pixel_width("häst"), title_pixel_width("hast"));
        assert!(title_pixel_width("iiii") < title_pixel_width("mmmm"));
    }

    #[test]
    fn cjk_characters_take_a_full_em() {
        // Four ideographs at the title size are four full em squares.
        assert_eq!(
            title_pixel_width("日本語版"),
            (TITLE_FONT_SIZE_PX * 4.0) as u32
        );
        assert_eq!(
            meta_description_pixel_width("日本語版"),
            (META_DESCRIPTION_FONT_SIZE_PX * 4.0) as u32
        );
    }

    #[test]
    fn zero_width_characters_add_nothing() {
        assert_eq!(title_pixel_width(""), 0);
        assert_eq!(title_pixel_width("a\u{200B}b"), title_pixel_width("ab"));
    }

    #[test]
    fn an_unlisted_character_falls_back_rather_than_vanishing() {
        // Cyrillic isn't in the tables; it must still occupy space.
        assert!(title_pixel_width("страница") > 50);
    }
}
