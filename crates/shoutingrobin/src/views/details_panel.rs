use gpui::{
    AnyElement, App, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{ActiveTheme, Icon as UiIcon, Sizable as _, scroll::ScrollableElement as _};

use crate::crawl::event::{A11yIssue, PageRecord, SdFormat, SdSeverity};
use crate::ui::icon::Icon;
use crate::ui::tag::{Tone, indexability_tone, status_code_tone, tone_tag};

pub struct DetailsPanel {
    pub selected: Option<PageRecord>,
}

impl DetailsPanel {
    pub fn new() -> Self {
        Self { selected: None }
    }

    pub fn set_selected(&mut self, record: Option<PageRecord>, cx: &mut Context<Self>) {
        self.selected = record;
        cx.notify();
    }
}

impl Default for DetailsPanel {
    fn default() -> Self {
        Self::new()
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "-".into();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    if bytes < 1024 * 1024 {
        return format!("{:.1} KB", bytes as f64 / 1024.0);
    }
    format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
}

fn or_dash(value: &Option<String>) -> SharedString {
    match value {
        Some(v) if !v.is_empty() => SharedString::from(v.clone()),
        _ => SharedString::from("-"),
    }
}

fn row(label: &str, value: impl IntoElement, muted: Hsla) -> AnyElement {
    div()
        .flex()
        .flex_grow()
        .justify_between()
        .gap_3()
        .text_xs()
        .child(
            div()
                .flex_shrink_0()
                .text_color(muted)
                .child(label.to_string()),
        )
        .child(div().max_w_full().min_w(px(0.)).child(value))
        .into_any_element()
}

fn section_header(
    label: &str,
    icon: Option<Icon>,
    summary: Option<AnyElement>,
    muted: Hsla,
) -> AnyElement {
    let mut header = div()
        .flex()
        .items_center()
        .gap_1p5()
        .text_xs()
        .text_color(muted);
    if let Some(icon) = icon {
        header = header.child(UiIcon::from(icon).xsmall());
    }
    header = header.child(SharedString::from(label.to_uppercase()));

    div()
        .flex()
        .items_center()
        .justify_between()
        .mb_1p5()
        .child(header)
        .when_some(summary, |this, s| this.child(s))
        .into_any_element()
}

fn section(
    label: &str,
    icon: Option<Icon>,
    summary: Option<AnyElement>,
    muted: Hsla,
    border: Hsla,
    body: AnyElement,
) -> AnyElement {
    div()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(border)
        .child(section_header(label, icon, summary, muted))
        .child(body)
        .into_any_element()
}

fn a11y_impact_tone(impact: &str) -> Tone {
    match impact {
        "critical" | "serious" => Tone::Err,
        "moderate" => Tone::Warn,
        _ => Tone::Neutral,
    }
}

fn serp_preview(rec: &PageRecord, cx: &App) -> AnyElement {
    let title = rec
        .title
        .as_deref()
        .filter(|t| !t.is_empty())
        .unwrap_or("No title");
    let url_display = rec.url.clone();
    let desc = rec
        .meta_description
        .as_deref()
        .filter(|d| !d.is_empty())
        .unwrap_or("No meta description");

    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_2()
        .py_1p5()
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().blue)
                .child(SharedString::from(title.to_string())),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().green)
                .child(SharedString::from(url_display)),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(SharedString::from(desc.to_string())),
        )
        .into_any_element()
}

fn a11y_issue_row(issue: &A11yIssue, muted: Hsla, fg: Hsla, border: Hsla, cx: &App) -> AnyElement {
    let impact_tag =
        tone_tag(a11y_impact_tone(&issue.impact)).child(SharedString::from(issue.impact.clone()));
    div()
        .flex()
        .flex_col()
        .gap_0p5()
        .pt_1()
        .border_t_1()
        .border_color(border)
        .child(
            div().flex().items_center().gap_1().child(impact_tag).child(
                div()
                    .text_color(fg)
                    .child(SharedString::from(issue.rule.clone())),
            ),
        )
        .when_some(issue.target.clone(), |el, target| {
            el.child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(SharedString::from(target)),
            )
        })
        .when_some(issue.html.clone(), |el, html| {
            el.child(
                div()
                    .text_xs()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_color(muted)
                    .overflow_x_scrollbar()
                    .child(SharedString::from(html)),
            )
        })
        .into_any_element()
}

fn lcp_tone(ms: u64) -> Tone {
    match ms {
        0..=2500 => Tone::Ok,
        2501..=4000 => Tone::Warn,
        _ => Tone::Err,
    }
}

fn cls_tone(value: f64) -> Tone {
    if value <= 0.1 {
        Tone::Ok
    } else if value <= 0.25 {
        Tone::Warn
    } else {
        Tone::Err
    }
}

fn inp_tone(ms: u64) -> Tone {
    match ms {
        0..=200 => Tone::Ok,
        201..=500 => Tone::Warn,
        _ => Tone::Err,
    }
}

fn ttfb_tone(ms: u64) -> Tone {
    match ms {
        0..=800 => Tone::Ok,
        801..=1800 => Tone::Warn,
        _ => Tone::Err,
    }
}

fn vital_tile(
    label: &str,
    value: SharedString,
    tone: Option<Tone>,
    muted: Hsla,
    border: Hsla,
    panel2: Hsla,
) -> AnyElement {
    let value_color = match tone {
        Some(Tone::Ok) => Some(gpui::hsla(142. / 360., 0.71, 0.45, 1.0)),
        Some(Tone::Warn) => Some(gpui::hsla(38. / 360., 0.92, 0.50, 1.0)),
        Some(Tone::Err) => Some(gpui::hsla(0. / 360., 0.84, 0.60, 1.0)),
        _ => None,
    };
    let mut value_div = div()
        .text_sm()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(value);
    if let Some(c) = value_color {
        value_div = value_div.text_color(c);
    }
    div()
        .bg(panel2)
        .border_1()
        .border_color(border)
        .rounded_md()
        .p_2()
        .flex()
        .flex_col()
        .items_center()
        .gap_0p5()
        .child(
            div()
                .text_xs()
                .text_color(muted)
                .child(SharedString::from(label.to_string())),
        )
        .child(value_div)
        .into_any_element()
}

impl Render for DetailsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let fg = theme.foreground;
        let border = theme.border;
        let bg = theme.background;
        let panel2 = theme.secondary;

        let body = match &self.selected {
            None => div()
                .id("details-scroll")
                .overflow_y_scrollbar()
                .p_4()
                .text_sm()
                .text_color(muted)
                .child("Select a URL to inspect details.")
                .into_any_element(),
            Some(rec) => {
                let header = div()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(UiIcon::from(Icon::PanelRightOpen).small())
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child("URL Details"),
                            ),
                    )
                    .child(
                        div()
                            .mt_1()
                            .text_xs()
                            .text_color(muted)
                            .child(SharedString::from(rec.url.clone())),
                    );

                // URL Information
                let status_tag = match rec.status {
                    Some(c) => tone_tag(status_code_tone(c))
                        .child(SharedString::from(c.to_string()))
                        .into_any_element(),
                    None => div().text_color(muted).child("-").into_any_element(),
                };
                let indexability_value = rec.indexability.clone().unwrap_or_else(|| "-".into());
                let indexability_tag = if indexability_value == "-" {
                    div().text_color(muted).child("-").into_any_element()
                } else {
                    tone_tag(indexability_tone(&indexability_value))
                        .child(SharedString::from(indexability_value.clone()))
                        .into_any_element()
                };
                let url_info = div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .gap_1()
                    .child(row("Address", SharedString::from(rec.url.clone()), muted))
                    .child(row("Status", status_tag, muted))
                    .child(row("Content Type", or_dash(&rec.content_type), muted))
                    .child(row(
                        "Size",
                        SharedString::from(format_bytes(rec.size_bytes)),
                        muted,
                    ))
                    .child(row(
                        "Response Time",
                        SharedString::from(format!("{}ms", rec.response_time.as_millis())),
                        muted,
                    ))
                    .child(row("Indexability", indexability_tag, muted))
                    .child(row("Canonical", or_dash(&rec.canonical), muted))
                    .when_some(rec.redirect_url.clone(), |el, redirect| {
                        el.child(row("Redirect URI", SharedString::from(redirect), muted))
                            .child(row(
                                "Redirect Status",
                                SharedString::from(
                                    rec.redirect_status
                                        .map(|s| s.to_string())
                                        .unwrap_or_else(|| "301".to_string()),
                                ),
                                muted,
                            ))
                    })
                    .when(
                        rec.headers
                            .iter()
                            .any(|(k, _)| k.eq_ignore_ascii_case("last-modified")),
                        |el| {
                            el.child(row(
                                "Last Modified",
                                SharedString::from(
                                    rec.headers
                                        .iter()
                                        .find(|(k, _)| k.eq_ignore_ascii_case("last-modified"))
                                        .map(|(_, v)| v.as_str())
                                        .unwrap_or("-"),
                                ),
                                muted,
                            ))
                        },
                    )
                    .into_any_element();

                // Page content
                let page_content = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(row("Title", or_dash(&rec.title), muted))
                    .child(row(
                        "Title Length",
                        SharedString::from(
                            rec.title
                                .as_ref()
                                .map(|t| t.len().to_string())
                                .unwrap_or_else(|| "0".into()),
                        ),
                        muted,
                    ))
                    .child(row(
                        "Title Pixel Width",
                        SharedString::from(
                            rec.title_pixel_width
                                .map(|w| w.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ),
                        muted,
                    ))
                    .when_some(rec.title_2.clone(), |el, t2| {
                        el.child(row("Title 2", SharedString::from(t2), muted))
                    })
                    .child(row("Meta Desc", or_dash(&rec.meta_description), muted))
                    .child(row(
                        "Meta Length",
                        SharedString::from(
                            rec.meta_description
                                .as_ref()
                                .map(|d| d.len().to_string())
                                .unwrap_or_else(|| "0".into()),
                        ),
                        muted,
                    ))
                    .child(row(
                        "Meta Pixel Width",
                        SharedString::from(
                            rec.meta_description_pixel_width
                                .map(|w| w.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ),
                        muted,
                    ))
                    .when_some(rec.meta_description_2.clone(), |el, md2| {
                        el.child(row("Meta Desc 2", SharedString::from(md2), muted))
                    })
                    .child(row("H1", or_dash(&rec.h1), muted))
                    .child(row(
                        "H1 Length",
                        SharedString::from(
                            rec.h1
                                .as_ref()
                                .map(|h| h.len().to_string())
                                .unwrap_or_else(|| "0".into()),
                        ),
                        muted,
                    ))
                    .when_some(rec.h1_2.clone(), |el, h1_2| {
                        el.child(row("H1-2", SharedString::from(h1_2), muted))
                    })
                    .child(row("H2", or_dash(&rec.h2), muted))
                    .child(row(
                        "Word Count",
                        SharedString::from(
                            rec.word_count
                                .map(|w| w.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ),
                        muted,
                    ))
                    .child(row("Meta Robots", or_dash(&rec.robots), muted))
                    .into_any_element();

                // Structured data
                let sd_summary = if rec.sd_items.is_empty() {
                    None
                } else {
                    Some(
                        tone_tag(Tone::Accent)
                            .child(SharedString::from(format!("{} types", rec.sd_items.len())))
                            .into_any_element(),
                    )
                };
                let sd_types_text = if rec.sd_types.is_empty() {
                    SharedString::from("-")
                } else {
                    SharedString::from(rec.sd_types.join(", "))
                };

                let mut sd_items_body = div().flex().flex_col().gap_1();
                for item in &rec.sd_items {
                    let format_label = match item.format {
                        SdFormat::JsonLd => "JSON-LD",
                        SdFormat::Microdata => "Microdata",
                    };
                    let format_tag = tone_tag(if item.format == SdFormat::JsonLd {
                        Tone::Accent
                    } else {
                        Tone::Neutral
                    })
                    .child(SharedString::from(format_label));
                    sd_items_body = sd_items_body.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .pt_1()
                            .child(
                                div().flex().items_center().gap_1().child(format_tag).child(
                                    div()
                                        .text_color(fg)
                                        .child(SharedString::from(item.type_name.clone())),
                                ),
                            )
                            .when(!item.raw_json.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(muted)
                                        .max_h(gpui::px(120.))
                                        .overflow_y_scrollbar()
                                        .child(SharedString::from(item.raw_json.clone())),
                                )
                            }),
                    );
                }
                for issue in &rec.sd_issues {
                    let tone = match issue.severity {
                        SdSeverity::Error => Tone::Err,
                        SdSeverity::Warning => Tone::Warn,
                    };
                    sd_items_body = sd_items_body.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .pt_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .child(
                                        tone_tag(tone)
                                            .child(SharedString::from(issue.code.clone())),
                                    )
                                    .child(
                                        div()
                                            .text_color(fg)
                                            .child(SharedString::from(issue.type_name.clone())),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(SharedString::from(issue.message.clone())),
                            ),
                    );
                }

                let sd_body = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(row("Types", sd_types_text, muted))
                    .child(row(
                        "JSON-LD",
                        SharedString::from(rec.sd_jsonld_count.to_string()),
                        muted,
                    ))
                    .child(row(
                        "Microdata",
                        SharedString::from(rec.sd_microdata_count.to_string()),
                        muted,
                    ))
                    .when(rec.og_type.is_some(), |el| {
                        el.child(row(
                            "Open Graph",
                            SharedString::from(rec.og_type.clone().unwrap_or_default()),
                            muted,
                        ))
                    })
                    .child(row(
                        "Errors",
                        if rec.sd_errors > 0 {
                            tone_tag(Tone::Err)
                                .child(SharedString::from(rec.sd_errors.to_string()))
                                .into_any_element()
                        } else {
                            SharedString::from("0").into_any_element()
                        },
                        muted,
                    ))
                    .child(row(
                        "Warnings",
                        if rec.sd_warnings > 0 {
                            tone_tag(Tone::Warn)
                                .child(SharedString::from(rec.sd_warnings.to_string()))
                                .into_any_element()
                        } else {
                            SharedString::from("0").into_any_element()
                        },
                        muted,
                    ))
                    .when(
                        !rec.sd_items.is_empty() || !rec.sd_issues.is_empty(),
                        |el| el.child(sd_items_body),
                    )
                    .into_any_element();

                // Accessibility
                let total_a11y = rec.a11y_errors + rec.a11y_warnings;
                let a11y_summary = if total_a11y == 0 {
                    None
                } else {
                    let tone = if rec.a11y_errors > 0 {
                        Tone::Err
                    } else {
                        Tone::Warn
                    };
                    Some(
                        tone_tag(tone)
                            .child(SharedString::from(format!("{} issues", total_a11y)))
                            .into_any_element(),
                    )
                };
                let mut a11y_body = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(row(
                        "Errors",
                        if rec.a11y_errors > 0 {
                            tone_tag(Tone::Err)
                                .child(SharedString::from(rec.a11y_errors.to_string()))
                                .into_any_element()
                        } else {
                            SharedString::from("0").into_any_element()
                        },
                        muted,
                    ))
                    .child(row(
                        "Warnings",
                        if rec.a11y_warnings > 0 {
                            tone_tag(Tone::Warn)
                                .child(SharedString::from(rec.a11y_warnings.to_string()))
                                .into_any_element()
                        } else {
                            SharedString::from("0").into_any_element()
                        },
                        muted,
                    ));
                for issue in &rec.a11y_issues {
                    a11y_body = a11y_body.child(a11y_issue_row(issue, muted, fg, border, cx));
                }
                let a11y_body = a11y_body.into_any_element();

                // Core Web Vitals tile grid
                let lcp_str = rec
                    .lcp_ms
                    .map(|ms| format!("{:.1}s", ms as f32 / 1000.0))
                    .unwrap_or_else(|| "-".into());
                let cls_str = rec
                    .cls
                    .map(|v| format!("{:.3}", v))
                    .unwrap_or_else(|| "-".into());
                let inp_str = rec
                    .inp_ms
                    .map(|ms| format!("{ms}ms"))
                    .unwrap_or_else(|| "-".into());
                let ttfb_str = rec
                    .ttfb_ms
                    .map(|ms| format!("{ms}ms"))
                    .unwrap_or_else(|| "-".into());

                let vitals_body = div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(div().flex_1().child(vital_tile(
                                "LCP",
                                SharedString::from(lcp_str),
                                rec.lcp_ms.map(lcp_tone),
                                muted,
                                border,
                                panel2,
                            )))
                            .child(div().flex_1().child(vital_tile(
                                "CLS",
                                SharedString::from(cls_str),
                                rec.cls.map(cls_tone),
                                muted,
                                border,
                                panel2,
                            )))
                            .child(div().flex_1().child(vital_tile(
                                "INP",
                                SharedString::from(inp_str),
                                rec.inp_ms.map(inp_tone),
                                muted,
                                border,
                                panel2,
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(div().flex_1().child(vital_tile(
                                "TTFB",
                                SharedString::from(ttfb_str),
                                rec.ttfb_ms.map(ttfb_tone),
                                muted,
                                border,
                                panel2,
                            )))
                            .child(div().flex_1().child(vital_tile(
                                "Resp",
                                SharedString::from(format!("{}ms", rec.response_time.as_millis())),
                                None,
                                muted,
                                border,
                                panel2,
                            )))
                            .child(div().flex_1()),
                    )
                    .into_any_element();

                // Link metrics
                let link_score_str = rec
                    .link_score
                    .map(|s| format!("{s:.1}"))
                    .unwrap_or_else(|| "-".into());
                let links_body = div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(row(
                        "Inlinks",
                        SharedString::from(rec.inlinks_count.to_string()),
                        muted,
                    ))
                    .child(row(
                        "Outlinks",
                        SharedString::from(rec.outlinks.len().to_string()),
                        muted,
                    ))
                    .child(row("Link Score", SharedString::from(link_score_str), muted))
                    .child(row(
                        "Backlinks",
                        SharedString::from(rec.backlinks.len().to_string()),
                        muted,
                    ))
                    .child(row(
                        "Depth",
                        SharedString::from(rec.depth.to_string()),
                        muted,
                    ))
                    .child(row(
                        "Hreflang",
                        SharedString::from(rec.hreflang_tags.len().to_string()),
                        muted,
                    ))
                    .child(row(
                        "Near Duplicates",
                        SharedString::from(
                            rec.near_duplicate_count
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "-".into()),
                        ),
                        muted,
                    ))
                    .child(row(
                        "Closest Similarity",
                        SharedString::from(
                            rec.closest_similarity
                                .map(|s| format!("{s}%"))
                                .unwrap_or_else(|| "-".into()),
                        ),
                        muted,
                    ))
                    .into_any_element();

                // Images
                let images_summary = if rec.images.is_empty() {
                    None
                } else {
                    let missing_alt = rec
                        .images
                        .iter()
                        .filter(|img| {
                            !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty())
                        })
                        .count();
                    if missing_alt > 0 {
                        Some(
                            tone_tag(Tone::Warn)
                                .child(SharedString::from(format!("{} missing alt", missing_alt)))
                                .into_any_element(),
                        )
                    } else {
                        Some(
                            tone_tag(Tone::Ok)
                                .child(SharedString::from(format!("{} images", rec.images.len())))
                                .into_any_element(),
                        )
                    }
                };

                let mut images_body = div().flex().flex_col().gap_1();
                if rec.images.is_empty() {
                    images_body =
                        images_body.child(div().text_color(muted).child("No images found"));
                } else {
                    for image in &rec.images {
                        let alt_tag = if image.has_alt_attr {
                            if image.alt.as_deref().is_none_or(|a| a.is_empty()) {
                                tone_tag(Tone::Warn).child(SharedString::from("empty"))
                            } else {
                                tone_tag(Tone::Ok).child(SharedString::from("yes"))
                            }
                        } else {
                            tone_tag(Tone::Err).child(SharedString::from("missing"))
                        };
                        images_body = images_body.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .pt_1()
                                .border_t_1()
                                .border_color(border)
                                .child(
                                    div()
                                        .text_xs()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_color(fg)
                                        .child(SharedString::from(image.src.clone())),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_xs()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .child(div().text_color(muted).child("Alt:"))
                                                .child(alt_tag)
                                                .when_some(image.alt.clone(), |el, alt| {
                                                    el.child(
                                                        div()
                                                            .text_color(fg)
                                                            .child(SharedString::from(alt)),
                                                    )
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_1()
                                                .when_some(image.width, |el, w| {
                                                    el.child(div().text_color(fg).child(
                                                        SharedString::from(format!("{w}px")),
                                                    ))
                                                })
                                                .when(
                                                    image.width.is_some() && image.height.is_some(),
                                                    |el| {
                                                        el.child(div().text_color(muted).child("x"))
                                                    },
                                                )
                                                .when_some(image.height, |el, h| {
                                                    el.child(div().text_color(fg).child(
                                                        SharedString::from(format!("{h}px")),
                                                    ))
                                                })
                                                .when(
                                                    image.width.is_none() && image.height.is_none(),
                                                    |el| {
                                                        el.child(
                                                            div()
                                                                .text_color(muted)
                                                                .child("no dimensions"),
                                                        )
                                                    },
                                                ),
                                        ),
                                ),
                        );
                    }
                }
                let images_section_body = images_body.into_any_element();

                div()
                    .id("details-scroll")
                    .overflow_y_scrollbar()
                    .flex()
                    .flex_col()
                    .child(header)
                    .child(section(
                        "URL Information",
                        None,
                        None,
                        muted,
                        border,
                        url_info,
                    ))
                    .child(section(
                        "Page Content",
                        None,
                        None,
                        muted,
                        border,
                        page_content,
                    ))
                    .child(section(
                        "Structured Data",
                        Some(Icon::Braces),
                        sd_summary,
                        muted,
                        border,
                        sd_body,
                    ))
                    .child(section(
                        "Accessibility",
                        Some(Icon::Accessibility),
                        a11y_summary,
                        muted,
                        border,
                        a11y_body,
                    ))
                    .child(section(
                        "Core Web Vitals",
                        Some(Icon::Gauge),
                        None,
                        muted,
                        border,
                        vitals_body,
                    ))
                    .child(section(
                        "Link Metrics",
                        None,
                        None,
                        muted,
                        border,
                        links_body,
                    ))
                    .child(section(
                        "Images",
                        Some(Icon::Image),
                        images_summary,
                        muted,
                        border,
                        images_section_body,
                    ))
                    .child(section(
                        "SERP Preview",
                        None,
                        None,
                        muted,
                        border,
                        serp_preview(rec, cx),
                    ))
                    .when(!rec.hreflang_issues.is_empty(), |el| {
                        let mut body = div().flex().flex_col().gap_0p5();
                        for issue in &rec.hreflang_issues {
                            let label = match issue {
                                crate::crawl::event::HreflangIssue::MissingReturnTag {
                                    lang,
                                    target_url,
                                } => {
                                    format!("Missing return tag: {lang} -> {target_url}")
                                }
                                crate::crawl::event::HreflangIssue::InvalidLanguageCode {
                                    code,
                                } => {
                                    format!("Invalid language code: {code}")
                                }
                                crate::crawl::event::HreflangIssue::MissingXDefault => {
                                    "Missing x-default".into()
                                }
                                crate::crawl::event::HreflangIssue::NonCanonicalUrl {
                                    hreflang_url,
                                } => {
                                    format!("Non-canonical target: {hreflang_url}")
                                }
                            };
                            body = body.child(
                                div()
                                    .text_xs()
                                    .text_color(gpui::hsla(0. / 360., 0.84, 0.60, 1.0))
                                    .child(SharedString::from(label)),
                            );
                        }
                        el.child(section(
                            "Hreflang Issues",
                            None,
                            Some(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(SharedString::from(
                                        rec.hreflang_issues.len().to_string(),
                                    ))
                                    .into_any_element(),
                            ),
                            muted,
                            border,
                            body.into_any_element(),
                        ))
                    })
                    .when(!rec.backlinks.is_empty(), |el| {
                        let mut body = div().flex().flex_col().gap_0p5();
                        for bl in &rec.backlinks {
                            body = body.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .pt_1()
                                    .border_t_1()
                                    .border_color(border)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .text_color(fg)
                                            .child(SharedString::from(bl.source_url.clone())),
                                    )
                                    .child(
                                        div().text_xs().text_color(muted).child(
                                            SharedString::from(
                                                bl.anchor
                                                    .as_deref()
                                                    .map(|a| format!("Anchor: {a}"))
                                                    .unwrap_or_else(|| "No anchor".into()),
                                            ),
                                        ),
                                    )
                                    .when(bl.rel.as_deref().is_some(), |el| {
                                        el.child(div().text_xs().text_color(muted).child(
                                            SharedString::from(format!(
                                                "Rel: {}",
                                                bl.rel.as_deref().unwrap()
                                            )),
                                        ))
                                    }),
                            );
                        }
                        el.child(section(
                            "Inlinks (From)",
                            None,
                            Some(
                                SharedString::from(format!("{} links", rec.backlinks.len()))
                                    .into_any_element(),
                            ),
                            muted,
                            border,
                            body.into_any_element(),
                        ))
                    })
                    .when(!rec.outlinks.is_empty(), |el| {
                        let mut body = div().flex().flex_col().gap_0p5();
                        let display_count = rec.outlinks.len().min(50);
                        for link in &rec.outlinks[..display_count] {
                            let is_nofollow = link
                                .rel
                                .as_deref()
                                .is_some_and(|r| r.to_ascii_lowercase().contains("nofollow"));
                            body = body.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .pt_1()
                                    .border_t_1()
                                    .border_color(border)
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_1()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_family(
                                                        cx.theme().mono_font_family.clone(),
                                                    )
                                                    .text_color(fg)
                                                    .child(SharedString::from(
                                                        link.dst_url.clone(),
                                                    )),
                                            )
                                            .when(is_nofollow, |el| {
                                                el.child(
                                                    tone_tag(Tone::Warn)
                                                        .child(SharedString::from("nofollow")),
                                                )
                                            }),
                                    )
                                    .child(
                                        div().text_xs().text_color(muted).child(
                                            SharedString::from(
                                                link.anchor
                                                    .as_deref()
                                                    .map(|a| a.to_string())
                                                    .unwrap_or_else(|| "-".into()),
                                            ),
                                        ),
                                    ),
                            );
                        }
                        if rec.outlinks.len() > display_count {
                            body = body.child(div().text_xs().text_color(muted).pt_1().child(
                                SharedString::from(format!(
                                    "... and {} more",
                                    rec.outlinks.len() - display_count
                                )),
                            ));
                        }
                        el.child(section(
                            "Outlinks (To)",
                            None,
                            Some(
                                SharedString::from(format!("{} links", rec.outlinks.len()))
                                    .into_any_element(),
                            ),
                            muted,
                            border,
                            body.into_any_element(),
                        ))
                    })
                    .when(!rec.headers.is_empty(), |el| {
                        let mut headers_body = div().flex().flex_col().gap_0p5();
                        for (key, value) in &rec.headers {
                            let display_value = if value.len() > 80 {
                                format!("{}...", &value[..80])
                            } else {
                                value.clone()
                            };
                            headers_body = headers_body.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .pt_1()
                                    .border_t_1()
                                    .border_color(border)
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(muted)
                                            .child(SharedString::from(key.clone())),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .text_color(fg)
                                            .overflow_x_scrollbar()
                                            .child(SharedString::from(display_value)),
                                    ),
                            );
                        }
                        el.child(section(
                            "HTTP Headers",
                            None,
                            Some(
                                SharedString::from(format!("{} headers", rec.headers.len()))
                                    .into_any_element(),
                            ),
                            muted,
                            border,
                            headers_body.into_any_element(),
                        ))
                    })
                    .into_any_element()
            }
        };

        div()
            .w(gpui::px(420.))
            .h_full()
            .border_l_1()
            .border_color(border)
            .bg(bg)
            .child(body)
    }
}
