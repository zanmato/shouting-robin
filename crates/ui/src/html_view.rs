use std::ops::Range;
use std::sync::Arc;

use gpui::{
    App, HighlightStyle, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText,
    Window, div, px,
};
use gpui_component::{ActiveTheme, highlighter::SyntaxHighlighter, scroll::ScrollableElement as _};

struct CachedHtml {
    text: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
}

#[derive(Clone, IntoElement)]
pub struct HtmlView(Arc<CachedHtml>);

impl HtmlView {
    pub fn new(html: &str, cx: &App) -> Self {
        let highlight_theme = &cx.theme().highlight_theme;

        let mut highlighter = SyntaxHighlighter::new("html");
        let rope = ropey::Rope::from(html);
        highlighter.update(None, &rope, None);
        let highlights = highlighter.styles(&(0..html.len()), highlight_theme.as_ref());

        Self(Arc::new(CachedHtml {
            text: SharedString::from(html.to_string()),
            highlights,
        }))
    }
}

impl RenderOnce for HtmlView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let data = &self.0;
        div()
            .w_full()
            .px_2()
            .py(px(1.))
            .bg(theme
                .highlight_theme
                .style
                .editor_background
                .unwrap_or(theme.background))
            .rounded(theme.radius)
            .font_family(theme.mono_font_family.clone())
            .text_size(px(12.))
            .text_color(theme.foreground)
            .overflow_x_scrollbar()
            .child(
                StyledText::new(data.text.clone()).with_highlights(data.highlights.iter().cloned()),
            )
    }
}
