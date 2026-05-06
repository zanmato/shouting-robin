#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderMode {
    #[default]
    Http,
    Chrome,
}

impl RenderMode {
    pub fn label(self) -> &'static str {
        match self {
            RenderMode::Http => "Crawl (HTTP)",
            RenderMode::Chrome => "Crawl (Chrome)",
        }
    }
}
