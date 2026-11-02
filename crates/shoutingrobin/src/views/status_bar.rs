use gpui_kit::component::ActiveTheme;
use gpui_kit::{Context, IntoElement, ParentElement, Render, SharedString, Styled, Window, div};

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
        let label = if self.running {
            t!("details.status.crawling")
        } else {
            t!("details.status.idle")
        };
        div()
            .flex()
            .items_center()
            .gap_4()
            .px_3()
            .h(gpui_kit::px(28.))
            .bg(theme.background)
            .text_xs()
            .text_color(theme.muted_foreground)
            .child(div().child(SharedString::from(label)))
            .child(div().child(SharedString::from(t!(
                "details.status.pages",
                count = self.pages
            ))))
            .child(div().child(SharedString::from(t!(
                "details.status.resources",
                count = self.resources
            ))))
            .child(div().child(SharedString::from(t!(
                "details.status.errors",
                count = self.errors
            ))))
    }
}
