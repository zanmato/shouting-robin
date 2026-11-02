use gpui_kit::SharedString;

use crate::ui::icon::Icon;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    SiteStructure,
    Security,
    Url,
    Directives,
    Overview,
    Links,
    Changes,
    Layout,
}

impl ResultTab {
    pub const ALL: &'static [ResultTab] = &[
        ResultTab::Overview,
        ResultTab::Changes,
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
        ResultTab::Layout,
        ResultTab::Ecommerce,
        ResultTab::Sitemaps,
        ResultTab::SiteStructure,
        ResultTab::Security,
        ResultTab::Url,
        ResultTab::Directives,
        ResultTab::Links,
    ];

    pub fn label(self) -> SharedString {
        match self {
            ResultTab::Internal => t!("app.tab.internal").into(),
            ResultTab::External => t!("app.tab.external").into(),
            ResultTab::ResponseCodes => t!("app.tab.response_codes").into(),
            ResultTab::PageTitles => t!("app.tab.page_titles").into(),
            ResultTab::MetaDesc => t!("app.tab.meta_desc").into(),
            ResultTab::H1 => "H1".into(),
            ResultTab::H2 => "H2".into(),
            ResultTab::Content => t!("app.tab.content").into(),
            ResultTab::Images => t!("app.tab.images").into(),
            ResultTab::Canonicals => t!("app.tab.canonicals").into(),
            ResultTab::Hreflang => "Hreflang".into(),
            ResultTab::StructuredData => t!("app.tab.structured_data").into(),
            ResultTab::Accessibility => t!("app.tab.accessibility").into(),
            ResultTab::Performance => t!("app.tab.performance").into(),
            ResultTab::Ecommerce => t!("app.tab.ecommerce").into(),
            ResultTab::Sitemaps => t!("app.tab.sitemaps").into(),
            ResultTab::SiteStructure => t!("app.tab.site_structure").into(),
            ResultTab::Security => t!("app.tab.security").into(),
            ResultTab::Url => "URL".into(),
            ResultTab::Directives => t!("app.tab.directives").into(),
            ResultTab::Overview => t!("app.tab.overview").into(),
            ResultTab::Links => t!("app.tab.links").into(),
            ResultTab::Changes => t!("app.tab.changes").into(),
            ResultTab::Layout => t!("app.tab.layout").into(),
        }
    }

    /// A line explaining what this tab counts, where the answer is deliberate
    /// and would otherwise read as a defect.
    ///
    /// Both notes below record a decision rather than describe the code: other
    /// tools draw these two lines elsewhere, and someone comparing the numbers
    /// deserves to know we chose the wider reading on purpose.
    pub fn note(self) -> Option<SharedString> {
        match self {
            ResultTab::Security => Some(t!("app.tab.note.security").into()),
            ResultTab::Url => Some(t!("app.tab.note.url").into()),
            _ => None,
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
            ResultTab::SiteStructure => Some(Icon::FolderOpen),
            ResultTab::Security => Some(Icon::ShieldCheck),
            ResultTab::Overview => Some(Icon::CircleAlert),
            ResultTab::Links => Some(Icon::Link),
            ResultTab::Changes => Some(Icon::ArrowUpDown),
            ResultTab::Layout => Some(Icon::LayoutPanelLeft),
            _ => None,
        }
    }
}
