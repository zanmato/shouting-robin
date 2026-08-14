//! The font bundled with the app, and where it is used.
//!
//! Google Sans, under the OFL (see `assets/fonts/OFL.txt`). It is here for
//! three reasons that happen to point the same way:
//!
//! * It is the font Google sets result-page titles in, so the SERP preview and
//!   the title pixel-width rules describe the same thing the user will see.
//!   `crawl::font_metrics` already measures titles against its advance widths.
//! * A PDF report is a document someone sends on, so it should look the same
//!   wherever it was written rather than depending on what the machine happens
//!   to have installed.
//! * The PDF layout engine mis-advances fonts carrying a legacy `kern` table
//!   alongside GPOS — DejaVu Sans and Liberation Sans, which is what a Linux
//!   box answers "sans-serif" with, come out with gaps inside words. Google
//!   Sans has GPOS only and 1000 units to the em, and renders correctly.

use gpui::App;

/// The family name the faces register under, in the PDF markup and in the app.
pub const FAMILY: &str = "Google Sans";

pub const REGULAR: &[u8] = include_bytes!("../../assets/fonts/GoogleSans-Regular.ttf");
pub const BOLD: &[u8] = include_bytes!("../../assets/fonts/GoogleSans-Bold.ttf");

/// Makes the bundled family available to the UI's text system, so an element
/// can ask for it by name. Called once at startup; a failure is logged and
/// leaves the app on its default font rather than stopping it.
pub fn register(cx: &App) {
    if let Err(e) = cx.text_system().add_fonts(vec![
        std::borrow::Cow::Borrowed(REGULAR),
        std::borrow::Cow::Borrowed(BOLD),
    ]) {
        tracing::warn!(error=%e, "failed to register the bundled font");
    }
}
