use gpui::SharedString;

use crate::ui::icon::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResultTab {
    Internal,
    External,
    ResponseCodes,
    PageTitles,
    MetaDesc,
    H1,
    H2,
    Content,
    Images,
    Canonicals,
    Hreflang,
    StructuredData,
    Accessibility,
    Performance,
    Ecommerce,
    Sitemaps,
    Security,
    Url,
    Directives,
    Overview,
}

impl ResultTab {
    pub const ALL: &'static [ResultTab] = &[
        ResultTab::Overview,
        ResultTab::Internal,
        ResultTab::External,
        ResultTab::ResponseCodes,
        ResultTab::PageTitles,
        ResultTab::MetaDesc,
        ResultTab::H1,
        ResultTab::H2,
        ResultTab::Content,
        ResultTab::Images,
        ResultTab::Canonicals,
        ResultTab::Hreflang,
        ResultTab::StructuredData,
        ResultTab::Accessibility,
        ResultTab::Performance,
        ResultTab::Ecommerce,
        ResultTab::Sitemaps,
        ResultTab::Security,
        ResultTab::Url,
        ResultTab::Directives,
    ];

    pub fn label(self) -> SharedString {
        match self {
            ResultTab::Internal => "Internal".into(),
            ResultTab::External => "External".into(),
            ResultTab::ResponseCodes => "Response Codes".into(),
            ResultTab::PageTitles => "Page Titles".into(),
            ResultTab::MetaDesc => "Meta Desc".into(),
            ResultTab::H1 => "H1".into(),
            ResultTab::H2 => "H2".into(),
            ResultTab::Content => "Content".into(),
            ResultTab::Images => "Images".into(),
            ResultTab::Canonicals => "Canonicals".into(),
            ResultTab::Hreflang => "Hreflang".into(),
            ResultTab::StructuredData => "Structured Data".into(),
            ResultTab::Accessibility => "Accessibility".into(),
            ResultTab::Performance => "Performance".into(),
            ResultTab::Ecommerce => "Ecommerce".into(),
            ResultTab::Sitemaps => "Sitemaps".into(),
            ResultTab::Security => "Security".into(),
            ResultTab::Url => "URL".into(),
            ResultTab::Directives => "Directives".into(),
            ResultTab::Overview => "Overview".into(),
        }
    }

    pub fn icon(self) -> Option<Icon> {
        match self {
            ResultTab::Images => Some(Icon::Image),
            ResultTab::StructuredData => Some(Icon::Braces),
            ResultTab::Accessibility => Some(Icon::Accessibility),
            ResultTab::Performance => Some(Icon::Gauge),
            ResultTab::Ecommerce => Some(Icon::ShoppingBag),
            ResultTab::Sitemaps => Some(Icon::Map),
            ResultTab::Security => Some(Icon::ShieldCheck),
            ResultTab::Overview => Some(Icon::Layers),
            _ => None,
        }
    }
}
