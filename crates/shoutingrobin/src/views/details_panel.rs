use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::base::TextSelection;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme, Icon as UiIcon, IconName, Sizable as _, VirtualListScrollHandle,
    scroll::ScrollableElement as _, tooltip::Tooltip, v_virtual_list,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, ClipboardItem, Context, Entity, EventEmitter, Hsla,
    InteractiveElement, IntoElement, ParentElement, Pixels, Render, SharedString, Size,
    StatefulInteractiveElement, Styled, Task, Window, div, prelude::FluentBuilder, px,
};

use crate::a11y_rules::rule_description;
use crate::crawl::RenderMode;
use crate::crawl::event::{A11yIssue, PageRecord, SdFormat, SdItem, SdSeverity};
use crate::ui::icon::Icon;
use crate::ui::selectable_text::SelectableText;
use crate::ui::tag::{Tone, indexability_tone, status_code_tone, tone_tag};
use crate::views::results_grid::ssr_diff_label;
use shoutingrobin_ui::{HtmlView, JsonView};

/// One page's use of an image: the Images tab's drill-down, and how a resource
/// row shows the pages that load it.
#[derive(Clone, Debug)]
pub struct ImageReference {
    pub page_url: String,
    pub alt: Option<String>,
    pub has_alt_attr: bool,
}

/// A unique image source and every page referencing it. The Images tab lists
/// one row per source, so this is where the per-page detail the flattened tab
/// used to show now lives.
#[derive(Clone, Debug)]
pub struct ImageDetails {
    pub src: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub references: Vec<ImageReference>,
}

/// What the panel is inspecting. Most tabs select a page; the Images tab
/// selects an image source, which is not one page's property.
#[derive(Clone, Debug)]
pub enum DetailsSelection {
    Page {
        record: Box<PageRecord>,
        /// The pages using this URL as an image. Inlinks only lists anchors,
        /// so without these a broken image on Response Codes has no page to
        /// fix it on.
        references: Vec<ImageReference>,
        /// The outlinks in the order the Outlinks tab lists them, as indices
        /// into `record.outlinks` with the destination's crawled status code,
        /// `None` for one not crawled. See `outlink_order` in the grid.
        outlink_order: Vec<(usize, Option<u16>)>,
    },
    Image(Box<ImageDetails>),
}

impl DetailsSelection {
    /// The pages that use the selection as an image.
    fn references(&self) -> &[ImageReference] {
        match self {
            Self::Page { references, .. } => references,
            Self::Image(image) => &image.references,
        }
    }
}

/// One pane of the details panel.
///
/// The panel used to be a single scrolling column with each long list scrolling
/// inside it. Two scroll areas under one pointer cannot be aimed at: the wheel
/// drives whichever is under it and the other moves too. One list per tab means
/// one scrollable region on screen at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailsTab {
    /// Everything that is a fact about the URL rather than a list.
    Overview,
    NearDuplicates,
    Images,
    Inlinks,
    Outlinks,
    /// The pages referencing a selected image.
    References,
}

impl DetailsTab {
    fn label(self) -> SharedString {
        match self {
            Self::Overview => t!("details.tab.overview"),
            Self::NearDuplicates => t!("details.tab.duplicates"),
            Self::Images => t!("details.tab.images"),
            Self::Inlinks => t!("details.tab.inlinks"),
            Self::Outlinks => t!("details.tab.outlinks"),
            Self::References => t!("details.tab.referenced_by"),
        }
        .into()
    }
}

pub struct DetailsPanel {
    selected: Option<DetailsSelection>,
    active_tab: DetailsTab,
    json_views: Vec<Option<JsonView>>,
    html_views: Vec<Option<HtmlView>>,
    /// One handle per virtualised section, held here rather than built during
    /// render. A scroll handle owns the offset: `v_virtual_list` makes a fresh
    /// one per call, and an element that tracks a scroll handle reads its
    /// offset instead of the offset the window persisted, so a list without a
    /// handle of its own is redrawn at zero every frame and cannot be scrolled
    /// at all.
    inlinks_scroll: VirtualListScrollHandle,
    outlinks_scroll: VirtualListScrollHandle,
    duplicates_scroll: VirtualListScrollHandle,
    image_references_scroll: VirtualListScrollHandle,
    /// The mode the crawl on screen ran in. Accessibility violations and Core
    /// Web Vitals are measured in a browser, so for an HTTP crawl those two
    /// sections can only ever be a column of dashes.
    render_mode: RenderMode,
    /// Reads the selected page's backlinks. Replaced, and so cancelled, by the
    /// next selection.
    on_demand_load: Option<Task<()>>,
}

impl DetailsPanel {
    pub fn new() -> Self {
        Self {
            selected: None,
            active_tab: DetailsTab::Overview,
            json_views: Vec::new(),
            html_views: Vec::new(),
            inlinks_scroll: VirtualListScrollHandle::new(),
            on_demand_load: None,
            outlinks_scroll: VirtualListScrollHandle::new(),
            duplicates_scroll: VirtualListScrollHandle::new(),
            image_references_scroll: VirtualListScrollHandle::new(),
            render_mode: RenderMode::Chrome,
        }
    }

    pub fn set_render_mode(&mut self, mode: RenderMode, cx: &mut Context<Self>) {
        self.render_mode = mode;
        cx.notify();
    }

    /// Reads what a page's details show that the crawl was loaded without,
    /// see [`crate::storage::load_page_on_demand`].
    fn load_on_demand(&self, crawl_id: i64, url: String, cx: &mut Context<Self>) -> Task<()> {
        let crawls = crate::app_database::AppDatabase::global(cx)
            .crawls()
            .clone();
        cx.spawn(async move |panel, cx| {
            let loaded = cx
                .background_spawn({
                    let url = url.clone();
                    async move {
                        let db = crawls.open(crawl_id).await?;
                        crate::storage::load_page_on_demand(&db, &url).await
                    }
                })
                .await;
            let loaded = match loaded {
                Ok(loaded) => loaded,
                Err(error) => {
                    tracing::error!(%error, %url, "failed to read the page's backlinks and structured data");
                    return;
                }
            };
            let filled = panel.update(cx, |panel, cx| {
                if let Some(DetailsSelection::Page { record, .. }) = &mut panel.selected
                    && record.url == url
                {
                    record.backlinks = loaded.backlinks.into();
                    record.sd_items = loaded.sd_items.into();
                    panel.json_views = json_views(&record.sd_items, cx);
                    cx.notify();
                }
            });
            if let Err(error) = filled {
                tracing::debug!(%error, "the details panel closed before the page's details arrived");
            }
        })
    }

    /// Shows `selection`, a row of crawl `crawl_id`. A page's backlinks and
    /// structured data JSON are not loaded with the crawl, so they are read
    /// for it here. Without a crawl the record's own are shown.
    pub fn set_selected(
        &mut self,
        selection: Option<DetailsSelection>,
        crawl_id: Option<i64>,
        cx: &mut Context<Self>,
    ) {
        self.on_demand_load = match (&selection, crawl_id) {
            (Some(DetailsSelection::Page { record, .. }), Some(crawl_id)) => {
                Some(self.load_on_demand(crawl_id, record.url.clone(), cx))
            }
            _ => None,
        };
        let record = match &selection {
            Some(DetailsSelection::Page { record, .. }) => Some(record.as_ref()),
            _ => None,
        };
        self.json_views = match record {
            Some(rec) => json_views(&rec.sd_items, cx),
            None => Vec::new(),
        };
        self.html_views = match record {
            Some(rec) => rec
                .a11y_issues
                .iter()
                .enumerate()
                .map(|(index, issue)| {
                    issue
                        .html
                        .as_deref()
                        .filter(|h| !h.is_empty())
                        .map(|h| HtmlView::new(("a11y-html", index), h, cx).copyable())
                })
                .collect(),
            None => Vec::new(),
        };
        // A new selection is a new set of rows, so every list starts at the top
        // rather than inheriting the previous page's position.
        for handle in [
            &self.inlinks_scroll,
            &self.outlinks_scroll,
            &self.duplicates_scroll,
            &self.image_references_scroll,
        ] {
            handle.set_offset(gpui_kit::point(px(0.), px(0.)));
        }
        self.selected = selection;
        // A new URL is a new set of tabs, and the one that was open may not
        // exist for it.
        self.active_tab = DetailsTab::Overview;
        cx.notify();
    }
}

/// One JSON view per structured data item, `None` for an item whose JSON has
/// not been read yet.
fn json_views(items: &[SdItem], cx: &App) -> Vec<Option<JsonView>> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            (!item.raw_json.is_empty())
                .then(|| JsonView::new(("sd-json", index), &item.raw_json, cx).copyable())
        })
        .collect()
}

/// Asks the app to open a page's layout, which needs the crawl and its
/// baseline from the database.
#[derive(Clone, Debug)]
pub enum DetailsPanelEvent {
    ViewLayout { url: String },
}

impl EventEmitter<DetailsPanelEvent> for DetailsPanel {}

/// Where the page landed among the crawl's layout templates, for a crawl that
/// captured layout.
fn layout_section(
    rec: &PageRecord,
    order: &TextOrder,
    panel: &Entity<DetailsPanel>,
    muted: Hsla,
    border: Hsla,
) -> Option<AnyElement> {
    let layout = rec.layout.as_ref()?;
    let template = layout
        .template
        .as_deref()
        .map(crate::crawl::layout_groups::template_display_name)
        .unwrap_or_else(|| t!("details.not_grouped").to_string());
    let joined_by = layout
        .assignment
        .map(|assignment| assignment.label())
        .unwrap_or_default();
    let url = rec.url.clone();
    let panel = panel.clone();
    let body = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(
            order,
            t!("details.field.template"),
            SharedString::from(template),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.joined_by"),
            SharedString::from(joined_by),
            muted,
        ))
        .when(layout.misses_template_markup(), |body| {
            body.child(row(
                order,
                t!("details.field.markup"),
                SharedString::from(t!(
                    "details.missing_template_markup",
                    kind = layout.template_type.clone().unwrap_or_default()
                )),
                muted,
            ))
        })
        .child(
            div().pt_1().child(
                Button::new("view-layout")
                    .xsmall()
                    .outline()
                    .label(t!("details.view_layout"))
                    .on_click(move |_, _, cx| {
                        let url = url.clone();
                        panel.update(cx, |_, cx| {
                            cx.emit(DetailsPanelEvent::ViewLayout { url });
                        });
                    }),
            ),
        )
        .into_any_element();
    Some(section(
        &t!("details.section.layout"),
        Some(Icon::LayoutPanelLeft),
        None,
        muted,
        border,
        body,
    ))
}

impl Default for DetailsPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// A field's value, or nothing at all when it has none.
///
/// The same convention the grid uses. A row here carries its label, so an
/// empty value column says "this page has no canonical" without spending a
/// character on saying it.
fn or_blank<T: AsRef<str>>(value: &Option<T>) -> SharedString {
    match value.as_ref().map(AsRef::as_ref) {
        Some(text) if !text.is_empty() => SharedString::from(text.to_string()),
        _ => SharedString::default(),
    }
}

/// Gives each selectable text in the panel its place in reading order, so a
/// selection dragged across several values copies them top to bottom rather
/// than in whatever order the window's registry holds them. The position is
/// also the text's element id, which keeps ids unique within a render and the
/// same from one frame to the next, so each text keeps its selection state.
struct TextOrder {
    next: Cell<u64>,
}

impl TextOrder {
    fn starting_at(first: u64) -> Self {
        Self {
            next: Cell::new(first),
        }
    }

    fn text(&self, text: impl Into<SharedString>) -> SelectableText {
        let position = self.next.get();
        self.next.set(position + 1);
        SelectableText::new(("details-text", position as usize), text).document_order(position)
    }
}

/// Where the virtual lists' text starts in reading order. A list is the only
/// pane on screen when its tab is open, so it only has to come after the
/// header above it.
const LIST_TEXT_ORDER: u64 = 1 << 32;

/// What a row shows on the right: text, which can be selected and copied, or
/// an element such as a status tag.
enum RowValue {
    Text(SharedString),
    Element(AnyElement),
}

impl From<SharedString> for RowValue {
    fn from(text: SharedString) -> Self {
        Self::Text(text)
    }
}

impl From<AnyElement> for RowValue {
    fn from(element: AnyElement) -> Self {
        Self::Element(element)
    }
}

fn row(
    order: &TextOrder,
    label: impl Into<SharedString>,
    value: impl Into<RowValue>,
    muted: Hsla,
) -> AnyElement {
    // Only the value is selectable. Every selectable text costs each frame
    // more while a selection is active (gpui-base republishes the selection to
    // all of them as each registers), and the label is beside the value anyway.
    let label: SharedString = label.into();
    let value = match value.into() {
        RowValue::Text(text) => order.text(text).into_any_element(),
        RowValue::Element(element) => element,
    };
    div()
        .flex()
        .flex_grow(1.0)
        .justify_between()
        .gap_3()
        .text_xs()
        .child(div().flex_shrink_0().text_color(muted).child(label))
        .child(div().max_w_full().min_w(px(0.)).child(value))
        .into_any_element()
}

/// "Copy" for whatever text is selected in the window. The text is read when
/// the menu opens: by the time an item is clicked, the press that opened the
/// menu or the click itself may have cleared the selection.
fn copy_selection_item(window: &mut Window, cx: &mut App) -> PopupMenuItem {
    let selected = TextSelection::selected_text(window, cx).trim().to_string();
    PopupMenuItem::new(t!("details.menu.copy"))
        .icon(IconName::Copy)
        .disabled(selected.is_empty())
        .on_click(move |_, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(selected.clone()));
        })
}

fn copy_text_item(label: SharedString, text: SharedString) -> PopupMenuItem {
    PopupMenuItem::new(label)
        .icon(IconName::Copy)
        .on_click(move |_, _, cx| {
            cx.write_to_clipboard(ClipboardItem::new_string(text.to_string()));
        })
}

fn open_url_item(url: SharedString) -> PopupMenuItem {
    PopupMenuItem::new(t!("details.menu.open_in_browser"))
        .icon(Icon::ExternalLink)
        .on_click(move |_, _, cx| cx.open_url(&url))
}

/// The menu for one entry in a list: the selection, the entry's URL, and the
/// one piece of detail worth copying on its own, such as a link's anchor text.
fn link_menu(
    menu: PopupMenu,
    window: &mut Window,
    cx: &mut App,
    url: &SharedString,
    detail: Option<(SharedString, SharedString)>,
) -> PopupMenu {
    menu.item(copy_selection_item(window, cx))
        .separator()
        .item(copy_text_item(
            t!("details.menu.copy_url").into(),
            url.clone(),
        ))
        .when_some(detail, |menu, (label, text)| {
            menu.item(copy_text_item(label, text))
        })
        .item(open_url_item(url.clone()))
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

/// axe's impact levels are stored as axe names them, so they are matched here
/// to be shown in the app's language, and anything else is shown as stored.
fn a11y_impact_label(impact: &str) -> SharedString {
    match impact {
        "critical" => t!("details.impact.critical").into(),
        "serious" => t!("details.impact.serious").into(),
        "moderate" => t!("details.impact.moderate").into(),
        "minor" => t!("details.impact.minor").into(),
        _ => SharedString::from(impact.to_string()),
    }
}

fn a11y_impact_tone(impact: &str) -> Tone {
    match impact {
        "critical" | "serious" => Tone::Err,
        "moderate" => Tone::Warn,
        _ => Tone::Neutral,
    }
}

fn serp_preview(rec: &PageRecord, order: &TextOrder, cx: &App) -> AnyElement {
    let title = rec
        .title
        .as_deref()
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| t!("details.serp.no_title").to_string());
    let url_display = rec.url.clone();
    let desc = rec
        .meta_description
        .as_deref()
        .filter(|d| !d.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| t!("details.serp.no_meta_description").to_string());

    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_2()
        .py_1p5()
        .child(
            // The heading of a real result is set in Google Sans, and it is
            // the font the title pixel-width rules measure against, so the
            // preview shows the line the way the rule reads it.
            div()
                .font_family(crate::ui::fonts::FAMILY)
                .text_size(px(18.))
                .text_color(cx.theme().blue)
                .child(order.text(title)),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().green)
                .child(order.text(url_display)),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(order.text(desc)),
        )
        .into_any_element()
}

#[allow(clippy::too_many_arguments)]
fn a11y_issue_row(
    issue: &A11yIssue,
    order: &TextOrder,
    html_view: Option<&HtmlView>,
    index: usize,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    let impact_tag =
        tone_tag(a11y_impact_tone(&issue.impact), cx).child(a11y_impact_label(&issue.impact));
    let rule_name = SharedString::from(issue.rule.clone());
    let rule_el: AnyElement = match rule_description(&issue.rule) {
        Some(desc) => {
            let desc = SharedString::from(desc);
            div()
                .id(("a11y-rule-tip", index))
                .text_color(fg)
                .text_sm()
                .child(order.text(rule_name))
                .tooltip(move |window, cx| Tooltip::new(desc.clone()).build(window, cx))
                .into_any_element()
        }
        None => div()
            .text_color(fg)
            .text_sm()
            .child(order.text(rule_name))
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
            el.child(div().text_xs().text_color(muted).child(order.text(target)))
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
    label: &'static str,
    order: &TextOrder,
    value: SharedString,
    tone: Option<Tone>,
    muted: Hsla,
    border: Hsla,
    panel2: Hsla,
) -> AnyElement {
    let value_color = match tone {
        Some(Tone::Ok) => Some(gpui_kit::hsla(142. / 360., 0.71, 0.45, 1.0)),
        Some(Tone::Warn) => Some(gpui_kit::hsla(38. / 360., 0.92, 0.50, 1.0)),
        Some(Tone::Err) => Some(gpui_kit::hsla(0. / 360., 0.84, 0.60, 1.0)),
        _ => None,
    };
    let label = order.text(label);
    let mut value_div = div()
        .text_sm()
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(order.text(value));
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
        .child(div().text_xs().text_color(muted).child(label))
        .child(value_div)
        .into_any_element()
}

fn header_block(rec: &PageRecord, order: &TextOrder, muted: Hsla, border: Hsla) -> AnyElement {
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
                                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                .child(SharedString::from(t!("details.url_details"))),
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
                .child(order.text(rec.url.clone())),
        )
        .into_any_element()
}

fn url_information_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    let status_tag = match rec.status {
        Some(c) => tone_tag(status_code_tone(c), cx)
            .child(SharedString::from(c.to_string()))
            .into_any_element(),
        None => div().into_any_element(),
    };
    let indexability_value = rec.indexability.clone().unwrap_or_default();
    let indexability_tag = if indexability_value.is_empty() {
        div().into_any_element()
    } else {
        tone_tag(indexability_tone(&indexability_value), cx)
            .child(SharedString::from(
                crate::crawl::event::indexability_status_text(&indexability_value),
            ))
            .into_any_element()
    };
    let url_info = div()
        .flex()
        .flex_col()
        .w_full()
        .gap_1()
        .child(row(
            order,
            t!("details.field.address"),
            SharedString::from(rec.url.clone()),
            muted,
        ))
        .child(row(order, t!("details.field.status"), status_tag, muted))
        .child(row(
            order,
            t!("details.field.content_type"),
            or_blank(&rec.content_type),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.size"),
            SharedString::from(crate::ui::tag::format_bytes(rec.size_bytes)),
            muted,
        ))
        .when_some(rec.transfer_bytes, |el, bytes| {
            el.child(row(
                order,
                t!("details.field.transferred"),
                SharedString::from(crate::ui::tag::format_bytes(bytes)),
                muted,
            ))
        })
        .child(row(
            order,
            t!("details.field.indexability"),
            indexability_tag,
            muted,
        ))
        .child(row(
            order,
            t!("details.field.canonical"),
            or_blank(&rec.canonical),
            muted,
        ))
        .when_some(rec.redirect_url.clone(), |el, redirect| {
            el.child(row(
                order,
                t!("details.field.redirect_uri"),
                SharedString::from(redirect),
                muted,
            ))
            .child(row(
                order,
                t!("details.field.redirect_status"),
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
                    order,
                    t!("details.field.last_modified"),
                    SharedString::from(
                        rec.headers
                            .iter()
                            .find(|(k, _)| k.eq_ignore_ascii_case("last-modified"))
                            .map(|(_, v)| v.as_str())
                            .unwrap_or_default(),
                    ),
                    muted,
                ))
            },
        )
        .into_any_element();

    section(
        &t!("details.section.url_information"),
        None,
        None,
        muted,
        border,
        url_info,
    )
}

fn page_content_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
) -> AnyElement {
    let page_content = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(
            order,
            t!("details.field.title"),
            or_blank(&rec.title),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.title_length"),
            SharedString::from(
                rec.title
                    .as_ref()
                    .map(|t| t.len().to_string())
                    .unwrap_or_else(|| "0".into()),
            ),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.title_pixel_width"),
            SharedString::from(
                rec.title_pixel_width
                    .map(|w| w.to_string())
                    .unwrap_or_default(),
            ),
            muted,
        ))
        .when_some(rec.title_2.clone(), |el, t2| {
            el.child(row(
                order,
                t!("details.field.title_2"),
                SharedString::from(t2),
                muted,
            ))
        })
        .child(row(
            order,
            t!("details.field.meta_description"),
            or_blank(&rec.meta_description),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.meta_length"),
            SharedString::from(
                rec.meta_description
                    .as_ref()
                    .map(|d| d.len().to_string())
                    .unwrap_or_else(|| "0".into()),
            ),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.meta_pixel_width"),
            SharedString::from(
                rec.meta_description_pixel_width
                    .map(|w| w.to_string())
                    .unwrap_or_default(),
            ),
            muted,
        ))
        .when_some(rec.meta_description_2.clone(), |el, md2| {
            el.child(row(
                order,
                t!("details.field.meta_description_2"),
                SharedString::from(md2),
                muted,
            ))
        })
        .child(row(order, "H1", or_blank(&rec.h1), muted))
        .child(row(
            order,
            t!("details.field.h1_length"),
            SharedString::from(
                rec.h1
                    .as_ref()
                    .map(|h| h.len().to_string())
                    .unwrap_or_else(|| "0".into()),
            ),
            muted,
        ))
        .when_some(rec.h1_2.clone(), |el, h1_2| {
            el.child(row(order, "H1-2", SharedString::from(h1_2), muted))
        })
        .child(row(order, "H2", or_blank(&rec.h2), muted))
        .child(row(
            order,
            t!("details.field.word_count"),
            SharedString::from(rec.word_count.map(|w| w.to_string()).unwrap_or_default()),
            muted,
        ))
        .when(rec.ssr_word_count.is_some(), |el| {
            el.child(row(
                order,
                t!("details.field.ssr_words"),
                SharedString::from(
                    rec.ssr_word_count
                        .map(|w| w.to_string())
                        .unwrap_or_default(),
                ),
                muted,
            ))
            .child(row(
                order,
                t!("details.field.ssr_csr_diff"),
                SharedString::from(ssr_diff_label(rec)),
                muted,
            ))
        })
        .child(row(
            order,
            t!("details.field.meta_robots"),
            or_blank(&rec.robots),
            muted,
        ))
        .into_any_element();

    section(
        &t!("details.section.page_content"),
        None,
        None,
        muted,
        border,
        page_content,
    )
}

fn structured_data_section(
    rec: &PageRecord,
    order: &TextOrder,
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
                .child(SharedString::from(if rec.sd_items.len() == 1 {
                    t!("details.summary.types_one", count = rec.sd_items.len())
                } else {
                    t!("details.summary.types_other", count = rec.sd_items.len())
                }))
                .into_any_element(),
        )
    };
    let sd_types_text = SharedString::from(rec.sd_types.join(", "));

    let mut sd_items_body = div().flex().flex_col().gap_1();
    for (item, json_view) in rec.sd_items.iter().zip(json_views.iter()) {
        let format_label = match item.format {
            SdFormat::JsonLd => "JSON-LD".into(),
            SdFormat::Microdata => t!("details.field.microdata"),
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
                            .child(order.text(item.type_name.clone())),
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
                                .child(order.text(issue.type_name.clone())),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(order.text(issue.translated_message())),
                ),
        );
    }

    let sd_body = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(order, t!("details.field.types"), sd_types_text, muted))
        .child(row(
            order,
            "JSON-LD",
            SharedString::from(rec.sd_jsonld_count.to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.microdata"),
            SharedString::from(rec.sd_microdata_count.to_string()),
            muted,
        ))
        .when(rec.og_type.is_some(), |el| {
            el.child(row(
                order,
                "Open Graph",
                SharedString::from(rec.og_type.clone().unwrap_or_default()),
                muted,
            ))
        })
        .child(row(
            order,
            t!("details.field.errors"),
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
            order,
            t!("details.field.warnings"),
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
        &t!("details.section.structured_data"),
        Some(Icon::Braces),
        sd_summary,
        muted,
        border,
        sd_body,
    )
}

fn accessibility_section(
    rec: &PageRecord,
    order: &TextOrder,
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
                .child(SharedString::from(if total_a11y == 1 {
                    t!("details.summary.issues_one", count = total_a11y)
                } else {
                    t!("details.summary.issues_other", count = total_a11y)
                }))
                .into_any_element(),
        )
    };
    let mut a11y_body = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(
            order,
            t!("details.field.errors"),
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
            order,
            t!("details.field.warnings"),
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
            order,
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
        &t!("details.section.accessibility"),
        Some(Icon::Accessibility),
        a11y_summary,
        muted,
        border,
        a11y_body,
    )
}

fn vitals_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
    panel2: Hsla,
) -> AnyElement {
    let lcp_str = rec
        .lcp_ms
        .map(|ms| format!("{:.1}s", ms as f32 / 1000.0))
        .unwrap_or_default();
    let cls_str = rec.cls.map(|v| format!("{:.3}", v)).unwrap_or_default();
    let fcp_str = rec.fcp_ms.map(|ms| format!("{ms}ms")).unwrap_or_default();
    let ttfb_str = rec.ttfb_ms.map(|ms| format!("{ms}ms")).unwrap_or_default();

    let tiles = [
        ("LCP", lcp_str, rec.lcp_ms.map(lcp_tone)),
        ("CLS", cls_str, rec.cls.map(cls_tone)),
        ("FCP", fcp_str, rec.fcp_ms.map(fcp_tone)),
        ("TTFB", ttfb_str, rec.ttfb_ms.map(ttfb_tone)),
    ];
    let vitals_body = div()
        .flex()
        .gap_2()
        .children(tiles.into_iter().map(|(label, value, tone)| {
            div().flex_1().min_w_0().child(vital_tile(
                label,
                order,
                SharedString::from(value),
                tone,
                muted,
                border,
                panel2,
            ))
        }))
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
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
) -> AnyElement {
    let link_score_str = rec
        .link_score
        .map(|s| format!("{s:.1}"))
        .unwrap_or_default();
    let links_body = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(
            order,
            t!("details.field.inlinks"),
            SharedString::from(rec.inlinks_count.to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.csr_inlinks"),
            SharedString::from(rec.csr_inlinks_count.to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.csr_in_percent"),
            SharedString::from(if rec.inlinks_count > 0 && rec.csr_inlinks_count > 0 {
                format!(
                    "{}%",
                    (rec.csr_inlinks_count as f64 / rec.inlinks_count as f64 * 100.0).round()
                        as u32
                )
            } else {
                String::new()
            }),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.outlinks"),
            SharedString::from(rec.outlinks.len().to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.csr_outlinks"),
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
            order,
            t!("details.field.csr_out_percent"),
            SharedString::from({
                let total = rec.outlinks.len();
                let csr_out = rec.outlinks.iter().filter(|o| o.csr_only).count();
                if total > 0 && csr_out > 0 {
                    format!(
                        "{}%",
                        (csr_out as f64 / total as f64 * 100.0).round() as u32
                    )
                } else {
                    String::new()
                }
            }),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.link_score"),
            SharedString::from(link_score_str),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.backlinks"),
            SharedString::from(backlink_count(rec).to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.depth"),
            SharedString::from(rec.depth.map(|d| d.to_string()).unwrap_or_default()),
            muted,
        ))
        .child(row(
            order,
            "Hreflang",
            SharedString::from(rec.hreflang_tags.len().to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.near_duplicates"),
            SharedString::from(
                rec.near_duplicate_count
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            ),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.closest_similarity"),
            SharedString::from(
                rec.closest_similarity
                    .map(|s| format!("{s}%"))
                    .unwrap_or_default(),
            ),
            muted,
        ))
        .into_any_element();

    section(
        &t!("details.section.link_metrics"),
        None,
        None,
        muted,
        border,
        links_body,
    )
}

fn images_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
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
                    .child(SharedString::from(t!(
                        "details.summary.missing_alt",
                        count = missing_alt
                    )))
                    .into_any_element(),
            )
        } else {
            Some(
                tone_tag(Tone::Ok, cx)
                    .child(SharedString::from(if rec.images.len() == 1 {
                        t!("details.summary.images_one", count = rec.images.len())
                    } else {
                        t!("details.summary.images_other", count = rec.images.len())
                    }))
                    .into_any_element(),
            )
        }
    };

    let mut images_body = div().flex().flex_col().gap_1();
    if rec.images.is_empty() {
        images_body = images_body.child(
            div()
                .text_color(muted)
                .text_sm()
                .child(SharedString::from(t!("details.images.none_found"))),
        );
    } else {
        for image in &rec.images {
            let alt_tag = if image.has_alt_attr {
                if image.alt.as_deref().is_none_or(|a| a.is_empty()) {
                    tone_tag(Tone::Warn, cx)
                        .child(SharedString::from(t!("details.images.alt_empty")))
                } else {
                    tone_tag(Tone::Ok, cx).child(SharedString::from(t!("details.images.alt_yes")))
                }
            } else {
                tone_tag(Tone::Err, cx).child(SharedString::from(t!("details.images.alt_missing")))
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
                            .child(order.text(image.src.clone())),
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
                                    .child(
                                        div().text_color(muted).child(SharedString::from(t!(
                                            "details.images.alt_label"
                                        ))),
                                    )
                                    .child(alt_tag)
                                    .when_some(image.alt.clone(), |el, alt| {
                                        el.child(div().text_color(fg).child(order.text(alt)))
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
                                        el.child(div().text_color(muted).child(SharedString::from(
                                            t!("details.images.no_dimensions"),
                                        )))
                                    }),
                            ),
                    ),
            );
        }
    }
    let images_section_body = images_body.into_any_element();

    section(
        &t!("details.section.images"),
        Some(Icon::Image),
        images_summary,
        muted,
        border,
        images_section_body,
    )
}

fn serp_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    section(
        &t!("details.section.serp_preview"),
        None,
        None,
        muted,
        border,
        serp_preview(rec, order, cx),
    )
}

fn hreflang_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> Option<AnyElement> {
    if rec.hreflang_issues.is_empty() {
        return None;
    }
    let mut body = div().flex().flex_col().gap_1();
    for issue in &rec.hreflang_issues {
        // Badged like the accessibility issues above, and graded like the
        // Hreflang tab: all of these are warnings there, and a panel that
        // paints them in error red says something the tab contradicts. Only an
        // invalid language code is unambiguously wrong.
        let (tone, name, detail) = match issue {
            crate::crawl::event::HreflangIssue::MissingReturnTag { lang, target_url } => (
                Tone::Warn,
                t!("details.hreflang.missing_return_tag"),
                Some(format!("{lang} -> {target_url}")),
            ),
            crate::crawl::event::HreflangIssue::InvalidLanguageCode { code } => (
                Tone::Err,
                t!("details.hreflang.invalid_language_code"),
                Some(code.clone()),
            ),
            crate::crawl::event::HreflangIssue::MissingXDefault => {
                (Tone::Warn, t!("details.hreflang.missing_x_default"), None)
            }
            crate::crawl::event::HreflangIssue::MissingSelfReference => (
                Tone::Warn,
                t!("details.hreflang.missing_self_reference"),
                None,
            ),
            crate::crawl::event::HreflangIssue::NonCanonicalUrl { hreflang_url } => (
                Tone::Warn,
                t!("details.hreflang.non_canonical_target"),
                Some(hreflang_url.clone()),
            ),
        };
        body = body.child(
            div()
                .flex()
                .flex_col()
                .gap_0p5()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(tone_tag(tone, cx).child(SharedString::from(match tone {
                            Tone::Err => t!("details.hreflang.error"),
                            _ => t!("details.hreflang.warning"),
                        })))
                        .child(div().text_xs().text_color(fg).child(order.text(name))),
                )
                .when_some(detail, |el, detail| {
                    el.child(div().text_xs().text_color(muted).child(order.text(detail)))
                }),
        );
    }
    Some(section(
        &t!("details.section.hreflang_issues"),
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

/// The height of one row in the link and image-reference lists. The lists are
/// virtualised, which needs every row to be the same known height, so a row is
/// two fixed lines: the URL and one line of detail under it, with enough room
/// above and below that consecutive rows read as separate entries rather than
/// one block of text.
const LINK_ROW_HEIGHT: Pixels = px(46.);

/// One entry in a list: its URL and a line of detail, both selectable, with a
/// menu for copying or opening it. `copyable_detail` offers the part of the
/// detail worth copying on its own, such as a link's anchor text, under the
/// given menu label.
///
/// The row is identified by its list and index. The menu keys its open state
/// on that id, and without one every row would share a single menu.
#[allow(clippy::too_many_arguments)]
fn link_row(
    list: &'static str,
    row_index: usize,
    url: SharedString,
    detail: SharedString,
    copyable_detail: Option<(SharedString, SharedString)>,
    leading: Option<AnyElement>,
    trailing: Option<AnyElement>,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> AnyElement {
    let order = TextOrder::starting_at(LIST_TEXT_ORDER + row_index as u64 * 2);
    let menu_url = url.clone();
    div()
        .id((list, row_index))
        .h(LINK_ROW_HEIGHT)
        .flex()
        .flex_col()
        .justify_center()
        .gap_1()
        .py_1()
        .border_t_1()
        .border_color(border)
        .child(
            div()
                .flex()
                .items_center()
                .gap_1()
                .when_some(leading, |el, tag| el.child(tag))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .text_xs()
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_color(fg)
                        .truncate()
                        .child(order.text(url)),
                )
                .when_some(trailing, |el, tag| el.child(tag)),
        )
        .child(
            div()
                .text_xs()
                .text_color(muted)
                .truncate()
                .child(order.text(detail)),
        )
        .context_menu(move |menu, window, cx| {
            link_menu(menu, window, cx, &menu_url, copyable_detail.clone())
        })
        .into_any_element()
}

fn link_row_sizes(row_count: usize) -> Rc<Vec<Size<Pixels>>> {
    Rc::new(vec![
        Size {
            width: px(0.),
            height: LINK_ROW_HEIGHT,
        };
        row_count
    ])
}

/// One duplicate URL is a single line, so its rows are shorter than a link's.
const DUPLICATE_ROW_HEIGHT: Pixels = px(18.);

fn duplicate_row_sizes(row_count: usize) -> Rc<Vec<Size<Pixels>>> {
    Rc::new(vec![
        Size {
            width: px(0.),
            height: DUPLICATE_ROW_HEIGHT,
        };
        row_count
    ])
}

/// The page a link or reference section is describing, or `None` when the
/// selection is not a page. Used inside the virtual lists' render closures,
/// which run against the panel rather than a captured record.
/// How many links point at the page. Counted with the crawl, so the tab and
/// the figure are right before the list itself has been read.
fn backlink_count(rec: &PageRecord) -> usize {
    (rec.inlinks_count as usize).max(rec.backlinks.len())
}

fn selected_page(panel: &DetailsPanel) -> Option<&PageRecord> {
    match &panel.selected {
        Some(DetailsSelection::Page { record, .. }) => Some(record),
        _ => None,
    }
}

fn headers_section(
    rec: &PageRecord,
    order: &TextOrder,
    muted: Hsla,
    fg: Hsla,
    border: Hsla,
    cx: &App,
) -> Option<AnyElement> {
    if rec.headers.is_empty() {
        return None;
    }
    let mut headers_body = div().flex().flex_col().gap_0p5();
    for (index, (key, value)) in rec.headers.iter().enumerate() {
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
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .text_color(muted)
                        .child(order.text(key.clone())),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_color(fg)
                        .overflow_x_scrollbar()
                        // One scroll position per header, rather than one for
                        // every value this line makes.
                        .id(("header-value", index))
                        .child(order.text(display_value)),
                ),
        );
    }
    Some(section(
        &t!("details.section.http_headers"),
        None,
        Some(
            div()
                .text_xs()
                .child(SharedString::from(if rec.headers.len() == 1 {
                    t!("details.summary.headers_one", count = rec.headers.len())
                } else {
                    t!("details.summary.headers_other", count = rec.headers.len())
                }))
                .into_any_element(),
        ),
        muted,
        border,
        headers_body.into_any_element(),
    ))
}

fn image_header_block(
    image: &ImageDetails,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
) -> AnyElement {
    let src = image.src.clone();
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
                        .child(UiIcon::from(Icon::Image).small())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                .child(SharedString::from(t!("details.image_details"))),
                        ),
                )
                .child(
                    div()
                        .id("open-image-details")
                        .cursor_pointer()
                        .text_color(muted)
                        .child(UiIcon::from(Icon::ExternalLink).small())
                        .on_click(move |_, _window, cx| {
                            cx.open_url(&src);
                        }),
                ),
        )
        .child(
            div()
                .mt_1()
                .text_xs()
                .text_color(muted)
                .child(order.text(image.src.clone())),
        )
        .into_any_element()
}

fn image_information_section(
    image: &ImageDetails,
    order: &TextOrder,
    muted: Hsla,
    border: Hsla,
) -> AnyElement {
    let dimensions = match (image.width, image.height) {
        (Some(w), Some(h)) => format!("{w} x {h}"),
        // One side declared is worth showing, and the missing half is what the
        // Missing Size Attributes rule is about.
        (Some(w), None) => format!("{w} x ?"),
        (None, Some(h)) => format!("? x {h}"),
        (None, None) => String::new(),
    };
    let missing_alt = image
        .references
        .iter()
        .filter(|reference| {
            !reference.has_alt_attr || reference.alt.as_deref().is_none_or(|alt| alt.is_empty())
        })
        .count();
    let body = div()
        .flex()
        .flex_col()
        .gap_1()
        .child(row(
            order,
            t!("details.field.references"),
            SharedString::from(image.references.len().to_string()),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.dimensions"),
            SharedString::from(dimensions),
            muted,
        ))
        .child(row(
            order,
            t!("details.field.missing_alt"),
            SharedString::from(missing_alt.to_string()),
            muted,
        ))
        .into_any_element();
    section(
        &t!("details.section.image_information"),
        None,
        None,
        muted,
        border,
        body,
    )
}

/// The tabs a selection has, in order, with the number of rows behind each.
/// A tab with nothing in it is not offered.
fn tabs_for(selection: &DetailsSelection, panel: &DetailsPanel) -> Vec<(DetailsTab, usize)> {
    let mut tabs = vec![(DetailsTab::Overview, 0)];
    match selection {
        DetailsSelection::Page {
            record: rec,
            references,
            ..
        } => {
            for (tab, count) in [
                (DetailsTab::NearDuplicates, rec.near_duplicate_urls.len()),
                (DetailsTab::Images, rec.images.len()),
                (DetailsTab::Inlinks, backlink_count(rec)),
                (DetailsTab::References, references.len()),
                (DetailsTab::Outlinks, rec.outlinks.len()),
            ] {
                if count > 0 {
                    tabs.push((tab, count));
                }
            }
        }
        DetailsSelection::Image(image) => {
            if !image.references.is_empty() {
                tabs.push((DetailsTab::References, image.references.len()));
            }
        }
    }
    let _ = panel;
    tabs
}

/// The strip of tabs under the header. Styled like the grid's filter segments,
/// so the two rows of tabs in the window read as the same control.
fn tab_strip(
    tabs: &[(DetailsTab, usize)],
    active: DetailsTab,
    panel: &Entity<DetailsPanel>,
    cx: &App,
) -> AnyElement {
    let mut strip = div()
        .flex()
        .flex_row()
        .gap(px(2.))
        .px_2()
        .py_1p5()
        .border_b_1()
        .border_color(cx.theme().border);

    for &(tab, count) in tabs {
        let is_active = tab == active;
        let segment = div()
            .id(SharedString::from(format!("details-tab-{:?}", tab)))
            .flex()
            .items_center()
            .gap_1()
            .px_1p5()
            .py_0p5()
            .rounded(cx.theme().radius)
            .cursor_pointer()
            .text_xs()
            .when(is_active, |el| {
                el.bg(cx.theme().tab_bar_segmented)
                    .text_color(cx.theme().tab_active_foreground)
            })
            .when(!is_active, |el| {
                el.text_color(cx.theme().tab_foreground)
                    .hover(|el| el.text_color(cx.theme().tab_active_foreground))
            })
            .child(tab.label())
            .when(count > 0, |el| {
                // The same chip the grid's tabs and filter segments carry their
                // counts in, so a count reads the same wherever it appears.
                el.child(
                    tone_tag(Tone::Neutral, cx)
                        .rounded_full()
                        .child(SharedString::from(count.to_string())),
                )
            })
            .on_click({
                let panel = panel.clone();
                move |_, _window, cx| {
                    panel.update(cx, |this, cx| {
                        this.active_tab = tab;
                        cx.notify();
                    });
                }
            });
        strip = strip.child(segment);
    }
    strip.into_any_element()
}

/// A list that fills the pane and scrolls on its own. The only scrollable
/// region in the panel while its tab is open.
fn list_pane(
    id: &'static str,
    panel: &Entity<DetailsPanel>,
    scroll: &VirtualListScrollHandle,
    sizes: Rc<Vec<Size<Pixels>>>,
    render: impl 'static + Fn(&mut DetailsPanel, std::ops::Range<usize>, &mut App) -> Vec<AnyElement>,
) -> AnyElement {
    div()
        .flex_1()
        .min_h(px(0.))
        .px_3()
        .child(
            v_virtual_list(panel.clone(), id, sizes, move |this, range, _window, cx| {
                render(this, range, cx)
            })
            .track_scroll(scroll),
        )
        .into_any_element()
}

impl Render for DetailsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The virtualised lists render against the panel itself rather than a
        // captured copy of the records, so they need a handle to it.
        let panel = cx.entity();
        let theme = cx.theme();
        let muted = theme.muted_foreground;
        let fg = theme.foreground;
        let border = theme.border;
        let panel2 = theme.secondary;

        let Some(selection) = &self.selected else {
            return div()
                .w(px(420.))
                .h_full()
                .border_l_1()
                .border_color(border)
                .rounded_br(crate::app::PANEL_RADIUS)
                .child(
                    div()
                        .p_4()
                        .text_sm()
                        .text_color(muted)
                        .child(SharedString::from(t!("details.empty"))),
                )
                .into_any_element();
        };

        let order = &TextOrder::starting_at(0);
        let tabs = tabs_for(selection, self);
        // The selection can change under an open tab, and a page with no
        // inlinks does not offer one.
        let active = if tabs.iter().any(|(tab, _)| *tab == self.active_tab) {
            self.active_tab
        } else {
            DetailsTab::Overview
        };

        let selected_url = SharedString::from(match &selection {
            DetailsSelection::Page { record: rec, .. } => rec.url.clone(),
            DetailsSelection::Image(image) => image.src.clone(),
        });
        let header = match &selection {
            DetailsSelection::Page { record: rec, .. } => header_block(rec, order, muted, border),
            DetailsSelection::Image(image) => image_header_block(image, order, muted, border),
        };

        // Every pane either scrolls itself or is a virtual list that does; the
        // panel around them never scrolls, so there is only ever one scrollable
        // region under the pointer.
        let pane = match (&selection, active) {
            (DetailsSelection::Page { record: rec, .. }, DetailsTab::NearDuplicates) => {
                let mono = cx.theme().mono_font_family.clone();
                list_pane(
                    "near-duplicates",
                    &panel,
                    &self.duplicates_scroll,
                    duplicate_row_sizes(rec.near_duplicate_urls.len()),
                    move |this, range, _cx| {
                        let Some(rec) = selected_page(this) else {
                            return Vec::new();
                        };
                        let first = range.start;
                        rec.near_duplicate_urls
                            .get(range)
                            .unwrap_or_default()
                            .iter()
                            .enumerate()
                            .map(|(offset, dup_url)| {
                                let row_index = first + offset;
                                let url = SharedString::from(dup_url.clone());
                                let order =
                                    TextOrder::starting_at(LIST_TEXT_ORDER + row_index as u64);
                                div()
                                    .id(("near-duplicates", row_index))
                                    .h(DUPLICATE_ROW_HEIGHT)
                                    .flex()
                                    .items_center()
                                    .text_xs()
                                    .font_family(mono.clone())
                                    .text_color(fg)
                                    .truncate()
                                    .child(order.text(url.clone()))
                                    .context_menu(move |menu, window, cx| {
                                        link_menu(menu, window, cx, &url, None)
                                    })
                                    .into_any_element()
                            })
                            .collect()
                    },
                )
            }
            (DetailsSelection::Page { record: rec, .. }, DetailsTab::Images) => div()
                .id("details-images")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(images_section(rec, order, muted, fg, border, cx))
                .into_any_element(),
            (DetailsSelection::Page { record: rec, .. }, DetailsTab::Inlinks) => list_pane(
                "inlinks",
                &panel,
                &self.inlinks_scroll,
                link_row_sizes(rec.backlinks.len()),
                move |this, range, cx| {
                    let Some(rec) = selected_page(this) else {
                        return Vec::new();
                    };
                    let first = range.start;
                    rec.backlinks
                        .get(range)
                        .unwrap_or_default()
                        .iter()
                        .enumerate()
                        .map(|(offset, backlink)| {
                            let anchor =
                                backlink.anchor.as_deref().filter(|a| !a.trim().is_empty());
                            let shown_anchor = anchor
                                .map(str::to_string)
                                .unwrap_or_else(|| t!("details.links.no_anchor").to_string());
                            let detail = match backlink.rel.as_deref() {
                                Some(rel) => format!("{shown_anchor} · rel={rel}"),
                                None => shown_anchor,
                            };
                            link_row(
                                "inlinks",
                                first + offset,
                                SharedString::from(backlink.source_url.clone()),
                                SharedString::from(detail),
                                anchor.map(|anchor| {
                                    (
                                        t!("details.menu.copy_anchor_text").into(),
                                        SharedString::from(anchor.to_string()),
                                    )
                                }),
                                None,
                                None,
                                muted,
                                fg,
                                border,
                                cx,
                            )
                        })
                        .collect()
                },
            ),
            (DetailsSelection::Page { record: rec, .. }, DetailsTab::Outlinks) => list_pane(
                "outlinks",
                &panel,
                &self.outlinks_scroll,
                link_row_sizes(rec.outlinks.len()),
                move |this, range, cx| {
                    let Some(DetailsSelection::Page {
                        record: rec,
                        outlink_order,
                        ..
                    }) = &this.selected
                    else {
                        return Vec::new();
                    };
                    let first = range.start;
                    outlink_order
                        .get(range)
                        .unwrap_or_default()
                        .iter()
                        .filter_map(|&(index, status)| Some((rec.outlinks.get(index)?, status)))
                        .enumerate()
                        .map(|(offset, (link, status))| {
                            let is_nofollow = link
                                .rel
                                .as_deref()
                                .is_some_and(|r| r.to_ascii_lowercase().contains("nofollow"));
                            // Only what needs fixing is tagged: a 2xx on every
                            // row would bury the one broken link among them.
                            // In front of the URL, which a long one would
                            // otherwise push out of sight.
                            let leading = status.filter(|status| *status >= 300).map(|status| {
                                let tone = if status >= 400 { Tone::Err } else { Tone::Warn };
                                tone_tag(tone, cx)
                                    .child(SharedString::from(status.to_string()))
                                    .into_any_element()
                            });
                            let trailing = is_nofollow.then(|| {
                                tone_tag(Tone::Warn, cx)
                                    .child(SharedString::from("nofollow"))
                                    .into_any_element()
                            });
                            let anchor = link.anchor.as_deref().filter(|a| !a.trim().is_empty());
                            link_row(
                                "outlinks",
                                first + offset,
                                SharedString::from(link.dst_url.clone()),
                                SharedString::from(anchor.map(str::to_string).unwrap_or_else(
                                    || t!("details.links.no_anchor_text").to_string(),
                                )),
                                anchor.map(|anchor| {
                                    (
                                        t!("details.menu.copy_anchor_text").into(),
                                        SharedString::from(anchor.to_string()),
                                    )
                                }),
                                leading,
                                trailing,
                                muted,
                                fg,
                                border,
                                cx,
                            )
                        })
                        .collect()
                },
            ),
            (selection, DetailsTab::References) => list_pane(
                "image-references",
                &panel,
                &self.image_references_scroll,
                link_row_sizes(selection.references().len()),
                move |this, range, cx| {
                    let Some(selection) = &this.selected else {
                        return Vec::new();
                    };
                    let first = range.start;
                    selection
                        .references()
                        .get(range)
                        .unwrap_or_default()
                        .iter()
                        .enumerate()
                        .map(|(offset, reference)| {
                            // The tag says what is missing, not merely that
                            // something is: this row is about the alt text this
                            // page gives the image, and nothing else.
                            let (tone, label) = if !reference.has_alt_attr {
                                (Tone::Err, t!("details.references.missing_alt"))
                            } else if reference.alt.as_deref().is_none_or(|alt| alt.is_empty()) {
                                (Tone::Warn, t!("details.references.empty_alt"))
                            } else {
                                (Tone::Ok, t!("details.references.alt"))
                            };
                            let trailing = tone_tag(tone, cx)
                                .child(SharedString::from(label))
                                .into_any_element();
                            let alt = SharedString::from(reference.alt.clone().unwrap_or_default());
                            link_row(
                                "image-references",
                                first + offset,
                                SharedString::from(reference.page_url.clone()),
                                alt.clone(),
                                (!alt.is_empty())
                                    .then(|| (t!("details.menu.copy_alt_text").into(), alt)),
                                None,
                                Some(trailing),
                                muted,
                                fg,
                                border,
                                cx,
                            )
                        })
                        .collect()
                },
            ),
            // Every page tab that is not a list. Also the fallback, so a
            // tab that does not apply to this selection lands somewhere real.
            (DetailsSelection::Page { record: rec, .. }, _) => div()
                .id("details-overview")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .flex()
                .flex_col()
                .child(url_information_section(rec, order, muted, border, cx))
                .when_some(
                    layout_section(rec, order, &panel, muted, border),
                    |el, s| el.child(s),
                )
                // An asset, a redirect or another file has no document to
                // describe, so these would be headings over empty values.
                .when(rec.is_page_document(), |el| {
                    el.child(page_content_section(rec, order, muted, border))
                        .child(serp_section(rec, order, muted, border, cx))
                        .child(structured_data_section(
                            rec,
                            order,
                            &self.json_views,
                            muted,
                            fg,
                            border,
                            cx,
                        ))
                        // Both of these are measured in a browser. On an HTTP crawl
                        // they would be a heading over nothing.
                        .when(self.render_mode.renders_javascript(), |el| {
                            el.child(accessibility_section(
                                rec,
                                order,
                                &self.html_views,
                                muted,
                                fg,
                                border,
                                cx,
                            ))
                            .child(vitals_section(rec, order, muted, border, panel2))
                        })
                        .child(link_metrics_section(rec, order, muted, border))
                })
                .when_some(
                    hreflang_section(rec, order, muted, fg, border, cx),
                    |el, s| el.child(s),
                )
                .when_some(
                    headers_section(rec, order, muted, fg, border, cx),
                    |el, s| el.child(s),
                )
                .into_any_element(),
            (DetailsSelection::Image(image), _) => div()
                .id("details-image-overview")
                .flex_1()
                .min_h(px(0.))
                .overflow_y_scrollbar()
                .child(image_information_section(image, order, muted, border))
                .into_any_element(),
        };

        div()
            .w(px(420.))
            .h_full()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(border)
            .rounded_br(crate::app::PANEL_RADIUS)
            .child(header)
            .when(tabs.len() > 1, |el| {
                el.child(tab_strip(&tabs, active, &panel, cx))
            })
            .child(pane)
            .context_menu(move |menu, window, cx| link_menu(menu, window, cx, &selected_url, None))
            .into_any_element()
    }
}

#[cfg(test)]
mod frame_bench;
