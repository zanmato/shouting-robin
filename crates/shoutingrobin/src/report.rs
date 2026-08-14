//! The PDF report: what a crawl found, in the shape the Overview tab shows it,
//! plus the URLs behind each row.
//!
//! The CSV export answers "give me the data". This answers "give me something I
//! can send to whoever owns the site": the same rules, in the same order, each
//! with the count it reports and a sample of the URLs it lands on.
//!
//! # How the pages are made
//!
//! takumi lays out HTML and renders it to a vector SVG; `svg2pdf` turns each of
//! those into a PDF form XObject; one XObject is drawn per page. Nothing here
//! rasterises, so the output stays sharp at any zoom and small on disk.
//!
//! Two consequences worth knowing. takumi emits glyphs as outlines, so the text
//! in the PDF is drawn rather than selectable. And nothing paginates for us: the
//! layout engine renders one viewport at a time, so [`paginate`] packs blocks of
//! known height into pages itself, which is why every block here is a fixed
//! number of fixed-height lines.

use std::collections::HashMap;

use anyhow::{Context as _, Result, anyhow};
use pdf_writer::{Content, Finish, Name, Pdf, Rect, Ref};
use takumi::prelude::{
    FontOverride, FontResource, Fonts, FromHtml, FromHtmlOptions, Node, SvgOptions, Viewport,
};
use takumi::render_svg;

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

/// A4 at 96 dpi, the size a browser assumes, so a length written in pixels here
/// means the same thing it would in a stylesheet.
const PAGE_WIDTH: f32 = 794.0;
const PAGE_HEIGHT: f32 = 1123.0;
const PAGE_MARGIN: f32 = 48.0;
/// Room at the foot of every page for the page number.
const FOOTER_HEIGHT: f32 = 32.0;

fn content_height() -> f32 {
    PAGE_HEIGHT - PAGE_MARGIN * 2.0 - FOOTER_HEIGHT
}

/// One piece of the report, with the height it will occupy. Heights are known
/// rather than measured because pagination has to happen before layout.
#[derive(Clone, Debug)]
enum Block {
    Title,
    SummaryTiles,
    SectionHeading(&'static str),
    TableHeader,
    TableRow(usize),
    IssueDetail(usize),
}

impl Block {
    fn height(&self, report: &Report) -> f32 {
        match self {
            // Site, date and mode under a heading.
            Block::Title => 96.0,
            Block::SummaryTiles => 84.0,
            Block::SectionHeading(_) => 44.0,
            Block::TableHeader => 28.0,
            Block::TableRow(_) => 24.0,
            Block::IssueDetail(index) => {
                let offenders = report
                    .issues
                    .get(*index)
                    .map(|issue| issue.offenders.len())
                    .unwrap_or(0);
                // Name and count, description, hint, then one line per URL.
                62.0 + 18.0 * offenders as f32 + 12.0
            }
        }
    }
}

/// Packs blocks into pages, keeping each block whole.
///
/// A block never splits across a page boundary, which is what keeps a rule's
/// URLs under the rule they belong to. A block taller than a page goes on a
/// page of its own and overflows it rather than disappearing, which cannot
/// happen at [`MAX_OFFENDERS`] but would be a silent hole if it ever did.
fn paginate(blocks: &[Block], report: &Report) -> Vec<Vec<Block>> {
    let limit = content_height();
    let mut pages: Vec<Vec<Block>> = Vec::new();
    let mut current: Vec<Block> = Vec::new();
    let mut used = 0.0;
    let mut index = 0;

    while index < blocks.len() {
        // A heading is glued to whatever follows it, so a page never ends with
        // a title standing over nothing.
        let group_len = match blocks[index] {
            Block::SectionHeading(_) | Block::TableHeader if index + 1 < blocks.len() => 2,
            _ => 1,
        };
        let group = &blocks[index..index + group_len];
        let height: f32 = group.iter().map(|block| block.height(report)).sum();

        if !current.is_empty() && used + height > limit {
            pages.push(std::mem::take(&mut current));
            used = 0.0;
            // A table carrying on over the break repeats its header, or the
            // second page of rows is five unlabelled columns.
            if matches!(group.first(), Some(Block::TableRow(_))) {
                used += Block::TableHeader.height(report);
                current.push(Block::TableHeader);
            }
        }
        used += height;
        current.extend_from_slice(group);
        index += group_len;
    }
    if !current.is_empty() {
        pages.push(current);
    }
    pages
}

fn blocks_for(report: &Report) -> Vec<Block> {
    let mut blocks = vec![Block::Title, Block::SummaryTiles];
    if !report.issues.is_empty() {
        blocks.push(Block::SectionHeading("Issues found"));
        blocks.push(Block::TableHeader);
        for index in 0..report.issues.len() {
            blocks.push(Block::TableRow(index));
        }
        blocks.push(Block::SectionHeading("The URLs behind each issue"));
        for (index, issue) in report.issues.iter().enumerate() {
            if issue.offenders.is_empty() {
                continue;
            }
            blocks.push(Block::IssueDetail(index));
        }
    }
    blocks
}

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

fn render_block(block: &Block, report: &Report) -> String {
    match block {
        Block::Title => format!(
            r#"<div style="display:flex;flex-direction:column;gap:6px;padding-bottom:20px">
                 <div style="font-size:13px;color:#64748b;letter-spacing:2px">SEO CRAWL REPORT</div>
                 <div style="font-size:30px;font-weight:700;color:#0f172a">{site}</div>
                 <div style="font-size:13px;color:#64748b">{generated} · {mode}</div>
               </div>"#,
            site = escape(&truncate(&report.site, 60)),
            generated = escape(&report.generated_at),
            mode = escape(&report.render_mode),
        ),
        Block::SummaryTiles => {
            let tiles: String = report
                .summary
                .iter()
                .map(|(label, value)| {
                    format!(
                        r#"<div style="display:flex;flex-direction:column;gap:4px;padding:10px 14px;background-color:#f1f5f9;border-radius:6px;width:140px">
                             <div style="font-size:11px;color:#64748b">{label}</div>
                             <div style="font-size:20px;font-weight:600;color:#0f172a">{value}</div>
                           </div>"#,
                        label = escape(label),
                        value = escape(value),
                    )
                })
                .collect();
            format!(
                r#"<div style="display:flex;flex-direction:row;gap:10px;padding-bottom:16px">{tiles}</div>"#
            )
        }
        Block::SectionHeading(text) => format!(
            r#"<div style="display:flex;flex-direction:column;gap:8px;padding-top:12px;padding-bottom:10px">
                 <div style="font-size:17px;font-weight:600;color:#0f172a">{text}</div>
                 <div style="height:1px;background-color:#e2e8f0"></div>
               </div>"#
        ),
        Block::TableHeader => cells(
            "ISSUE",
            "TYPE",
            "PRIORITY",
            "URLS",
            "% OF TOTAL",
            "#64748b",
            "#64748b",
            11,
        ),
        Block::TableRow(index) => {
            let Some(issue) = report.issues.get(*index) else {
                return String::new();
            };
            cells(
                &truncate(&issue.name, 46),
                &issue.issue_type,
                &issue.priority,
                &issue.count.to_string(),
                &format!("{:.1}%", issue.pct),
                type_color(&issue.issue_type),
                priority_color(&issue.priority),
                12,
            )
        }
        Block::IssueDetail(index) => {
            let Some(issue) = report.issues.get(*index) else {
                return String::new();
            };
            let urls: String = issue
                .offenders
                .iter()
                .map(|url| {
                    format!(
                        r#"<div style="font-size:11px;color:#334155;height:18px">{}</div>"#,
                        escape(&truncate(url, 96))
                    )
                })
                .collect();
            let more = if issue.count > issue.offenders.len() {
                format!(
                    r#"<div style="font-size:11px;color:#94a3b8;height:18px">and {} more</div>"#,
                    issue.count - issue.offenders.len()
                )
            } else {
                String::new()
            };
            format!(
                r#"<div style="display:flex;flex-direction:column;gap:3px;padding-bottom:12px">
                     <div style="display:flex;flex-direction:row;gap:8px">
                       <div style="font-size:13px;font-weight:600;color:{color}">{name}</div>
                       <div style="font-size:13px;color:#64748b">{count} URLs</div>
                     </div>
                     <div style="font-size:11px;color:#475569">{description}</div>
                     <div style="font-size:11px;color:#64748b">{hint}</div>
                     {urls}{more}
                   </div>"#,
                color = type_color(&issue.issue_type),
                name = escape(&truncate(&issue.name, 60)),
                count = issue.count,
                description = escape(&truncate(&issue.description, 120)),
                hint = escape(&truncate(&issue.hint, 120)),
            )
        }
    }
}

/// One row of the issues table. The same widths are used for the header and the
/// rows, so the columns line up without a table layout, which takumi has no
/// support for.
#[allow(clippy::too_many_arguments)]
fn cells(
    name: &str,
    issue_type: &str,
    priority: &str,
    count: &str,
    pct: &str,
    type_color: &str,
    priority_color: &str,
    font_size: u32,
) -> String {
    format!(
        r#"<div style="display:flex;flex-direction:row;align-items:center;height:24px;gap:8px">
             <div style="width:330px;font-size:{font_size}px;color:#0f172a">{name}</div>
             <div style="width:90px;font-size:{font_size}px;color:{type_color}">{issue_type}</div>
             <div style="width:80px;font-size:{font_size}px;color:{priority_color}">{priority}</div>
             <div style="width:60px;font-size:{font_size}px;color:#0f172a;text-align:right">{count}</div>
             <div style="width:80px;font-size:{font_size}px;color:#0f172a;text-align:right">{pct}</div>
           </div>"#,
        name = escape(name),
        issue_type = escape(issue_type),
        priority = escape(priority),
        count = escape(count),
        pct = escape(pct),
    )
}

fn page_html(blocks: &[Block], report: &Report, page: usize, total: usize) -> String {
    let body: String = blocks
        .iter()
        .map(|block| render_block(block, report))
        .collect();
    format!(
        r#"<div style="display:flex;flex-direction:column;width:{width}px;height:{height}px;padding:{margin}px;background-color:#ffffff;font-family:report">
             <div style="display:flex;flex-direction:column;flex-grow:1">{body}</div>
             <div style="display:flex;flex-direction:row;justify-content:space-between;height:{footer}px;align-items:flex-end">
               <div style="font-size:10px;color:#94a3b8">{site}</div>
               <div style="font-size:10px;color:#94a3b8">Page {page} of {total}</div>
             </div>
           </div>"#,
        width = PAGE_WIDTH,
        height = PAGE_HEIGHT,
        margin = PAGE_MARGIN,
        footer = FOOTER_HEIGHT,
        site = escape(&truncate(&report.site, 60)),
    )
}

/// The font the report is set in.
///
/// takumi registers no system fonts, so one has to be handed to it. usvg's
/// font database is already in the dependency tree through `svg2pdf`, so the
/// lookup costs nothing extra: ask it for the platform's sans-serif and read
/// the face's bytes back out.
fn report_fonts() -> Result<Fonts> {
    use svg2pdf::usvg::fontdb::{Database, Family, Query};

    let mut database = Database::new();
    database.load_system_fonts();

    // `Family::SansSerif` resolves to one hardcoded family name (Arial), so a
    // machine without that exact font answers "no sans-serif" while holding
    // several. Ask for the usual names too, then take whatever is installed:
    // any face beats failing to write the report.
    let named = [
        "DejaVu Sans",
        "Liberation Sans",
        "Noto Sans",
        "Helvetica",
        "Ubuntu",
        "Cantarell",
    ];
    let id = std::iter::once(Family::SansSerif)
        .chain(named.iter().copied().map(Family::Name))
        .find_map(|family| {
            database.query(&Query {
                families: &[family],
                ..Default::default()
            })
        })
        .or_else(|| database.faces().next().map(|face| face.id))
        .ok_or_else(|| anyhow!("no sans-serif font is installed"))?;
    let bytes = database
        .with_face_data(id, |data, _index| data.to_vec())
        .ok_or_else(|| anyhow!("the sans-serif font has no readable data"))?;

    let mut fonts = Fonts::default();
    fonts
        // Registered under a name of our own, so the markup can ask for
        // "report" rather than whatever the platform's sans-serif happens to
        // be called.
        .register(FontResource::new(bytes).override_info(FontOverride {
            family_name: Some("report".into()),
            ..Default::default()
        }))
        .map_err(|err| anyhow!("failed to register the report font: {err}"))?;
    Ok(fonts)
}

/// Lays the report out and returns the bytes of a PDF.
pub fn render_pdf(report: &Report) -> Result<Vec<u8>> {
    let pages = paginate(&blocks_for(report), report);
    if pages.is_empty() {
        return Err(anyhow!("there is nothing to report"));
    }
    let fonts = report_fonts()?;
    let total = pages.len();

    let mut svgs = Vec::with_capacity(total);
    for (index, blocks) in pages.iter().enumerate() {
        let html = page_html(blocks, report, index + 1, total);
        let node = Node::from_html(&html, FromHtmlOptions::default())
            .map_err(|err| anyhow!("the report markup is not usable: {err}"))?;
        let options = SvgOptions::builder()
            .viewport(Viewport::new((PAGE_WIDTH as u32, PAGE_HEIGHT as u32)))
            .node(node)
            .fonts(&fonts)
            .build();
        svgs.push(render_svg(options).map_err(|err| anyhow!("failed to lay out a page: {err}"))?);
    }

    assemble_pdf(&svgs)
}

/// Draws one page per SVG.
///
/// `svg2pdf` hands back a form XObject normalised to a unit square, so each
/// page scales it back up to the page box. The chunks it produces number their
/// own objects from 1, so they are renumbered into this document as they are
/// merged.
fn assemble_pdf(svgs: &[String]) -> Result<Vec<u8>> {
    let mut pdf = Pdf::new();
    let mut next_id = 1;
    let mut alloc = move || {
        let id = Ref::new(next_id);
        next_id += 1;
        id
    };

    let catalog_id = alloc();
    let page_tree_id = alloc();

    let options = svg2pdf::usvg::Options::default();
    let conversion = svg2pdf::ConversionOptions::default();

    let mut page_ids = Vec::with_capacity(svgs.len());
    let mut pending = Vec::with_capacity(svgs.len());
    for svg in svgs {
        let tree = svg2pdf::usvg::Tree::from_str(svg, &options)
            .context("the rendered page is not usable SVG")?;
        let (chunk, xobject_id) = svg2pdf::to_chunk(&tree, conversion)
            .map_err(|err| anyhow!("failed to convert a page to PDF: {err}"))?;

        let mut remapped = HashMap::new();
        chunk.renumber_into(&mut pdf, |old| {
            *remapped.entry(old).or_insert_with(&mut alloc)
        });
        let xobject_id = *remapped
            .get(&xobject_id)
            .ok_or_else(|| anyhow!("a converted page lost its content"))?;

        let page_id = alloc();
        let content_id = alloc();
        page_ids.push(page_id);
        pending.push((page_id, content_id, xobject_id));
    }

    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id)
        .count(page_ids.len() as i32)
        .kids(page_ids.iter().copied());

    // PDF's unit is a point, so a page in 96 dpi pixels is scaled by 72/96.
    let width = PAGE_WIDTH * 72.0 / 96.0;
    let height = PAGE_HEIGHT * 72.0 / 96.0;
    for (page_id, content_id, xobject_id) in pending {
        let mut page = pdf.page(page_id);
        page.parent(page_tree_id)
            .media_box(Rect::new(0.0, 0.0, width, height))
            .contents(content_id);
        page.resources()
            .x_objects()
            .pair(Name(b"S"), xobject_id)
            .finish();
        page.finish();

        let mut content = Content::new();
        content.transform([width, 0.0, 0.0, height, 0.0, 0.0]);
        content.x_object(Name(b"S"));
        pdf.stream(content_id, &content.finish());
    }

    Ok(pdf.finish())
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
            render_mode: "HTTP".into(),
            summary: vec![("URLs crawled".into(), "125".into())],
            issues: (0..issue_count)
                .map(|index| issue(&format!("Rule number {index}"), offenders))
                .collect(),
        }
    }

    #[test]
    fn a_short_report_is_one_page() {
        let report = report(3, 2);
        let pages = paginate(&blocks_for(&report), &report);
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn a_long_report_spills_onto_further_pages() {
        let report = report(40, MAX_OFFENDERS);
        let pages = paginate(&blocks_for(&report), &report);
        assert!(pages.len() > 3, "got {} pages", pages.len());
        let limit = content_height();
        for (index, page) in pages.iter().enumerate() {
            let used: f32 = page.iter().map(|block| block.height(&report)).sum();
            assert!(
                used <= limit || page.len() == 1,
                "page {} overflows at {used}px",
                index + 1
            );
        }
    }

    #[test]
    fn every_issue_keeps_its_row_and_its_urls() {
        let report = report(40, 4);
        let pages = paginate(&blocks_for(&report), &report);
        let mut rows = 0;
        let mut details = 0;
        for block in pages.iter().flatten() {
            match block {
                Block::TableRow(_) => rows += 1,
                Block::IssueDetail(_) => details += 1,
                _ => {}
            }
        }
        assert_eq!(rows, 40, "every rule keeps its row in the table");
        assert_eq!(details, 40, "and its own list of URLs");
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

    /// The whole pipeline, on a report small enough to keep the test quick:
    /// layout, SVG, and a PDF with one page per laid-out page.
    #[test]
    fn a_report_renders_to_a_pdf() {
        let report = report(6, 3);
        let bytes = match render_pdf(&report) {
            Ok(bytes) => bytes,
            // A machine with no fonts installed cannot lay out text, and that
            // is the environment's business rather than a defect here.
            Err(err) if err.to_string().contains("no sans-serif font") => return,
            Err(err) => panic!("{err}"),
        };
        assert!(bytes.starts_with(b"%PDF-"), "not a PDF");
        assert!(bytes.len() > 1000, "suspiciously small: {}", bytes.len());
    }
}
