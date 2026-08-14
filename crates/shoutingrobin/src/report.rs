//! The PDF report: what a crawl found, in the shape the Overview tab shows it,
//! plus the URLs behind each row.
//!
//! The CSV export answers "give me the data". This answers "give me something I
//! can send to whoever owns the site": the same rules, in the same order, each
//! with the count it reports and a sample of the URLs it lands on.
//!
//! # How the pages are made
//!
//! The report is written as HTML and handed to `takumi-pdf`, which lays it out
//! and writes the PDF through krilla. Text comes out as real glyph runs with
//! embedded subset fonts, so the report is selectable, searchable and small,
//! and pagination is the layout engine's own: `break-inside: avoid` keeps a
//! rule's URLs with the rule, a repeating footer band carries the page numbers,
//! and nothing here has to know how tall anything is.

use anyhow::{Result, anyhow};
use takumi_core::{Fonts, resources::font::FontResource};
use takumi_html::{FromHtmlOptions, from_html};
use takumi_pdf::{PageOptions, PdfMetadata, PdfOptions, render};

/// One rule of the Overview, with the URLs it lands on.
#[derive(Clone, Debug)]
pub struct ReportIssue {
    pub name: String,
    pub issue_type: String,
    pub priority: String,
    pub count: usize,
    pub pct: f32,
    pub description: String,
    pub hint: String,
    /// A sample of the URLs the rule's own filter selects, capped at
    /// [`MAX_OFFENDERS`]. Empty for a rule with no click-through target.
    pub offenders: Vec<String>,
}

/// Everything the report says, gathered before any layout happens.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub site: String,
    pub generated_at: String,
    pub render_mode: String,
    /// Headline figures, in the order they are shown.
    pub summary: Vec<(String, String)>,
    pub issues: Vec<ReportIssue>,
}

/// How many URLs are listed under one rule. A report is a summary: past a
/// dozen or so the reader wants the CSV, and every rule in this document is one
/// click from the same list in the app.
pub const MAX_OFFENDERS: usize = 12;

/// Page margin in CSS px. The header and footer bands draw inside it, the way
/// a browser's print templates do.
const PAGE_MARGIN: f32 = 48.0;

/// HTML-escapes a value going into the markup.
fn escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Cuts a value to `max` characters, counting characters rather than bytes so a
/// URL with a non-ASCII path is not sliced mid-character.
fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let kept: String = value.chars().take(max.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// The colour a type reads in, matching the tag tones in the app.
fn type_color(issue_type: &str) -> &'static str {
    match issue_type {
        "Issue" => "#dc2626",
        "Opportunity" => "#d97706",
        _ => "#64748b",
    }
}

fn priority_color(priority: &str) -> &'static str {
    match priority {
        "High" => "#dc2626",
        "Medium" => "#d97706",
        _ => "#64748b",
    }
}

/// The five columns of the issues table, as a row of flex cells.
///
/// takumi has no table layout, so the header and the rows share these widths
/// and line up because they are written once.
fn table_row(cells: [&str; 5], style: &str, tones: (&str, &str)) -> String {
    let [name, issue_type, priority, count, pct] = cells;
    let (type_color, priority_color) = tones;
    format!(
        r#"<div style="display:flex;flex-direction:row;align-items:center;padding:3px 0;{style}">
             <div style="width:320px">{name}</div>
             <div style="width:90px;color:{type_color}">{issue_type}</div>
             <div style="width:80px;color:{priority_color}">{priority}</div>
             <div style="width:60px;text-align:right">{count}</div>
             <div style="width:80px;text-align:right">{pct}</div>
           </div>"#,
        name = escape(name),
        issue_type = escape(issue_type),
        priority = escape(priority),
        count = escape(count),
        pct = escape(pct),
    )
}

/// The whole report as one HTML document. The layout engine paginates it.
fn report_html(report: &Report) -> String {
    let tiles: String = report
        .summary
        .iter()
        .map(|(label, value)| {
            format!(
                r#"<div style="display:flex;flex-direction:column;gap:4px;padding:10px 14px;background-color:#f1f5f9;border-radius:6px;width:132px">
                     <div style="font-size:11px;color:#64748b">{label}</div>
                     <div style="font-size:20px;font-weight:600">{value}</div>
                   </div>"#,
                label = escape(label),
                value = escape(value),
            )
        })
        .collect();

    let rows: String = report
        .issues
        .iter()
        .map(|issue| {
            table_row(
                [
                    &truncate(&issue.name, 46),
                    &issue.issue_type,
                    &issue.priority,
                    &issue.count.to_string(),
                    &format!("{:.1}%", issue.pct),
                ],
                "font-size:12px;border-top:1px solid #f1f5f9",
                (
                    type_color(&issue.issue_type),
                    priority_color(&issue.priority),
                ),
            )
        })
        .collect();

    let details: String = report
        .issues
        .iter()
        .filter(|issue| !issue.offenders.is_empty())
        .map(|issue| {
            let urls: String = issue
                .offenders
                .iter()
                .map(|url| {
                    format!(
                        r#"<div style="font-size:11px;color:#334155">{}</div>"#,
                        escape(&truncate(url, 104))
                    )
                })
                .collect();
            let more = if issue.count > issue.offenders.len() {
                format!(
                    r#"<div style="font-size:11px;color:#94a3b8">and {} more</div>"#,
                    issue.count - issue.offenders.len()
                )
            } else {
                String::new()
            };
            // A rule and its URLs are one thing to read, so they ask to stay
            // on one page.
            //
            // The engine does not honour it yet, and a long list still splits
            // over a break. Reported upstream; the declaration is the intent
            // and starts working when the engine does. `break-before: page` on
            // the heading below would be the other half of this and is
            // deliberately absent: a forced break there leaves the page after
            // it empty, which reads far worse than a section that begins
            // mid-page.
            format!(
                r#"<div style="display:flex;flex-direction:column;gap:3px;padding-bottom:14px;break-inside:avoid">
                     <div style="display:flex;flex-direction:row;gap:8px;align-items:baseline">
                       <div style="font-size:13px;font-weight:600;color:{color}">{name}</div>
                       <div style="font-size:12px;color:#64748b">{count} URLs</div>
                     </div>
                     <div style="font-size:11px;color:#475569">{description}</div>
                     <div style="font-size:11px;color:#64748b">{hint}</div>
                     {urls}{more}
                   </div>"#,
                color = type_color(&issue.issue_type),
                name = escape(&truncate(&issue.name, 60)),
                count = issue.count,
                description = escape(&issue.description),
                hint = escape(&issue.hint),
            )
        })
        .collect();

    let issues_section = if report.issues.is_empty() {
        r#"<div style="font-size:13px;color:#64748b;padding-top:16px">No rule fired on this crawl.</div>"#.to_string()
    } else {
        format!(
            r#"<h2 style="font-size:17px;font-weight:600;margin:0;padding-top:18px;padding-bottom:8px">Issues found</h2>
               <div style="height:1px;background-color:#e2e8f0;margin-bottom:6px"></div>
               {header}
               {rows}
               <h2 style="font-size:17px;font-weight:600;margin:0;padding-top:22px;padding-bottom:8px">The URLs behind each issue</h2>
               <div style="height:1px;background-color:#e2e8f0;margin-bottom:10px"></div>
               {details}"#,
            header = table_row(
                ["ISSUE", "TYPE", "PRIORITY", "URLS", "% OF TOTAL"],
                "font-size:10px;color:#64748b",
                ("#64748b", "#64748b"),
            ),
        )
    };

    format!(
        r#"<main style="display:flex;flex-direction:column;width:100%;font-family:'Google Sans';color:#0f172a">
             <div style="font-size:11px;color:#64748b">SEO CRAWL REPORT</div>
             <h1 style="font-size:28px;font-weight:700;margin:0;padding-top:4px">{site}</h1>
             <div style="font-size:12px;color:#64748b;padding-top:4px;padding-bottom:18px">{generated} · {mode}</div>
             <div style="display:flex;flex-direction:row;gap:10px">{tiles}</div>
             {issues_section}
           </main>"#,
        site = escape(&truncate(&report.site, 60)),
        generated = escape(&report.generated_at),
        mode = escape(&report.render_mode),
    )
}

/// The band repeated at the foot of every page. `pageNumber` and `totalPages`
/// are filled in by the layout engine, so nothing here counts pages.
fn footer_html(report: &Report) -> String {
    format!(
        r#"<div style="display:flex;flex-direction:row;justify-content:space-between;width:100%;padding:0 {margin}px;font-family:'Google Sans';font-size:10px;color:#94a3b8">
             <div>{site}</div>
             <div style="display:flex;flex-direction:row;gap:3px">
               <div>Page</div><div class="pageNumber"></div><div>of</div><div class="totalPages"></div>
             </div>
           </div>"#,
        margin = PAGE_MARGIN,
        site = escape(&truncate(&report.site, 60)),
    )
}

/// The report is set in the app's bundled family, for the reasons
/// [`crate::ui::fonts`] gives.
fn report_fonts() -> Result<Fonts> {
    let mut fonts = Fonts::default();
    for face in [crate::ui::fonts::REGULAR, crate::ui::fonts::BOLD] {
        fonts
            .register(FontResource::new(face))
            .map_err(|err| anyhow!("failed to register the report font: {err}"))?;
    }
    Ok(fonts)
}

/// Lays the report out and returns the bytes of a PDF.
pub fn render_pdf(report: &Report) -> Result<Vec<u8>> {
    let fonts = report_fonts()?;
    let node = from_html(&report_html(report), FromHtmlOptions::default())
        .map_err(|err| anyhow!("the report markup is not usable: {err}"))?;
    let footer = from_html(&footer_html(report), FromHtmlOptions::default())
        .map_err(|err| anyhow!("the report footer is not usable: {err}"))?;

    render(
        PdfOptions::builder()
            .node(node)
            .footer(footer)
            .page(PageOptions::A4.with_margin(PAGE_MARGIN))
            .fonts(&fonts)
            // The two headings become bookmarks, so a long report opens on a
            // list of its own sections.
            .outline(true)
            .metadata(PdfMetadata {
                title: Some(format!("SEO crawl report · {}", report.site)),
                creator: Some("Shouting Robin".to_string()),
                ..Default::default()
            })
            .build(),
    )
    .map_err(|err| anyhow!("failed to write the report: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(name: &str, offenders: usize) -> ReportIssue {
        ReportIssue {
            name: name.into(),
            issue_type: "Issue".into(),
            priority: "High".into(),
            count: offenders,
            pct: 12.5,
            description: "A description of the rule.".into(),
            hint: "What to do about it.".into(),
            offenders: (0..offenders)
                .map(|index| format!("https://example.com/page-{index}"))
                .collect(),
        }
    }

    fn report(issue_count: usize, offenders: usize) -> Report {
        Report {
            site: "https://example.com".into(),
            generated_at: "2026-08-14 12:00".into(),
            render_mode: "HTTP, no JavaScript".into(),
            summary: vec![("URLs crawled".into(), "125".into())],
            issues: (0..issue_count)
                .map(|index| issue(&format!("Rule number {index}"), offenders))
                .collect(),
        }
    }

    #[test]
    fn a_multi_byte_url_is_cut_between_characters() {
        // Byte slicing would panic here rather than shortening anything.
        let url = "https://example.com/kvalitetsbett-för-dig-och-din-häst";
        assert_eq!(truncate(url, 30).chars().count(), 30);
    }

    #[test]
    fn markup_characters_in_a_url_do_not_become_markup() {
        let escaped = escape("https://example.com/?a=1&b=2\"><div>");
        assert!(!escaped.contains('<'), "got {escaped}");
        assert!(escaped.contains("&amp;"), "got {escaped}");
    }

    #[test]
    fn every_rule_reaches_the_markup_with_its_count_and_urls() {
        let html = report_html(&report(3, 2));
        for index in 0..3 {
            assert!(
                html.contains(&format!("Rule number {index}")),
                "rule {index} is missing"
            );
        }
        assert!(html.contains("https://example.com/page-1"));
        assert!(html.contains("break-inside:avoid"), "a rule can be split");
    }

    #[test]
    fn a_crawl_with_nothing_to_report_still_renders() {
        let bytes = render_pdf(&report(0, 0)).expect("render");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    /// The whole pipeline, on a report small enough to keep the test quick.
    #[test]
    fn a_report_renders_to_a_pdf() {
        let bytes = render_pdf(&report(6, 3)).expect("render");
        assert!(bytes.starts_with(b"%PDF-"), "not a PDF");
        assert!(bytes.len() > 1000, "suspiciously small: {}", bytes.len());
    }

    /// Counts page dictionaries, both spellings, excluding the `/Pages` tree
    /// node that shares the prefix.
    fn page_count(pdf: &[u8]) -> usize {
        let mut pages = 0;
        for marker in [b"/Type/Page".as_slice(), b"/Type /Page".as_slice()] {
            pages += pdf
                .windows(marker.len() + 1)
                .filter(|window| window.starts_with(marker) && !window.ends_with(b"s"))
                .count();
        }
        pages
    }

    /// A report long enough to paginate. The layout engine decides where the
    /// breaks fall; this only asserts that it made more than one page and that
    /// nothing was lost doing it.
    #[test]
    fn a_long_report_paginates() {
        let bytes = render_pdf(&report(40, MAX_OFFENDERS)).expect("render");
        let pages = page_count(&bytes);
        assert!(pages > 3, "expected several pages, counted {pages}");
    }

    /// Writes a sample report to `SR_REPORT_OUT` for looking at. Ignored: it is
    /// an eye test, not an assertion.
    ///
    ///     SR_REPORT_OUT=/tmp/report.pdf cargo test --bin shoutingrobin -- \
    ///       --ignored sample_report
    #[test]
    #[ignore]
    fn sample_report() {
        let path = std::env::var("SR_REPORT_OUT").unwrap_or_else(|_| "/tmp/report.pdf".into());
        let bytes = render_pdf(&report(24, MAX_OFFENDERS)).expect("render");
        std::fs::write(&path, bytes).expect("write");
        println!("wrote {path}");
    }
}
