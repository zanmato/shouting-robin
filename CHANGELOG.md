# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-09-10

### Features

- Redesigned the interface, with a modern app shell
- Split the details panel into tabs, each its own scrollable region, with badged counts
- Aggregated the Links, Images and Hreflang tabs to one row per URL or source
- Added image size and status to the Images tab
- Recorded every hop of a redirect chain
- Recorded the URLs a crawl knows about but never fetched
- Reported how much of a page's link graph only exists after rendering
- Added noindex directives, H2 outline rules, missing body element, canonical and ecommerce rules as overview rules
- Read hreflang from Link headers and sitemaps
- Captured sitemap lastmod and reported unhealthy sitemap entries
- Validated rich result URLs
- Exported the crawl as a PDF report
- Set the report and the SERP preview in Google Sans
- Added bulk CSV export
- Batched live crawl updates during a crawl
- Moved crawl controls into the gear panel, with one fetch per page and paced auxiliary requests

### Fixes

- Corrected structured data, hreflang, title, link and content extraction
- Computed crawl depth from a walk of the link graph, and recomputed inlink counts from the full graph after a crawl
- Refreshing the UI from the database when a crawl finishes
- Found the redirects spider hid behind a 200, and treated an unfollowed 3xx as a redirect everywhere
- Decoded HTML entities in hrefs before queueing them, and inflated gzipped sitemaps before parsing
- Modelled canonicalisation as a non indexable state, and counted a missing canonical
- Fixed missing headers, and discarded response headers Chrome reported from another request
- Measured SERP pixel width with real Arial advance widths, titles in Google Sans at 22px and descriptions in Arial at 14px
- Counted characters, not bytes, in the length columns and filters
- Counted overview rules against the population their percentage divides by, image rules in images, and instance derived rules in pages
- Gated the pixel width and self reference rules on content eligibility
- Raised the low content threshold to 200 words
- Read a link's aria-label, or its image alt, as anchor text when it has none
- Exempted subresources, XHR calls and bundled assets from the URL quality rules
- Only reported a missing hreflang return tag when the target was crawled
- Status checked images Chrome could not size, without re-fetching API calls
- Replaced an inline image's payload with a hash and its size
- Removed cookie alerts from the words diff
- Verified updates against checksums and signatures, bundled axe-core, and measured vitals in visible windows
- Fixed table sorting for text, trimmed quotes from HTTP headers, removed content type from redirects and included links from the initial page

### Performance

- Wrote each page in one transaction, and loaded a crawl in one query
- Built a crawl's aggregates off the foreground thread, with a loading state
- Built the overview's issue entries once, not once per cell
- Virtualised the link lists in the details panel

## [0.1.0] - 2026-06-03

### Features

- Initial version

[0.2.0]: https://github.com/zanmato/shouting-robin/releases/tag/v0.2.0
[0.1.0]: https://github.com/zanmato/shouting-robin/releases/tag/v0.1.0
