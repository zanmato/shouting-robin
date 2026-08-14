#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderMode {
    #[default]
    Http,
    Chrome,
}

impl RenderMode {
    /// The mode a stored crawl ran in. Anything that isn't the Chrome marker is
    /// plain HTTP, which is also what a row written before the column existed
    /// should read as.
    pub fn from_stored(value: &str) -> Self {
        if value == "chrome" {
            Self::Chrome
        } else {
            Self::Http
        }
    }

    /// True when the crawl ran a real browser, and so could measure the things
    /// only a rendered page has: accessibility violations and Core Web Vitals.
    pub fn renders_javascript(self) -> bool {
        matches!(self, Self::Chrome)
    }
}
