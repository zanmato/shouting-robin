//! Plain text in the window's text selection, as `gpui_kit::base::SelectableText`
//! with two changes.
//!
//! gpui-base 0.8.0 asks for the window's whole selected text twice in every
//! paint, to learn whether projecting this run changed it. Each call walks
//! every participant in the window, so a view with `n` selectable texts pays
//! `n²` per frame: the details panel's ~150 values took a re-render from 1.4 ms
//! to 7.1 ms, and to 13 ms with a selection. Only this run's own projection can
//! change during its paint, so comparing it with the one painted last frame
//! answers the same question for one run's cost.
//!
//! It also redraws the window when its selection changes. gpui-base updates the
//! selection on every pointer move of a drag, but its `SelectableText` never
//! asks for a frame, so a selection only showed once something else drew one,
//! usually the release that ended it.
//!
//! Remove this once gpui-base's `SelectableText` does both, and
//! `end_released_selection_drag` once its selection layer ends a drag on a
//! move with no button held.

use gpui_kit::base::{
    TextSelectionHandle, TextSelectionProjection, TextSelectionRegistration, TextSelectionRun,
    Theme,
};
use gpui_kit::{
    App, BorderStyle, Bounds, Corners, Edges, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, Point,
    SharedString, Styled as _, StyledText, Subscription, Window, transparent_black,
};
use std::ops::Range;

pub struct SelectableText {
    id: ElementId,
    text: SharedString,
    styled_text: StyledText,
    document_order: u64,
}

impl SelectableText {
    pub fn new(id: impl Into<ElementId>, text: impl Into<SharedString>) -> Self {
        let text = text.into();
        Self {
            id: id.into(),
            styled_text: StyledText::new(text.clone()),
            text,
            document_order: 0,
        }
    }

    /// Places this text in reading order among the window's selectable text.
    pub fn document_order(mut self, order: u64) -> Self {
        self.document_order = order;
        self
    }

    fn paint_selection(
        layout: &gpui_kit::TextLayout,
        range: Range<usize>,
        color: Hsla,
        window: &mut Window,
    ) {
        let (Some(start), Some(end)) = (
            layout.position_for_index(range.start),
            layout.position_for_index(range.end),
        ) else {
            return;
        };
        for bounds in selection_quad_bounds(start, end, layout.bounds(), layout.line_height()) {
            window.paint_quad(PaintQuad {
                bounds,
                background: color.into(),
                corner_radii: Corners::default(),
                border_widths: Edges::default(),
                border_color: transparent_black(),
                border_style: BorderStyle::default(),
            });
        }
    }
}

fn selection_quad_bounds(
    start: Point<Pixels>,
    end: Point<Pixels>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
) -> Vec<Bounds<Pixels>> {
    if start.y == end.y {
        return vec![Bounds::from_corners(
            start,
            Point::new(end.x, end.y + line_height),
        )];
    }
    let mut quads = vec![Bounds::from_corners(
        start,
        Point::new(bounds.right(), start.y + line_height),
    )];
    if end.y > start.y + line_height {
        quads.push(Bounds::from_corners(
            Point::new(bounds.left(), start.y + line_height),
            Point::new(bounds.right(), end.y),
        ));
    }
    quads.push(Bounds::from_corners(
        Point::new(bounds.left(), end.y),
        Point::new(end.x, end.y + line_height),
    ));
    quads
}

impl IntoElement for SelectableText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// What the element keeps between frames: its participant, the selection it
/// painted last, and the subscription that redraws the window when the
/// selection changes.
struct RetainedState {
    handle: TextSelectionHandle,
    painted: Option<TextSelectionProjection>,
    _redraw_on_change: Subscription,
}

impl RetainedState {
    fn new(handle: TextSelectionHandle, window: &Window, cx: &mut App) -> Self {
        Self {
            _redraw_on_change: handle.refresh_window_on_change(window, cx),
            handle,
            painted: None,
        }
    }
}

impl Element for SelectableText {
    type RequestLayoutState = TextSelectionHandle;
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let handle = window.with_element_state(
            global_id.expect("SelectableText has an element id"),
            |retained: Option<RetainedState>, window| {
                let retained = retained.unwrap_or_else(|| {
                    RetainedState::new(TextSelectionHandle::new(self.text.clone(), cx), window, cx)
                });
                (retained.handle.clone(), retained)
            },
        );
        let (layout_id, ()) = self
            .styled_text
            .request_layout(global_id, inspector_id, window, cx);
        (layout_id, handle)
    }

    fn prepaint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        handle: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        self.styled_text
            .prepaint(global_id, inspector_id, bounds, &mut (), window, cx);
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);
        let registration = TextSelectionRegistration::new(hitbox.clone(), bounds)
            .with_document_order(self.document_order)
            .with_text_bounds(vec![bounds])
            .with_rendered_element(handle, window, cx);
        handle.register(registration, window, cx);
        hitbox
    }

    fn paint(
        &mut self,
        global_id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        handle: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let layout = self.styled_text.layout().clone();
        let projection = handle.update_runs(
            &[
                TextSelectionRun::new(self.text.clone(), layout.clone(), bounds)
                    .with_document_order(self.document_order),
            ],
            cx,
        );
        let changed = window.with_element_state(
            global_id.expect("SelectableText has an element id"),
            |retained: Option<RetainedState>, window| {
                let mut retained =
                    retained.unwrap_or_else(|| RetainedState::new(handle.clone(), window, cx));
                let changed = retained.painted.as_ref() != Some(&projection);
                retained.painted = Some(projection.clone());
                (changed, retained)
            },
        );
        // Whatever painted before this run read the selection as it stood
        // before this projection, so it needs another frame to see it.
        if changed && projection.ranges().iter().any(Option::is_some) {
            window.refresh();
        }
        let color = Theme::global(cx).tokens.colors.selection;
        for range in projection.ranges().iter().flatten().cloned() {
            Self::paint_selection(&layout, range, color, window);
        }
        self.styled_text.paint(
            global_id,
            inspector_id,
            bounds,
            &mut (),
            &mut (),
            window,
            cx,
        );
    }
}

/// Ends a selection drag whose button was released where the window could not
/// see it.
///
/// Pressing the edge of a client-decorated window starts both a resize and,
/// in gpui-base's selection layer, a drag. The compositor keeps the release,
/// so the drag never ends, and the layer extends the selection on every later
/// move whether or not a button is held: moving the pointer after a resize
/// selected text. A move with the button up means the drag is over.
///
/// Render this anywhere above the selection layer in paint order, such as in
/// the window's root view: its listener has to run before the layer's.
pub fn end_released_selection_drag() -> impl IntoElement {
    gpui_kit::canvas(
        |_, _, _| {},
        |_, _, window, _| {
            window.on_mouse_event(|event: &gpui_kit::MouseMoveEvent, phase, window, cx| {
                if phase.bubble() && event.pressed_button != Some(gpui_kit::MouseButton::Left) {
                    gpui_kit::base::TextSelection::end(window, cx);
                }
            });
        },
    )
    .absolute()
    .size_0()
}
