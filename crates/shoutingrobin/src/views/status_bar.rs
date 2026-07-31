use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div};
use gpui_component::ActiveTheme;

pub struct StatusBar {
    pub crawled: u64,
    pub queued: u64,
    pub errors: u64,
    pub running: bool,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            crawled: 0,
            queued: 0,
            errors: 0,
            running: false,
        }
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
            .child(div().child(format!("Crawled {}", self.crawled)))
            .child(div().child(format!("Queue {}", self.queued)))
            .child(div().child(format!("Errors {}", self.errors)))
    }
}
