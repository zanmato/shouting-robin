# Shouting Robin Test Site

Serve with:

    python3 -m http.server 8000

Then crawl `http://127.0.0.1:8000/` from the app in HTTP mode.

## Pages

| File | Title | H1 | Meta desc | Status | Indexability | Notes |
|---|---|---|---|---|---|---|
| index.html | Test Site Home | Test Site Home | yes | 200 | Indexable | Links to all other pages |
| about.html | About | About | yes | 200 | Indexable | JSON-LD structured data, has H2 |
| missing-meta.html | Missing Meta | (none) | (none) | 200 | Indexable | Missing H1 and meta description |
| duplicate-title.html | Test Site Home | Duplicate title | yes | 200 | Indexable | Same title as index.html |
| noindex.html | Noindex Page | Noindex Page | yes | 200 | Non-Indexable | robots noindex meta tag |
| external-link.html | External Links | External Links | yes | 200 | Indexable | Links to https://example.com |
| broken-link.html | Broken Link | Broken Link | yes | 200 | Indexable | Links to /does-not-exist.html (404 target) |
| images.html | Images | Images | yes | 200 | Indexable | 3 images: 1 with alt, 1 without alt, 1 empty alt |
| redirect.html | Redirect | (none) | (none) | 200 | Indexable | meta refresh redirect to index.html |
| large.html | Large Body | Large Body | yes | 200 | Indexable | 10 paragraphs of lorem ipsum |

## Expected crawl results (HTTP mode)

| Metric | Expected |
|---|---|
| Total pages crawled | 9 (all except /does-not-exist.html which is only linked, not discovered by spider unless followed) |
| Internal links | all hrefs to 127.0.0.1:8000 |
| External links | 1 (https://example.com/) |
| 200 status | 9 |
| 404 status | 0 (spider does not follow broken links by default) |
| Indexable | 8 |
| Non-Indexable | 1 (noindex.html) |
| Pages with missing H1 | 2 (missing-meta.html, redirect.html) |
| Pages with missing meta description | 2 (missing-meta.html, redirect.html) |
| Duplicate titles | 2 pages share "Test Site Home" |
| Images without alt text | 1 (images.html, second img) |
| Structured data pages | 1 (about.html, Organization schema) |

## Shop (`/shop/`)

A small store for layout capture: grouping pages by template and finding
layout changes between two crawls. Nothing outside `/shop/` links to it, so
the crawls above never reach it. Crawl `/shop/` with `/shop/` as the only
include pattern. Like a real store, it has a header, navigation and footer on
every page, and `BreadcrumbList` on every page but the home page.

| Pages | Template | Notes |
|---|---|---|
| `index.html` | Home | `WebSite` markup |
| `info/shipping.html`, `info/returns.html`, `info/about.html` | None shared | Free-form pages built from different blocks, like a CMS |
| `category/chargers.html`, `category/cables.html`, `category/headphones.html` | Category | 9, 9 and 4 products. `cables` has a banner and a modifier class on `main` (`main--with-banner`), which must not split it from the others |
| `product/*.html` | Product | 2 to 5 gallery thumbnails and 4 to 9 spec rows. `charger-65w` has a sale badge and manufacturer content inside its description. `charger-20w` and `cable-2m` have reviews, which load only when scrolled into view. `powerbank-10k` has no `Product` markup, so only its layout puts it with the products |

`shop.js` changes what a real store changes between two loads without its
layout changing: the related products come in another order and the
"viewing now" message gets a new number. Two plain crawls must report no
layout changes.

`variant.css` is empty. The integration test server serves
`variant-changed.css` at that URL after `ServerGuard::set_shop_layout_variant(true)`,
which stands in for a theme change. A second crawl must report each of its
rules:

| Rule | Change |
|---|---|
| `h1` font size | H1 resized and restyled on every page |
| `.price` colour | Price restyled, alone on product pages and as a list on category, product and home pages |
| `.breadcrumbs` hidden | Breadcrumb removed on every page but the home page |
| `.trust-badges` hidden | Trust badges removed, on product pages only |
| `.site-footer` padding and background | Footer restyled on every page |

With `python3 -m http.server`, copy `variant-changed.css` over `variant.css`
to see the change by hand.
