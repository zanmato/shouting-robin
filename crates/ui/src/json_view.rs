use std::ops::Range;
use std::sync::Arc;

use gpui::{
    App, HighlightStyle, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText,
    Window, div, px,
};
use gpui_component::{ActiveTheme, highlighter::SyntaxHighlighter};

struct CachedJson {
    formatted: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
}

#[derive(Clone, IntoElement)]
pub struct JsonView(Arc<CachedJson>);

impl JsonView {
    pub fn new(json: &str, cx: &App) -> Self {
        let formatted = format_json(json);
        let highlight_theme = &cx.theme().highlight_theme;

        let mut highlighter = SyntaxHighlighter::new("json");
        let rope = ropey::Rope::from(formatted.as_str());
        highlighter.update(None, &rope, None);
        let highlights = highlighter.styles(&(0..formatted.len()), highlight_theme.as_ref());

        Self(Arc::new(CachedJson {
            formatted: SharedString::from(formatted),
            highlights,
        }))
    }
}

fn format_json(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| raw.to_string())
}

impl RenderOnce for JsonView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let data = &self.0;
        div()
            .w_full()
            .p_2()
            .bg(theme
                .highlight_theme
                .style
                .editor_background
                .unwrap_or(theme.background))
            .rounded(theme.radius)
            .font_family(theme.mono_font_family.clone())
            .text_size(px(12.))
            .text_color(theme.foreground)
            .child(
                StyledText::new(data.formatted.clone())
                    .with_highlights(data.highlights.iter().cloned()),
            )
    }
}
