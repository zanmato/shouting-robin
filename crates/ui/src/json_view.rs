use std::ops::Range;
use std::sync::Arc;

use gpui_kit::component::{ActiveTheme, highlighter::SyntaxHighlighter};
use gpui_kit::{
    App, ElementId, HighlightStyle, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    StyledText, Window, div, px,
};

struct CachedJson {
    formatted: SharedString,
    highlights: Vec<(Range<usize>, HighlightStyle)>,
}

/// Pretty-printed, highlighted JSON. `id` names the view, which only matters
/// once it is [`JsonView::copyable`]: the copy button keeps its state under it.
#[derive(Clone, IntoElement)]
pub struct JsonView {
    id: ElementId,
    cached: Arc<CachedJson>,
    copyable: bool,
}

impl JsonView {
    pub fn new(id: impl Into<ElementId>, json: &str, cx: &App) -> Self {
        let formatted = format_json(json);
        let highlight_theme = &cx.theme().highlight_theme;

        let mut highlighter = SyntaxHighlighter::new("json");
        let rope = ropey::Rope::from(formatted.as_str());
        highlighter.update(None, &rope, None);
        let highlights = highlighter.styles(&(0..formatted.len()), highlight_theme.as_ref());

        Self {
            id: id.into(),
            cached: Arc::new(CachedJson {
                formatted: SharedString::from(formatted),
                highlights,
            }),
            copyable: false,
        }
    }

    /// Shows a copy button over the JSON while the pointer is on it. It copies
    /// the JSON as shown, pretty-printed.
    pub fn copyable(mut self) -> Self {
        self.copyable = true;
        self
    }
}

fn format_json(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| raw.to_string())
}

impl RenderOnce for JsonView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let data = &self.cached;
        let background = theme
            .highlight_theme
            .style
            .editor_background
            .unwrap_or(theme.background);
        let json = div()
            .w_full()
            .p_2()
            .bg(background)
            .rounded(theme.radius)
            .font_family(theme.mono_font_family.clone())
            .text_size(px(12.))
            .text_color(theme.foreground)
            .child(
                StyledText::new(data.formatted.clone())
                    .with_highlights(data.highlights.iter().cloned()),
            );
        if self.copyable {
            crate::with_copy_button(
                self.id,
                data.formatted.clone(),
                background,
                json,
                window,
                cx,
            )
        } else {
            json.into_any_element()
        }
    }
}
