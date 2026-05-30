use gpui::{
    App, IntoElement, ParentElement, RenderOnce, SharedString, Styled, StyledText, Window, div, px,
};
use gpui_component::{ActiveTheme, highlighter::SyntaxHighlighter};

#[derive(IntoElement)]
pub struct JsonView {
    json: String,
}

impl JsonView {
    pub fn new(json: impl Into<String>) -> Self {
        Self { json: json.into() }
    }

    fn format_json(raw: &str) -> String {
        serde_json::from_str::<serde_json::Value>(raw)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or_else(|| raw.to_string())
    }
}

impl RenderOnce for JsonView {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let formatted = Self::format_json(&self.json);
        let theme = cx.theme();
        let highlight_theme = theme.highlight_theme.clone();

        let mut highlighter = SyntaxHighlighter::new("json");
        let rope = ropey::Rope::from(formatted.as_str());
        highlighter.update(None, &rope, None);
        let highlights = highlighter.styles(&(0..formatted.len()), &highlight_theme);

        div()
            .w_full()
            .p_2()
            .bg(highlight_theme
                .style
                .editor_background
                .unwrap_or(theme.background))
            .rounded(theme.radius)
            .font_family(theme.mono_font_family.clone())
            .text_size(px(12.))
            .text_color(theme.foreground)
            .child(StyledText::new(SharedString::from(formatted)).with_highlights(highlights))
    }
}
