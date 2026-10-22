//! Site icons for the crawl history.
//!
//! Icons are stored once per host as a small PNG, so the history list never
//! touches the network to draw them. A crawl refreshes the icon of the host it
//! starts on, and the history fetches icons for older crawls that have none.

use std::io::Cursor;
use std::time::Duration;

use anyhow::Context as _;
use image::{RgbaImage, imageops};
use resvg::{tiny_skia, usvg};
use sqlx::SqlitePool;
use url::Url;

/// Twice the size the history draws, so the icon stays sharp on HiDPI screens.
pub const ICON_SIZE: u32 = 32;

/// Larger than any sensible favicon. A response this big is not one.
const MAX_ICON_BYTES: usize = 1024 * 1024;

/// How long a host without an icon is left alone before it is tried again.
pub const RETRY_MISSING_AFTER_SECONDS: i64 = 7 * 24 * 60 * 60;

/// The key icons are stored under: the host, plus the port when it is not the
/// scheme's default, since two ports can serve two different sites.
pub fn icon_host(page_url: &str) -> Option<String> {
    let parsed = Url::parse(page_url).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    Some(match parsed.port() {
        Some(port) => format!("{host}:{port}"),
        None => host,
    })
}

/// A client for fetching icons outside a crawl, where no crawl config applies.
pub fn icon_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(concat!("ShoutingRobin/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// The icon URLs a page declares, best first, followed by `/favicon.ico`,
/// which browsers fall back to when a page declares none.
pub fn icon_candidates(html: &str, page_url: &Url) -> Vec<Url> {
    let document = scraper::Html::parse_document(html);
    let base_url = scraper::Selector::parse("base[href]")
        .ok()
        .and_then(|selector| {
            document
                .select(&selector)
                .next()
                .and_then(|base| base.value().attr("href"))
                .and_then(|href| page_url.join(href).ok())
        })
        .unwrap_or_else(|| page_url.clone());

    let mut ranked: Vec<(u32, Url)> = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("link[rel][href]") {
        for link in document.select(&selector) {
            let element = link.value();
            let (Some(rel), Some(href)) = (element.attr("rel"), element.attr("href")) else {
                continue;
            };
            let Some(rank) = icon_rank(rel, element.attr("sizes"), element.attr("type"), href)
            else {
                continue;
            };
            if let Ok(url) = base_url.join(href.trim()) {
                ranked.push((rank, url));
            }
        }
    }
    ranked.sort_by_key(|(rank, _)| *rank);

    let mut candidates: Vec<Url> = Vec::new();
    for (_, url) in ranked {
        if !candidates.contains(&url) {
            candidates.push(url);
        }
    }
    if let Ok(fallback) = page_url.join("/favicon.ico")
        && !candidates.contains(&fallback)
    {
        candidates.push(fallback);
    }
    candidates
}

/// Lower is better, `None` for links that are not icons. A vector scales to
/// any size. Raster icons rank by how close they are to `ICON_SIZE`, larger
/// before smaller since scaling down keeps detail. Touch icons come last:
/// they are made for home screens and often carry an opaque background.
fn icon_rank(rel: &str, sizes: Option<&str>, mime_type: Option<&str>, href: &str) -> Option<u32> {
    let rel_tokens: Vec<String> = rel
        .split_ascii_whitespace()
        .map(str::to_ascii_lowercase)
        .collect();
    let is_icon = rel_tokens.iter().any(|token| token == "icon");
    let is_touch_icon = rel_tokens
        .iter()
        .any(|token| token == "apple-touch-icon" || token == "apple-touch-icon-precomposed");
    if !is_icon && !is_touch_icon {
        return None;
    }

    let is_vector = mime_type.is_some_and(|mime| mime.eq_ignore_ascii_case("image/svg+xml"))
        || href
            .split(['?', '#'])
            .next()
            .is_some_and(|path| path.to_ascii_lowercase().ends_with(".svg"));
    let largest_side = sizes.and_then(|sizes| {
        sizes
            .split_ascii_whitespace()
            .filter_map(|size| {
                let (width, height) = size
                    .to_ascii_lowercase()
                    .split_once('x')
                    .map(|(w, h)| (w.parse::<u32>().ok(), h.parse::<u32>().ok()))?;
                Some(width?.max(height?))
            })
            .max()
    });
    let declares_any = sizes.is_some_and(|sizes| sizes.eq_ignore_ascii_case("any"));

    let rank = if is_vector || declares_any {
        0
    } else {
        match largest_side {
            Some(side) if side >= ICON_SIZE => 1 + side - ICON_SIZE,
            Some(side) => 1_000 + (ICON_SIZE - side),
            // Usually a 16 and 32 pixel .ico, which suits the list well.
            None => 500,
        }
    };
    Some(if is_icon { rank } else { 10_000 + rank })
}

/// Turns icon bytes in any common format into a square `ICON_SIZE` PNG,
/// centred on a transparent background when the icon is not square.
pub fn render_icon(bytes: &[u8]) -> anyhow::Result<Vec<u8>> {
    let image = match image::guess_format(bytes) {
        Ok(format) => image::load_from_memory_with_format(bytes, format)
            .context("icon could not be decoded")?
            .into_rgba8(),
        Err(_) => rasterize_svg(bytes)?,
    };
    let fitted = fit_to_icon(image);
    let mut png = Vec::new();
    fitted
        .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .context("icon could not be encoded")?;
    Ok(png)
}

fn rasterize_svg(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
    let tree = usvg::Tree::from_data(bytes, &usvg::Options::default())
        .context("icon is neither a known image format nor SVG")?;
    let size = tree.size();
    let scale = ICON_SIZE as f32 / size.width().max(size.height());
    let offset_x = (ICON_SIZE as f32 - size.width() * scale) / 2.0;
    let offset_y = (ICON_SIZE as f32 - size.height() * scale) / 2.0;
    let mut pixmap =
        tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE).context("icon canvas is empty")?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y),
        &mut pixmap.as_mut(),
    );
    let pixels: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|pixel| {
            let color = pixel.demultiply();
            [color.red(), color.green(), color.blue(), color.alpha()]
        })
        .collect();
    RgbaImage::from_raw(ICON_SIZE, ICON_SIZE, pixels).context("icon pixels do not fit the canvas")
}

fn fit_to_icon(image: RgbaImage) -> RgbaImage {
    let (width, height) = image.dimensions();
    if width == ICON_SIZE && height == ICON_SIZE {
        return image;
    }
    let scale = ICON_SIZE as f32 / width.max(height).max(1) as f32;
    let scaled_width = ((width as f32 * scale).round() as u32).clamp(1, ICON_SIZE);
    let scaled_height = ((height as f32 * scale).round() as u32).clamp(1, ICON_SIZE);
    let scaled = imageops::resize(
        &image,
        scaled_width,
        scaled_height,
        imageops::FilterType::Lanczos3,
    );
    let mut canvas = RgbaImage::new(ICON_SIZE, ICON_SIZE);
    imageops::overlay(
        &mut canvas,
        &scaled,
        i64::from((ICON_SIZE - scaled_width) / 2),
        i64::from((ICON_SIZE - scaled_height) / 2),
    );
    canvas
}

/// The first icon of `page_url` that loads and decodes, as a PNG. `None`
/// when the site has none. Failures of single candidates are expected (a
/// stale link, a missing `/favicon.ico`) and only end the search when every
/// candidate failed.
pub async fn fetch_site_icon(
    client: &reqwest::Client,
    page_url: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let requested = Url::parse(page_url).context("page URL is not valid")?;
    let candidates = match client.get(requested.clone()).send().await {
        Ok(response) => {
            // Relative icon links resolve against where the page ended up
            // after redirects, not where the crawl started.
            let final_url = response.url().clone();
            match response.text().await {
                Ok(html) => icon_candidates(&html, &final_url),
                Err(e) => {
                    tracing::debug!(url = %final_url, error = %e, "page for site icon unreadable");
                    icon_candidates("", &final_url)
                }
            }
        }
        Err(e) => {
            tracing::debug!(url = %requested, error = %e, "page for site icon failed to load");
            icon_candidates("", &requested)
        }
    };

    for candidate in candidates {
        match fetch_icon_bytes(client, &candidate).await {
            Ok(bytes) => match render_icon(&bytes) {
                Ok(png) => return Ok(Some(png)),
                Err(e) => tracing::debug!(url = %candidate, error = %e, "site icon not usable"),
            },
            Err(e) => tracing::debug!(url = %candidate, error = %e, "site icon failed to load"),
        }
    }
    Ok(None)
}

async fn fetch_icon_bytes(client: &reqwest::Client, url: &Url) -> anyhow::Result<Vec<u8>> {
    let response = client.get(url.clone()).send().await?.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_ICON_BYTES as u64)
    {
        anyhow::bail!("icon is larger than {MAX_ICON_BYTES} bytes");
    }
    let bytes = response.bytes().await?;
    if bytes.len() > MAX_ICON_BYTES {
        anyhow::bail!("icon is larger than {MAX_ICON_BYTES} bytes");
    }
    Ok(bytes.to_vec())
}

/// Fetches and stores the icon of the host `page_url` is on. A failed fetch
/// is stored too, so the history does not ask again on every start, and it
/// never replaces an icon fetched earlier.
pub async fn refresh_site_icon(
    pool: &SqlitePool,
    client: &reqwest::Client,
    page_url: &str,
) -> anyhow::Result<Option<Vec<u8>>> {
    let host = icon_host(page_url).context("page URL has no host")?;
    let icon = match fetch_site_icon(client, page_url).await {
        Ok(icon) => icon,
        Err(e) => {
            tracing::debug!(%host, error = %e, "site icon fetch failed");
            None
        }
    };
    crate::storage::save_site_icon(pool, &host, icon.as_deref()).await?;
    Ok(icon)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Url {
        Url::parse("https://shop.example/category/page.html").expect("valid URL")
    }

    #[test]
    fn hosts_keep_non_default_ports() {
        assert_eq!(
            icon_host("https://WWW.Example.com/a").as_deref(),
            Some("www.example.com")
        );
        assert_eq!(
            icon_host("https://example.com:443/").as_deref(),
            Some("example.com")
        );
        assert_eq!(
            icon_host("http://127.0.0.1:8765/shop/").as_deref(),
            Some("127.0.0.1:8765")
        );
        assert_eq!(icon_host("not a url"), None);
    }

    #[test]
    fn candidates_prefer_vectors_then_sizes_near_the_icon_size() {
        let html = r#"<html><head>
            <link rel="apple-touch-icon" href="/touch.png">
            <link rel="icon" sizes="16x16" href="/16.png">
            <link rel="icon" sizes="192x192" href="/192.png">
            <link rel="icon" sizes="32x32" href="/32.png">
            <link rel="shortcut icon" href="favicon.svg">
            <link rel="stylesheet" href="/style.css">
        </head></html>"#;
        let candidates: Vec<String> = icon_candidates(html, &page())
            .into_iter()
            .map(|url| url.path().to_string())
            .collect();
        assert_eq!(
            candidates,
            [
                "/category/favicon.svg",
                "/32.png",
                "/192.png",
                "/16.png",
                "/touch.png",
                "/favicon.ico"
            ]
        );
    }

    #[test]
    fn candidates_resolve_against_the_base_element() {
        let html = r#"<head><base href="https://cdn.example/assets/"><link rel="icon" href="icon.png"></head>"#;
        let candidates = icon_candidates(html, &page());
        assert_eq!(
            candidates.first().map(Url::as_str),
            Some("https://cdn.example/assets/icon.png")
        );
        assert_eq!(
            candidates.last().map(Url::as_str),
            Some("https://shop.example/favicon.ico")
        );
    }

    #[test]
    fn a_page_without_icon_links_falls_back_to_favicon_ico() {
        let candidates = icon_candidates("<p>no head</p>", &page());
        assert_eq!(
            candidates.iter().map(Url::as_str).collect::<Vec<_>>(),
            ["https://shop.example/favicon.ico"]
        );
    }

    fn decoded(png: &[u8]) -> RgbaImage {
        image::load_from_memory_with_format(png, image::ImageFormat::Png)
            .expect("output is a PNG")
            .into_rgba8()
    }

    #[test]
    fn raster_icons_are_scaled_and_centred() -> anyhow::Result<()> {
        let wide = RgbaImage::from_pixel(64, 32, image::Rgba([200, 10, 10, 255]));
        let mut bytes = Vec::new();
        wide.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Png)?;

        let icon = decoded(&render_icon(&bytes)?);
        assert_eq!(icon.dimensions(), (ICON_SIZE, ICON_SIZE));
        assert_eq!(icon.get_pixel(16, 16).0[3], 255, "the middle is painted");
        assert_eq!(
            icon.get_pixel(16, 0).0[3],
            0,
            "the letterbox is transparent"
        );
        Ok(())
    }

    #[test]
    fn ico_files_decode() -> anyhow::Result<()> {
        let square = RgbaImage::from_pixel(16, 16, image::Rgba([10, 120, 10, 255]));
        let mut bytes = Vec::new();
        square.write_to(&mut Cursor::new(&mut bytes), image::ImageFormat::Ico)?;

        let icon = decoded(&render_icon(&bytes)?);
        assert_eq!(icon.dimensions(), (ICON_SIZE, ICON_SIZE));
        assert_eq!(icon.get_pixel(0, 0).0, [10, 120, 10, 255]);
        Ok(())
    }

    #[test]
    fn svg_icons_are_rasterized() -> anyhow::Result<()> {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect width="10" height="10" fill="#0000ff"/></svg>"##;
        let icon = decoded(&render_icon(svg)?);
        assert_eq!(icon.dimensions(), (ICON_SIZE, ICON_SIZE));
        assert_eq!(icon.get_pixel(16, 16).0, [0, 0, 255, 255]);
        Ok(())
    }

    #[test]
    fn html_is_not_an_icon() {
        assert!(render_icon(b"<!doctype html><title>Not found</title>").is_err());
    }
}
