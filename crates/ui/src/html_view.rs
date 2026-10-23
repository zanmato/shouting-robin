use std::ops::Range;
use std::sync::Arc;

use gpui_kit::component::{
    ActiveTheme, highlighter::SyntaxHighlighter, scroll::ScrollableElement as _,
};
use gpui_kit::{
    App, ElementId, HighlightStyle, InteractiveElement as _, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, StyledText, Window, div, px,
};

struct CachedHtml {
    text: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
}

/// A highlighted HTML snippet that scrolls sideways when it is wider than its
/// container.
///
/// `id` keys the scroll position. Without one, every view would take it from
/// the line that makes the scroll area, and all the snippets on screen would
/// scroll together.
#[derive(Clone, IntoElement)]
pub struct HtmlView {
    id: ElementId,
    cached: Arc<CachedHtml>,
    copyable: bool,
}

impl HtmlView {
    pub fn new(id: impl Into<ElementId>, html: &str, cx: &App) -> Self {
        let highlight_theme = &cx.theme().highlight_theme;

        let mut highlighter = SyntaxHighlighter::new("html");
        let rope = ropey::Rope::from(html);
        highlighter.update(None, &rope, None);
        let highlights = highlighter.styles(&(0..html.len()), highlight_theme.as_ref());

        Self {
            id: id.into(),
            cached: Arc::new(CachedHtml {
                text: SharedString::from(html.to_string()),
                highlights,
            }),
            copyable: false,
        }
    }

    /// Shows a copy button over the snippet while the pointer is on it.
    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
}

impl RenderOnce for HtmlView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let data = &self.cached;
        let background = theme
            .highlight_theme
            .style
            .editor_background
            .unwrap_or(theme.background);
        let snippet = div()
            .w_full()
            .px_2()
            .py(px(1.))
            .bg(background)
            .rounded(theme.radius)
            .font_family(theme.mono_font_family.clone())
            .text_size(px(12.))
            .text_color(theme.foreground)
            .overflow_x_scrollbar()
            // Under the view's own id, so each snippet keeps its own position.
            .id("scroll")
            .child(
                StyledText::new(data.text.clone()).with_highlights(data.highlights.iter().cloned()),
            );
        if self.copyable {
            crate::with_copy_button(self.id, data.text.clone(), background, snippet, window, cx)
        } else {
            div().id(self.id).w_full().child(snippet).into_any_element()
        }
    }
}
