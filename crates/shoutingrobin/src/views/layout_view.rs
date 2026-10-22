//! One page's layout as a wireframe, next to its baseline when the earlier
//! crawl captured it, with what changed between the two. Shown in a dialog
//! from the details panel.

use std::collections::HashMap;

use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    App, BorderStyle, Bounds, Context, Hsla, InteractiveElement, IntoElement, ParentElement,
    Render, ScrollHandle, SharedString, StatefulInteractiveElement, Styled, Window, canvas, div,
    point, px, quad, size,
};

use crate::crawl::layout::{
    LayoutChange, LayoutChangeKind, LayoutTables, PageComparison, PageLayout, Rect, paints,
};
use crate::ui::tag::{Tone, tone_tag};

/// Width each wireframe is drawn at. The page is scaled to fit.
const COLUMN_WIDTH: f32 = 375.;
/// Pages taller than this are cut off; nothing below it is drawn.
const MAX_PAGE_HEIGHT: f32 = 20_000.;
const WIREFRAME_HEIGHT: f32 = 600.;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Highlight {
    Added,
    Removed,
    Geometry,
    Style,
    Content,
}

impl Highlight {
    fn of(change: &LayoutChange) -> Self {
        let has = |kind| change.kinds.contains(&kind);
        if has(LayoutChangeKind::Added) {
            Self::Added
        } else if has(LayoutChangeKind::Removed) {
            Self::Removed
        } else if has(LayoutChangeKind::Moved)
            || has(LayoutChangeKind::Resized)
            || has(LayoutChangeKind::Count)
        {
            Self::Geometry
        } else if has(LayoutChangeKind::Restyled) {
            Self::Style
        } else {
            Self::Content
        }
    }

    fn tone(self) -> Tone {
        match self {
            Self::Added => Tone::Ok,
            Self::Removed => Tone::Err,
            Self::Geometry => Tone::Warn,
            Self::Style => Tone::Info,
            Self::Content => Tone::Neutral,
        }
    }

    fn color(self, cx: &App) -> Hsla {
        let theme = cx.theme();
        match self {
            Self::Added => theme.success,
            Self::Removed => theme.danger,
            Self::Geometry => theme.warning,
            Self::Style => theme.info,
            Self::Content => theme.muted_foreground,
        }
    }
}

#[derive(Clone, Copy)]
enum Side {
    Before,
    After,
}

pub struct LayoutView {
    url: String,
    comparison: PageComparison,
    scroll: ScrollHandle,
    selected: Option<usize>,
}

impl LayoutView {
    pub fn new(url: String, mut comparison: PageComparison) -> Self {
        // Layout changes first: they are what the view is opened for.
        comparison
            .diff
            .changes
            .sort_by_key(|change| !change.is_layout());
        Self {
            url,
            comparison,
            scroll: ScrollHandle::new(),
            selected: None,
        }
    }

    fn select(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected = Some(index);
        let Some(change) = self.comparison.diff.changes.get(index) else {
            return;
        };
        let after_top = change
            .after_boxes
            .first()
            .map(|box_index| box_top(&self.comparison.after.0, *box_index));
        let before_top = self.comparison.before.as_ref().and_then(|(layout, _)| {
            change
                .before_boxes
                .first()
                .map(|box_index| box_top(layout, *box_index))
        });
        if let Some(top) = after_top.or(before_top) {
            self.scroll
                .set_offset(point(px(0.), px(-(top - 40.).max(0.))));
        }
        cx.notify();
    }

    /// Each changed box's highlight on one side, and whether it belongs to
    /// the selected change.
    fn highlights(&self, side: Side) -> HashMap<usize, (Highlight, bool)> {
        let mut highlights = HashMap::new();
        for (index, change) in self.comparison.diff.changes.iter().enumerate() {
            let highlight = Highlight::of(change);
            let selected = self.selected == Some(index);
            let boxes = match side {
                Side::Before => &change.before_boxes,
                Side::After => &change.after_boxes,
            };
            for box_index in boxes {
                let entry = highlights
                    .entry(*box_index)
                    .or_insert((highlight, selected));
                entry.1 |= selected;
            }
        }
        highlights
    }

    fn change_row(
        &self,
        index: usize,
        change: &LayoutChange,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let highlight = Highlight::of(change);
        let selected = self.selected == Some(index);
        let details: Vec<SharedString> = change
            .details
            .iter()
            .take(4)
            .map(|detail| SharedString::from(detail.clone()))
            .collect();
        let more = change.details.len().saturating_sub(4);
        div()
            .id(("layout-change", index))
            .px_2()
            .py_1p5()
            .rounded(theme.radius)
            .cursor_pointer()
            .when(selected, |row| row.bg(theme.accent))
            .hover(|row| row.bg(theme.muted))
            .on_click(cx.listener(move |this, _, _, cx| this.select(index, cx)))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        tone_tag(highlight.tone(), cx)
                            .child(SharedString::from(change.kinds_text())),
                    )
                    .when(change.list, |row| {
                        row.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("list"),
                        )
                    }),
            )
            .child(
                div()
                    .mt_1()
                    .text_xs()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_color(cx.theme().foreground)
                    .truncate()
                    .child(SharedString::from(change.label().to_string())),
            )
            .children(details.into_iter().map(|detail| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .truncate()
                    .child(detail)
            }))
            .when(more > 0, |row| {
                row.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(SharedString::from(format!("and {more} more"))),
                )
            })
    }
}

fn box_top(layout: &PageLayout, box_index: usize) -> f32 {
    let scale = COLUMN_WIDTH / layout.width.max(1) as f32;
    layout
        .boxes
        .get(box_index)
        .map_or(0., |layout_box| layout_box.rect.y as f32 * scale)
}

/// The boxes one side draws, scaled to the column, with their colours.
struct Wireframe {
    height: f32,
    quads: Vec<(Rect, Option<(Hsla, bool)>)>,
    scale: f32,
}

fn wireframe(
    layout: &PageLayout,
    tables: &LayoutTables,
    highlights: &HashMap<usize, (Highlight, bool)>,
    cx: &App,
) -> Wireframe {
    let scale = COLUMN_WIDTH / layout.width.max(1) as f32;
    let page_height = (layout.height as f32).min(MAX_PAGE_HEIGHT);
    let mut quads = Vec::new();
    for (index, layout_box) in layout.boxes.iter().enumerate() {
        let rect = layout_box.clip.unwrap_or(layout_box.rect);
        if rect.y as f32 > page_height || rect.width <= 0 || rect.height <= 0 {
            continue;
        }
        match highlights.get(&index) {
            Some((highlight, selected)) => {
                quads.push((rect, Some((highlight.color(cx), *selected))));
            }
            None if paints(layout_box, tables) => quads.push((rect, None)),
            None => {}
        }
    }
    // Highlights last, so nothing plain is drawn over them.
    quads.sort_by_key(|(_, highlight)| highlight.is_some());
    Wireframe {
        height: page_height * scale,
        quads,
        scale,
    }
}

fn wireframe_element(frame: Wireframe, cx: &App) -> impl IntoElement {
    let plain_border = cx.theme().muted_foreground.opacity(0.45);
    let plain_fill = cx.theme().muted_foreground.opacity(0.06);
    let selected_border = cx.theme().foreground;
    let background = cx.theme().background;
    let height = frame.height;
    div()
        .w(px(COLUMN_WIDTH))
        .h(px(height))
        .flex_shrink_0()
        .border_1()
        .border_color(cx.theme().border)
        .bg(background)
        .child(
            canvas(
                |_, _, _| {},
                move |bounds: Bounds<gpui_kit::Pixels>, _, window, _| {
                    for (rect, highlight) in &frame.quads {
                        let origin = bounds.origin
                            + point(
                                px(rect.x as f32 * frame.scale),
                                px(rect.y as f32 * frame.scale),
                            );
                        let extent = size(
                            px((rect.width as f32 * frame.scale).max(1.)),
                            px((rect.height as f32 * frame.scale).max(1.)),
                        );
                        let quad_bounds = Bounds::new(origin, extent);
                        let paint = match highlight {
                            Some((color, selected)) => quad(
                                quad_bounds,
                                px(0.),
                                color.opacity(0.28),
                                px(if *selected { 2. } else { 1. }),
                                if *selected { selected_border } else { *color },
                                BorderStyle::Solid,
                            ),
                            None => quad(
                                quad_bounds,
                                px(0.),
                                plain_fill,
                                px(0.5),
                                plain_border,
                                BorderStyle::Solid,
                            ),
                        };
                        window.paint_quad(paint);
                    }
                },
            )
            .size_full(),
        )
}

impl Render for LayoutView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let diff = &self.comparison.diff;
        let layout_count = diff.layout_change_count();
        let content_count = diff.content_change_count();
        let plural = |count: usize| if count == 1 { "" } else { "s" };
        let mut summary = match (&self.comparison.before, layout_count, content_count) {
            (None, _, _) => {
                "The baseline crawl has no comparable layout for this page.".to_string()
            }
            (Some(_), 0, 0) => "No changes since the baseline crawl.".to_string(),
            (Some(_), layout, content) => format!(
                "{layout} layout change{}, {content} text or image change{}",
                plural(layout),
                plural(content),
            ),
        };
        if diff.consequences > 0 {
            summary.push_str(&format!(
                ". {} change{} that follow from these are not listed",
                diff.consequences,
                plural(diff.consequences)
            ));
        }

        let after_frame = wireframe(
            &self.comparison.after.0,
            &self.comparison.after.1,
            &self.highlights(Side::After),
            cx,
        );
        let before_frame =
            self.comparison.before.as_ref().map(|(layout, tables)| {
                wireframe(layout, tables, &self.highlights(Side::Before), cx)
            });

        let change_rows: Vec<_> = diff
            .changes
            .clone()
            .iter()
            .enumerate()
            .map(|(index, change)| self.change_row(index, change, cx).into_any_element())
            .collect();

        let column = |label: &'static str, frame: Wireframe, cx: &App| {
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(label),
                )
                .child(wireframe_element(frame, cx))
        };

        v_flex()
            .gap_2()
            .child(
                div()
                    .text_xs()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_color(cx.theme().foreground)
                    .truncate()
                    .child(SharedString::from(self.url.clone())),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(summary)),
            )
            .child(
                h_flex()
                    .items_start()
                    .gap_3()
                    .child(
                        div()
                            .id("layout-changes")
                            .w(px(300.))
                            .h(px(WIREFRAME_HEIGHT))
                            .flex_shrink_0()
                            .overflow_y_scroll()
                            .when(change_rows.is_empty(), |list| {
                                list.child(
                                    div()
                                        .p_2()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Nothing to list."),
                                )
                            })
                            .children(change_rows),
                    )
                    .child(
                        div()
                            .id("layout-wireframes")
                            .flex_1()
                            .h(px(WIREFRAME_HEIGHT))
                            .overflow_y_scroll()
                            .track_scroll(&self.scroll)
                            .child(
                                h_flex()
                                    .items_start()
                                    .gap_3()
                                    .when_some(before_frame, |row, frame| {
                                        row.child(column("Baseline crawl", frame, cx))
                                    })
                                    .child(column("This crawl", after_frame, cx)),
                            ),
                    ),
            )
            .child(legend(cx))
    }
}

fn legend(cx: &App) -> impl IntoElement {
    h_flex().gap_2().children(
        [
            (Highlight::Added, "added"),
            (Highlight::Removed, "removed"),
            (Highlight::Geometry, "moved or resized"),
            (Highlight::Style, "restyled"),
            (Highlight::Content, "text or image"),
        ]
        .into_iter()
        .map(|(highlight, label)| tone_tag(highlight.tone(), cx).child(SharedString::from(label))),
    )
}
