// gpui-fast's macros emit `gpui::` paths, and GPUI Kit is the only GPUI
// dependency here, so it has to answer to that name.
extern crate gpui_kit as gpui;

pub mod html_view;
pub mod json_view;
pub mod tab;

pub use html_view::HtmlView;
pub use json_view::JsonView;
pub use tab::{Tab, TabBar, TabVariant};
