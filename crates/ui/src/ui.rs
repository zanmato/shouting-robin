// gpui-fast's macros emit `gpui::` paths, and GPUI Kit is the only GPUI
// dependency here, so it has to answer to that name.
extern crate gpui_kit as gpui;

pub mod html_view;
pub mod json_view;
pub mod tab;

use gpui_kit::component::{Sizable as _, clipboard::Clipboard};
use gpui_kit::{
    AnyElement, App, ElementId, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _,
};

pub use html_view::HtmlView;
pub use json_view::JsonView;
pub use tab::{Tab, TabBar, TabVariant};

/// `content` with a copy button for `text` in its top right corner while the
/// pointer is over it.
///
/// The button is only built while it shows. A button per snippet on every
/// frame cost the details panel about 23 µs each, for buttons nobody could
/// see. It sits beside the content rather than in it, so a snippet that
/// scrolls sideways does not carry it away, and takes the snippet's
/// background so the code under it does not show through.
fn with_copy_button(
    id: ElementId,
    text: SharedString,
    background: Hsla,
    content: impl IntoElement,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let hovered = window.use_keyed_state(id.clone(), cx, |_, _| false);
    let show_button = *hovered.read(cx);
    div()
        .id(id)
        .relative()
        .w_full()
        .child(content)
        .on_hover(move |hovering, _, cx| {
            hovered.update(cx, |hovered, cx| {
                if *hovered != *hovering {
                    *hovered = *hovering;
                    cx.notify();
                }
            });
        })
        .when(show_button, |snippet| {
            snippet.child(
                div()
                    .absolute()
                    .top_0p5()
                    .right_0p5()
                    .rounded_sm()
                    .bg(background)
                    .child(Clipboard::new("copy").value(text).tooltip("Copy").xsmall()),
            )
        })
        .into_any_element()
}
