//! Frame cost of the details panel, measured with GPUI's benchmark harness:
//! real text shaping, the wgpu headless renderer on Linux, and allocation
//! counts through the app's own allocator.
//!
//! Ignored by default because it takes a minute and wants a release build:
//!
//! ```sh
//! cargo test --release -p shoutingrobin --bin shoutingrobin details_panel_frames \
//!     -- --ignored --nocapture --test-threads=1
//! ```
//!
//! `DETAILS_BENCH=<substring>` runs only the scenarios whose name contains it.
//! `BENCH_MEASUREMENT` picks what Criterion analyzes (see `bench_metrics`).

use std::time::Duration;

use gpui_kit::{
    AppContext as _, BenchAppContext, BenchMeasurement, BenchReport, Context, Entity, IntoElement,
    Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement as _,
    PlatformInput, Point, Render, Styled as _, Window, bench_platform, div, point, px,
};

use super::{DetailsPanel, DetailsSelection, DetailsTab};
use crate::crawl::event::{
    A11yIssue, Backlink, HreflangIssue, ImageRef, Outlink, PageRecord, SdFormat, SdIssue, SdItem,
    SdSeverity,
};

/// A page at the heavy end of what a crawl produces: a template-linked page
/// with thousands of inlinks, a mega menu of outlinks and a full set of
/// headers, structured data and accessibility findings.
fn heavy_page(variant: usize) -> PageRecord {
    let host = "https://www.example.com";
    let url = format!("{host}/se/kategori/produkt-{variant}");
    PageRecord {
        url: url.clone(),
        status: Some(200),
        content_type: Some("text/html; charset=utf-8".into()),
        size_bytes: 184_320,
        response_time: Duration::from_millis(212),
        title: Some(format!(
            "Produkt {variant} för känslig mage och tarm | Exempelbutiken"
        )),
        meta_description: Some(
            "Ett fullfoder för hundar med känslig mage. Lättsmält protein, präbiotika och \
             omega-3 för en lugn tarm. Fri frakt över 499 kr."
                .into(),
        ),
        h1: Some(format!("Produkt {variant}")),
        h2: Some("Därför fungerar det".into()),
        canonical: Some(url),
        robots: Some("index, follow".into()),
        word_count: Some(1_240),
        depth: Some(3),
        is_internal: true,
        is_page: true,
        indexability: Some("Indexable".into()),
        h1_count: 1,
        h2_count: 6,
        title_count: 1,
        title_pixel_width: Some(512),
        meta_description_pixel_width: Some(920),
        hreflang_tags: (0..8)
            .map(|index| (format!("lang-{index}"), format!("{host}/{index}/produkt")))
            .collect(),
        hreflang_issues: vec![
            HreflangIssue::MissingXDefault,
            HreflangIssue::MissingReturnTag {
                lang: "da".into(),
                target_url: format!("{host}/dk/produkt-{variant}"),
            },
        ],
        sd_types: vec!["Product".into(), "BreadcrumbList".into(), "Organization".into()],
        sd_jsonld_count: 3,
        sd_items: (0..3)
            .map(|index| SdItem {
                format: SdFormat::JsonLd,
                type_name: format!("Type{index}"),
                raw_json: format!(
                    r#"{{"@context":"https://schema.org","@type":"Product","name":"Produkt {variant}","sku":"SKU-{index}","offers":{{"@type":"Offer","price":"349.00","priceCurrency":"SEK","availability":"https://schema.org/InStock"}},"brand":{{"@type":"Brand","name":"Exempel"}}}}"#
                ),
            })
            .collect(),
        sd_issues: (0..4)
            .map(|index| SdIssue {
                severity: if index % 2 == 0 {
                    SdSeverity::Warning
                } else {
                    SdSeverity::Error
                },
                type_name: "Product".into(),
                code: format!("missing_field_{index}"),
                message: "The field is recommended for rich results.".into(),
            })
            .collect(),
        sd_errors: 2,
        sd_warnings: 2,
        lcp_ms: Some(2_340),
        cls: Some(0.04),
        fcp_ms: Some(1_100),
        ttfb_ms: Some(212),
        a11y_errors: 6,
        a11y_warnings: 4,
        a11y_issues: (0..10)
            .map(|index| A11yIssue {
                rule: "color-contrast".into(),
                impact: if index % 3 == 0 { "serious" } else { "moderate" }.into(),
                target: Some(format!("main > section:nth-child({index}) > p")),
                html: Some(format!(
                    r#"<p class="lead muted">Kort text om produkt {index}</p>"#
                )),
            })
            .collect(),
        images: (0..40)
            .map(|index| ImageRef {
                src: format!("{host}/media/produkt-{variant}/bild-{index}.webp"),
                alt: (index % 4 != 0).then(|| format!("Bild {index}")),
                width: Some(800),
                height: (index % 5 != 0).then_some(600),
                has_alt_attr: index % 7 != 0,
            })
            .collect(),
        outlinks: (0..300)
            .map(|index| Outlink {
                dst_url: format!("{host}/se/kategori/sida-{index}"),
                anchor: Some(format!("Länk {index}")),
                rel: (index % 25 == 0).then(|| "nofollow".into()),
                csr_only: index % 10 == 0,
            })
            .collect(),
        backlinks: (0..3_000)
            .map(|index| Backlink {
                source_url: format!("{host}/se/blogg/artikel-{index}"),
                anchor: Some(format!("Läs mer om produkt {variant}")),
                rel: None,
            })
            .collect(),
        inlinks_count: 3_000,
        unique_inlinks_count: 2_800,
        near_duplicate_urls: (0..30)
            .map(|index| format!("{host}/se/kategori/produkt-{index}"))
            .collect(),
        near_duplicate_count: Some(30),
        closest_similarity: Some(94),
        link_score: Some(42.5),
        headers: (0..40)
            .map(|index| {
                (
                    format!("x-header-{index}"),
                    format!("value-{index}-{}", "abcdef".repeat(index % 20)),
                )
            })
            .collect(),
        in_sitemap: Some(true),
        ..PageRecord::default()
    }
}

/// The panel as the app hosts it: next to the guard that ends a selection
/// drag whose release the window never saw.
struct Host {
    panel: Entity<DetailsPanel>,
}

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(crate::ui::selectable_text::end_released_selection_drag())
            .child(self.panel.clone())
    }
}

#[derive(Clone, Copy)]
enum Frame {
    /// The panel re-renders an unchanged page, which is what happens whenever
    /// anything around it notifies.
    Rerender,
    /// The open list scrolls by one row.
    Scroll,
    /// Another page is selected.
    SwitchPage,
}

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    tab: DetailsTab,
    /// Drags a selection across the panel before measuring, so the frames
    /// also pay for publishing it to every selectable run.
    with_selection: bool,
    frame: Frame,
}

const SCENARIOS: [Scenario; 5] = [
    Scenario {
        name: "overview/rerender",
        tab: DetailsTab::Overview,
        with_selection: false,
        frame: Frame::Rerender,
    },
    Scenario {
        name: "overview/rerender-with-selection",
        tab: DetailsTab::Overview,
        with_selection: true,
        frame: Frame::Rerender,
    },
    Scenario {
        name: "inlinks/rerender",
        tab: DetailsTab::Inlinks,
        with_selection: false,
        frame: Frame::Rerender,
    },
    Scenario {
        name: "inlinks/scroll",
        tab: DetailsTab::Inlinks,
        with_selection: false,
        frame: Frame::Scroll,
    },
    Scenario {
        name: "overview/switch-page",
        tab: DetailsTab::Overview,
        with_selection: false,
        frame: Frame::SwitchPage,
    },
];

fn dispatch(window: &mut gpui_kit::BenchWindowContext, event: PlatformInput) {
    window.update(|window, cx| {
        window.dispatch_event(event, cx);
    });
    window.run_until_idle();
}

/// Presses at `from`, drags to `to` and releases, the way a person selects.
fn drag_select(
    window: &mut gpui_kit::BenchWindowContext,
    from: Point<gpui_kit::Pixels>,
    to: Point<gpui_kit::Pixels>,
) {
    dispatch(
        window,
        PlatformInput::MouseDown(MouseDownEvent {
            button: MouseButton::Left,
            position: from,
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        }),
    );
    let draws = |window: &mut gpui_kit::BenchWindowContext| {
        window.update(|window, _| {
            window
                .frame_duration_snapshot()
                .draw_duration_histogram
                .len()
        })
    };
    let draws_before = draws(window);
    move_in_steps(window, from, to, Some(MouseButton::Left));
    // A selection has to show while it is being made, not once the button is
    // released.
    assert!(
        draws(window) > draws_before,
        "dragging a selection drew no frames"
    );
    dispatch(
        window,
        PlatformInput::MouseUp(MouseUpEvent {
            button: MouseButton::Left,
            position: to,
            modifiers: Modifiers::default(),
            click_count: 1,
        }),
    );
}

/// Moves the pointer from `from` to `to` in eight steps, over whatever text
/// lies between them, holding `pressed_button`.
fn move_in_steps(
    window: &mut gpui_kit::BenchWindowContext,
    from: Point<gpui_kit::Pixels>,
    to: Point<gpui_kit::Pixels>,
    pressed_button: Option<MouseButton>,
) {
    for step in 1..=8 {
        let fraction = step as f32 / 8.;
        dispatch(
            window,
            PlatformInput::MouseMove(MouseMoveEvent {
                position: point(
                    from.x + (to.x - from.x) * fraction,
                    from.y + (to.y - from.y) * fraction,
                ),
                pressed_button,
                modifiers: Modifiers::default(),
            }),
        );
    }
}

/// A press whose release the window never gets, as when the compositor takes
/// it for a resize or a table's click handler stops it, followed by the
/// pointer moving with no button held. That is hovering, and must not select.
fn press_with_lost_release(
    window: &mut gpui_kit::BenchWindowContext,
    from: Point<gpui_kit::Pixels>,
    to: Point<gpui_kit::Pixels>,
) {
    dispatch(
        window,
        PlatformInput::MouseDown(MouseDownEvent {
            button: MouseButton::Left,
            position: from,
            modifiers: Modifiers::default(),
            click_count: 1,
            first_mouse: false,
        }),
    );
    move_in_steps(window, from, to, None);
    let selected = window.update(gpui_kit::base::TextSelection::selected_text);
    assert!(
        selected.is_empty(),
        "hovering after a lost release selected:\n{selected}"
    );
}

fn run_scenario(scenario: Scenario, cx: &mut BenchAppContext) {
    cx.update(|cx| {
        gpui_kit::init(cx);
        crate::ui::fonts::register(cx);
    });
    let pages = [heavy_page(1), heavy_page(2)];
    let mut window = cx.add_empty_window();
    let panel: Entity<DetailsPanel> = window.update(|window, cx| {
        let panel = cx.new(|_| DetailsPanel::new());
        let host = cx.new(|_| Host {
            panel: panel.clone(),
        });
        window.replace_root(cx, |window, cx| {
            gpui_kit::component::Root::new(host, window, cx)
        });
        panel
    });
    window.update(|_, cx| {
        panel.update(cx, |panel, cx| {
            panel.set_selected(
                Some(DetailsSelection::Page {
                    record: Box::new(pages[0].clone()),
                    references: Vec::new(),
                }),
                cx,
            );
            panel.active_tab = scenario.tab;
            cx.notify();
        });
    });
    window.run_until_idle();

    if scenario.with_selection {
        press_with_lost_release(
            &mut window,
            point(px(24.), px(120.)),
            point(px(380.), px(900.)),
        );
        drag_select(
            &mut window,
            point(px(24.), px(120.)),
            point(px(380.), px(900.)),
        );
    }
    if scenario.with_selection {
        let selected = window.update(gpui_kit::base::TextSelection::selected_text);
        // A frame with nothing selected would measure the idle case twice.
        // The values have to come out in the order they are shown: a run out
        // of reading order is copied out of order too.
        let position = |needle: &str| {
            selected
                .find(needle)
                .unwrap_or_else(|| panic!("{needle:?} is not in the selection:\n{selected}"))
        };
        assert!(position("text/html") < position("180.0 KB"));
        assert!(position("180.0 KB") < position("index, follow"));
    }

    let app = window.app_context();
    let mut frame_index = 0usize;
    app.bench_renderer(panel, move |panel, _window, cx| {
        frame_index += 1;
        match scenario.frame {
            Frame::Rerender => {}
            Frame::Scroll => {
                let rows = panel.selected_backlink_count();
                let row = frame_index % rows.saturating_sub(40).max(1);
                panel
                    .inlinks_scroll
                    .set_offset(point(px(0.), -(super::LINK_ROW_HEIGHT * row as f32)));
            }
            Frame::SwitchPage => {
                let page = pages[frame_index % pages.len()].clone();
                panel.set_selected(
                    Some(DetailsSelection::Page {
                        record: Box::new(page),
                        references: Vec::new(),
                    }),
                    cx,
                );
            }
        }
        cx.notify();
    });
}

impl DetailsPanel {
    fn selected_backlink_count(&self) -> usize {
        super::selected_page(self).map_or(0, |page| page.backlinks.len())
    }
}

#[test]
#[ignore = "frame benchmark: run in release with --ignored --nocapture"]
fn details_panel_frames() {
    // `gpui_kit::init` follows the desktop's reduce motion setting with a
    // detached task that waits on the portal over D-Bus. Criterion builds a new
    // app for every pass, and that task outlives each one and then panics on
    // the app it was given. Without a session bus it gives up before touching
    // the app. Nothing else in this test process talks to D-Bus.
    // SAFETY: set before any app or thread of this test exists, and the test
    // runs with --test-threads=1.
    unsafe { std::env::set_var("DBUS_SESSION_BUS_ADDRESS", "disabled:") };
    let filter = std::env::var("DETAILS_BENCH").unwrap_or_default();
    let mut criterion = criterion::Criterion::default()
        .with_measurement(BenchMeasurement::from_env_or_exit())
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
        .without_plots();
    for scenario in SCENARIOS
        .into_iter()
        .filter(|scenario| scenario.name.contains(filter.as_str()))
    {
        let report = BenchReport::with_fps(120);
        criterion.bench_function(scenario.name, {
            let report = report.clone();
            move |bencher| {
                let mut cx = BenchAppContext::new_with_platform_and_report(
                    bench_platform(
                        Some(Box::new(gpui_kit::platform::current_headless_renderer)),
                        gpui_kit::platform::bench_text_system(),
                    ),
                    Some("details_panel_frames"),
                    bencher,
                    report.clone(),
                );
                run_scenario(scenario, &mut cx);
                cx.teardown();
            }
        });
        report.print(scenario.name);
    }
}
