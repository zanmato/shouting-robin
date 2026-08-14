use gpui::{
    AnyElement, App, Context, Hsla, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Icon as UiIcon, Sizable as _, scroll::ScrollableElement as _, tooltip::Tooltip,
};

use crate::a11y_rules::rule_description;
use crate::crawl::event::{A11yIssue, PageRecord, SdFormat, SdSeverity};
use crate::ui::icon::Icon;
use crate::ui::tag::{Tone, indexability_tone, status_code_tone, tone_tag};
use crate::views::results_grid::ssr_diff_label;
use shoutingrobin_ui::{HtmlView, JsonView};

pub struct DetailsPanel {
    pub selected: Option<PageRecord>,
    json_views: Vec<Option<JsonView>>,
    html_views: Vec<Option<HtmlView>>,
}

impl DetailsPanel {
    pub fn new() -> Self {
        Self {
            selected: None,
            json_views: Vec::new(),
            html_views: Vec::new(),
        }
    }

    pub fn set_selected(&mut self, record: Option<PageRecord>, cx: &mut Context<Self>) {
        self.json_views = match &record {
            Some(rec) => rec
                .sd_items
                .iter()
                .map(|item| {
                    if item.raw_json.is_empty() {
                        None
                    } else {
                        Some(JsonView::new(&item.raw_json, cx))
                    }
                })
                .collect(),
            None => Vec::new(),
        };
        self.html_views = match &record {
            Some(rec) => rec
                .a11y_issues
                .iter()
                .map(|issue| {
                    issue
                        .html
                        .as_deref()
                        .filter(|h| !h.is_empty())
                        .map(|h| HtmlView::new(h, cx))
                })
                .collect(),
            None => Vec::new(),
        };
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
        .flex_grow(1.0)
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

fn a11y_issue_row(
    issue: &A11yIssue,
    html_view: Option<&HtmlView>,
    index: usize,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    let impact_tag = tone_tag(a11y_impact_tone(&issue.impact), cx)
        .child(SharedString::from(issue.impact.clone()));
    let rule_name = SharedString::from(issue.rule.clone());
    let rule_el: AnyElement = match rule_description(&issue.rule) {
        Some(desc) => {
            let desc = SharedString::from(desc.to_string());
            div()
                .id(("a11y-rule-tip", index))
                .text_color(fg)
                .text_sm()
                .child(rule_name)
                .tooltip(move |window, cx| Tooltip::new(desc.clone()).build(window, cx))
                .into_any_element()
        }
        None => div()
            .text_color(fg)
            .text_sm()
            .child(rule_name)
            .into_any_element(),
    };
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
                .child(impact_tag)
                .child(rule_el),
        )
        .when_some(issue.target.clone(), |el, target| {
            el.child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child(SharedString::from(target)),
            )
        })
        .when_some(html_view.cloned(), |el, view| el.child(view))
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

fn fcp_tone(ms: u64) -> Tone {
    match ms {
        0..=1800 => Tone::Ok,
        1801..=3000 => Tone::Warn,
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

fn header_block(rec: &PageRecord, muted: Hsla, border: Hsla) -> AnyElement {
    let url = rec.url.clone();
    div()
        .px_3()
        .py_2()
        .border_b_1()
        .border_color(border)
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
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
                        .id("open-url-details")
                        .cursor_pointer()
                        .text_color(muted)
                        .child(UiIcon::from(Icon::ExternalLink).small())
                        .on_click(move |_, _window, cx| {
                            cx.open_url(&url);
                        }),
                ),
        )
        .child(
            div()
                .mt_1()
                .text_xs()
                .text_color(muted)
                .child(SharedString::from(rec.url.clone())),
        )
        .into_any_element()
}

fn url_information_section(rec: &PageRecord, muted: Hsla, border: Hsla, cx: &App) -> AnyElement {
    let status_tag = match rec.status {
        Some(c) => tone_tag(status_code_tone(c), cx)
            .child(SharedString::from(c.to_string()))
            .into_any_element(),
        None => div().text_color(muted).child("-").into_any_element(),
    };
    let indexability_value = rec.indexability.clone().unwrap_or_else(|| "-".into());
    let indexability_tag = if indexability_value == "-" {
        div().text_color(muted).child("-").into_any_element()
    } else {
        tone_tag(indexability_tone(&indexability_value), cx)
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

    section("URL Information", None, None, muted, border, url_info)
}

fn page_content_section(rec: &PageRecord, muted: Hsla, border: Hsla) -> AnyElement {
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
        .when(rec.ssr_word_count.is_some(), |el| {
            el.child(row(
                "SSR Words",
                SharedString::from(
                    rec.ssr_word_count
                        .map(|w| w.to_string())
                        .unwrap_or_else(|| "-".into()),
                ),
                muted,
            ))
            .child(row(
                "SSR/CSR Diff",
                SharedString::from(ssr_diff_label(rec)),
                muted,
            ))
        })
        .child(row("Meta Robots", or_dash(&rec.robots), muted))
        .into_any_element();

    section("Page Content", None, None, muted, border, page_content)
}

fn structured_data_section(
    rec: &PageRecord,
    json_views: &[Option<JsonView>],
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    let sd_summary = if rec.sd_items.is_empty() {
        None
    } else {
        Some(
            tone_tag(Tone::Accent, cx)
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
    for (item, json_view) in rec.sd_items.iter().zip(json_views.iter()) {
        let format_label = match item.format {
            SdFormat::JsonLd => "JSON-LD",
            SdFormat::Microdata => "Microdata",
        };
        let format_tag = tone_tag(
            if item.format == SdFormat::JsonLd {
                Tone::Accent
            } else {
                Tone::Neutral
            },
            cx,
        )
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
                            .text_sm()
                            .child(SharedString::from(item.type_name.clone())),
                    ),
                )
                .when_some(json_view.clone(), |el, view| el.child(view)),
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
                        .child(tone_tag(tone, cx).child(SharedString::from(issue.code.clone())))
                        .child(
                            div()
                                .text_color(fg)
                                .text_sm()
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
                tone_tag(Tone::Err, cx)
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
                tone_tag(Tone::Warn, cx)
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

    section(
        "Structured Data",
        Some(Icon::Braces),
        sd_summary,
        muted,
        border,
        sd_body,
    )
}

fn accessibility_section(
    rec: &PageRecord,
    html_views: &[Option<HtmlView>],
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
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
            tone_tag(tone, cx)
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
                tone_tag(Tone::Err, cx)
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
                tone_tag(Tone::Warn, cx)
                    .child(SharedString::from(rec.a11y_warnings.to_string()))
                    .into_any_element()
            } else {
                SharedString::from("0").into_any_element()
            },
            muted,
        ));
    for (index, issue) in rec.a11y_issues.iter().enumerate() {
        a11y_body = a11y_body.child(a11y_issue_row(
            issue,
            html_views[index].as_ref(),
            index,
            muted,
            fg,
            border,
            cx,
        ));
    }
    let a11y_body = a11y_body.into_any_element();

    section(
        "Accessibility",
        Some(Icon::Accessibility),
        a11y_summary,
        muted,
        border,
        a11y_body,
    )
}

fn vitals_section(rec: &PageRecord, muted: Hsla, border: Hsla, panel2: Hsla) -> AnyElement {
    let lcp_str = rec
        .lcp_ms
        .map(|ms| format!("{:.1}s", ms as f32 / 1000.0))
        .unwrap_or_else(|| "-".into());
    let cls_str = rec
        .cls
        .map(|v| format!("{:.3}", v))
        .unwrap_or_else(|| "-".into());
    let fcp_str = rec
        .fcp_ms
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
                    "FCP",
                    SharedString::from(fcp_str),
                    rec.fcp_ms.map(fcp_tone),
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
                .child(div().flex_1())
                .child(div().flex_1()),
        )
        .into_any_element();

    section(
        "Core Web Vitals",
        Some(Icon::Gauge),
        None,
        muted,
        border,
        vitals_body,
    )
}

fn link_metrics_section(
    rec: &PageRecord,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
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
            "CSR Inlinks",
            SharedString::from(rec.csr_inlinks_count.to_string()),
            muted,
        ))
        .child(row(
            "CSR In %",
            SharedString::from(if rec.inlinks_count > 0 && rec.csr_inlinks_count > 0 {
                format!(
                    "{}%",
                    (rec.csr_inlinks_count as f64 / rec.inlinks_count as f64 * 100.0).round()
                        as u32
                )
            } else {
                "-".into()
            }),
            muted,
        ))
        .child(row(
            "Outlinks",
            SharedString::from(rec.outlinks.len().to_string()),
            muted,
        ))
        .child(row(
            "CSR Outlinks",
            SharedString::from(
                rec.outlinks
                    .iter()
                    .filter(|o| o.csr_only)
                    .count()
                    .to_string(),
            ),
            muted,
        ))
        .child(row(
            "CSR Out %",
            SharedString::from({
                let total = rec.outlinks.len();
                let csr_out = rec.outlinks.iter().filter(|o| o.csr_only).count();
                if total > 0 && csr_out > 0 {
                    format!(
                        "{}%",
                        (csr_out as f64 / total as f64 * 100.0).round() as u32
                    )
                } else {
                    "-".into()
                }
            }),
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
            SharedString::from(
                rec.depth
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "-".into()),
            ),
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
        .when(!rec.near_duplicate_urls.is_empty(), |el| {
            let mut urls_body = div().flex().flex_col().gap_0p5();
            for dup_url in &rec.near_duplicate_urls {
                urls_body = urls_body.child(
                    div()
                        .text_xs()
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_color(fg)
                        .truncate()
                        .child(SharedString::from(dup_url.clone())),
                );
            }
            el.child(
                div()
                    .pl(px(0.))
                    .pt_1()
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .pb_0p5()
                            .child("Duplicate Pages:"),
                    )
                    .child(urls_body),
            )
        })
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

    section("Link Metrics", None, None, muted, border, links_body)
}

fn images_section(rec: &PageRecord, muted: Hsla, fg: Hsla, border: Hsla, cx: &App) -> AnyElement {
    let images_summary = if rec.images.is_empty() {
        None
    } else {
        let missing_alt = rec
            .images
            .iter()
            .filter(|img| !img.has_alt_attr || img.alt.as_deref().is_none_or(|a| a.is_empty()))
            .count();
        if missing_alt > 0 {
            Some(
                tone_tag(Tone::Warn, cx)
                    .child(SharedString::from(format!("{} missing alt", missing_alt)))
                    .into_any_element(),
            )
        } else {
            Some(
                tone_tag(Tone::Ok, cx)
                    .child(SharedString::from(format!("{} images", rec.images.len())))
                    .into_any_element(),
            )
        }
    };

    let mut images_body = div().flex().flex_col().gap_1();
    if rec.images.is_empty() {
        images_body = images_body.child(div().text_color(muted).text_sm().child("No images found"));
    } else {
        for image in &rec.images {
            let alt_tag = if image.has_alt_attr {
                if image.alt.as_deref().is_none_or(|a| a.is_empty()) {
                    tone_tag(Tone::Warn, cx).child(SharedString::from("empty"))
                } else {
                    tone_tag(Tone::Ok, cx).child(SharedString::from("yes"))
                }
            } else {
                tone_tag(Tone::Err, cx).child(SharedString::from("missing"))
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
                                            div().text_color(fg).child(SharedString::from(alt)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .when_some(image.width, |el, w| {
                                        el.child(
                                            div()
                                                .text_color(fg)
                                                .child(SharedString::from(format!("{w}px"))),
                                        )
                                    })
                                    .when(image.width.is_some() && image.height.is_some(), |el| {
                                        el.child(div().text_color(muted).child("x"))
                                    })
                                    .when_some(image.height, |el, h| {
                                        el.child(
                                            div()
                                                .text_color(fg)
                                                .child(SharedString::from(format!("{h}px"))),
                                        )
                                    })
                                    .when(image.width.is_none() && image.height.is_none(), |el| {
                                        el.child(div().text_color(muted).child("no dimensions"))
                                    }),
                            ),
                    ),
            );
        }
    }
    let images_section_body = images_body.into_any_element();

    section(
        "Images",
        Some(Icon::Image),
        images_summary,
        muted,
        border,
        images_section_body,
    )
}

fn serp_section(rec: &PageRecord, muted: Hsla, border: Hsla, cx: &App) -> AnyElement {
    section(
        "SERP Preview",
        None,
        None,
        muted,
        border,
        serp_preview(rec, cx),
    )
}

fn hreflang_section(rec: &PageRecord, muted: Hsla, border: Hsla) -> Option<AnyElement> {
    if rec.hreflang_issues.is_empty() {
        return None;
    }
    let mut body = div().flex().flex_col().gap_0p5();
    for issue in &rec.hreflang_issues {
        let label = match issue {
            crate::crawl::event::HreflangIssue::MissingReturnTag { lang, target_url } => {
                format!("Missing return tag: {lang} -> {target_url}")
            }
            crate::crawl::event::HreflangIssue::InvalidLanguageCode { code } => {
                format!("Invalid language code: {code}")
            }
            crate::crawl::event::HreflangIssue::MissingXDefault => "Missing x-default".into(),
            crate::crawl::event::HreflangIssue::MissingSelfReference => {
                "Missing self reference".into()
            }
            crate::crawl::event::HreflangIssue::NonCanonicalUrl { hreflang_url } => {
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
    Some(section(
        "Hreflang Issues",
        None,
        Some(
            div()
                .text_xs()
                .text_color(muted)
                .child(SharedString::from(rec.hreflang_issues.len().to_string()))
                .into_any_element(),
        ),
        muted,
        border,
        body.into_any_element(),
    ))
}

fn inlinks_section(
    rec: &PageRecord,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> Option<AnyElement> {
    if rec.backlinks.is_empty() {
        return None;
    }
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
                    div().text_xs().text_color(muted).child(SharedString::from(
                        bl.anchor
                            .as_deref()
                            .map(|a| format!("Anchor: {a}"))
                            .unwrap_or_else(|| "No anchor".into()),
                    )),
                )
                .when_some(bl.rel.as_deref(), |el, rel| {
                    el.child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(SharedString::from(format!("Rel: {rel}"))),
                    )
                }),
        );
    }
    Some(section(
        "Inlinks (From)",
        None,
        Some(
            div()
                .text_xs()
                .child(SharedString::from(format!("{} links", rec.backlinks.len())))
                .into_any_element(),
        ),
        muted,
        border,
        body.into_any_element(),
    ))
}

fn outlinks_section(
    rec: &PageRecord,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> Option<AnyElement> {
    if rec.outlinks.is_empty() {
        return None;
    }
    let mut body = div().flex().flex_col().gap_0p5();
    // Every link, not the first 50: the Links tab lists one row per URL now, so
    // this section is the only place an individual link's anchor and rel are
    // visible. It builds a child per link, which is what the inlinks section
    // above already does and what virtualising both sections will fix.
    for link in &rec.outlinks {
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
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_color(fg)
                                .child(SharedString::from(link.dst_url.clone())),
                        )
                        .when(is_nofollow, |el| {
                            el.child(tone_tag(Tone::Warn, cx).child(SharedString::from("nofollow")))
                        }),
                )
                .child(
                    div().text_xs().text_color(muted).child(SharedString::from(
                        link.anchor
                            .as_deref()
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "-".into()),
                    )),
                ),
        );
    }
    Some(section(
        "Outlinks (To)",
        None,
        Some(
            div()
                .text_xs()
                .child(SharedString::from(format!("{} links", rec.outlinks.len())))
                .into_any_element(),
        ),
        muted,
        border,
        body.into_any_element(),
    ))
}

fn headers_section(
    rec: &PageRecord,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> Option<AnyElement> {
    if rec.headers.is_empty() {
        return None;
    }
    let mut headers_body = div().flex().flex_col().gap_0p5();
    for (key, value) in &rec.headers {
        // Truncate by characters, not bytes: slicing a multi-byte value at byte
        // 80 would land mid-character and panic.
        let display_value = if value.chars().count() > 80 {
            format!("{}...", value.chars().take(80).collect::<String>())
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
    Some(section(
        "HTTP Headers",
        None,
        Some(
            div()
                .text_xs()
                .child(SharedString::from(format!("{} headers", rec.headers.len())))
                .into_any_element(),
        ),
        muted,
        border,
        headers_body.into_any_element(),
    ))
}

impl Render for DetailsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let fg = theme.foreground;
        let border = theme.border;
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
            Some(rec) => div()
                .id("details-scroll")
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .child(header_block(rec, muted, border))
                .child(url_information_section(rec, muted, border, cx))
                .child(page_content_section(rec, muted, border))
                .child(structured_data_section(
                    rec,
                    &self.json_views,
                    muted,
                    fg,
                    border,
                    cx,
                ))
                .child(accessibility_section(
                    rec,
                    &self.html_views,
                    muted,
                    fg,
                    border,
                    cx,
                ))
                .child(vitals_section(rec, muted, border, panel2))
                .child(link_metrics_section(rec, muted, fg, border, cx))
                .child(images_section(rec, muted, fg, border, cx))
                .child(serp_section(rec, muted, border, cx))
                .when_some(hreflang_section(rec, muted, border), |el, s| el.child(s))
                .when_some(inlinks_section(rec, muted, fg, border, cx), |el, s| {
                    el.child(s)
                })
                .when_some(outlinks_section(rec, muted, fg, border, cx), |el, s| {
                    el.child(s)
                })
                .when_some(headers_section(rec, muted, fg, border, cx), |el, s| {
                    el.child(s)
                })
                .into_any_element(),
        };

        div()
            .w(gpui::px(420.))
            .h_full()
            .border_l_1()
            .border_color(border)
            .rounded_br(crate::app::PANEL_RADIUS)
            .child(body)
    }
}
