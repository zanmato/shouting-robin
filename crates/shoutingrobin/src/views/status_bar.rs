use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::ActiveTheme;

/// Live counts for the footer. Each figure is the number of rows of one kind
/// the crawl has produced so far, so it only ever goes up.
pub struct StatusBar {
    /// HTML documents the crawler navigated to.
    pub pages: u64,
    /// Images, scripts, styles, fonts and API calls those pages pulled in.
    pub resources: u64,
    pub errors: u64,
    pub running: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            pages: 0,
            resources: 0,
            errors: 0,
            running: false,
        }
    }

    pub fn reset(&mut self) {
        self.pages = 0;
        self.resources = 0;
        self.errors = 0;
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for StatusBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let label = if self.running { "Crawling" } else { "Idle" };
        div()
            .flex()
            .items_center()
            .gap_4()
            .px_3()
            .h(gpui::px(28.))
            .bg(theme.background)
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(div().child(label))
            .child(div().child(format!("Pages {}", self.pages)))
            .child(div().child(format!("Resources {}", self.resources)))
            .child(div().child(format!("Errors {}", self.errors)))
    }
}
