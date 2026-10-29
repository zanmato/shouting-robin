# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Features

- Searched and filtered a tab's rows from a bar under its filters, opened
  with Ctrl+F or the Filter button. It takes column conditions such as
  Code = 200 joined by Match All or Match Any, and a cell's context menu
  filters by its value or excludes it
- Showed the Shouting Robin bird shouting while a crawl loads, in place of
  the spinner

### Performance

- Kept large crawls small in memory and quick to open. A crawl of 409k rows
  compared with one of 269k now holds 1.2 GB in place of 3.5 GB and reads
  in 7.5 s in place of 28.5 s
- Stored a finished crawl's tab and filter counts, so opening it again skips
  counting (6.9 s to 0.3 s on that crawl)
- Stored each crawl's page counts when it finishes, so the crawl history
  lists quickly
- Stored each URL and repeated text once per crawl, which shrank a 2.4 GB
  database to 750 MB. The first start after updating converts the database
  once, which takes about a minute for one that size

### Fixes

- Counted sitemap orphans and URLs blocked by robots.txt in the crawl
  history
- Removed the Total Types, JSON-LD and Microdata columns from the Structured
  Data tab, which were always empty, and the empty badges of cells with no
  value
- Left the page sections out of the details panel for assets, redirects and
  other files, which have no page to describe
- Gave Filter by This Value and Exclude This Value icons in a cell's context
  menu

## [0.2.3] - 2026-10-28

### Features

- Flagged page titles, meta descriptions and H1s that contain HTML
  entities such as `&amp;`
- Listed the pages that reference an image or other resource, in a
  Referenced by tab
- Notified you through the system when a crawl finishes while the app is
  in the background
- Shortened large counts in tab badges, filters and the crawl history
  (1.2k, 39k, 1.2M), with the exact count on hover
- Showed each step while a crawl loads, with a progress bar
- Attributed sitemap URLs to the child sitemap that lists them, in place
  of the sitemap index
- Identified crawls as Shouting Robin in the user agent
- Played a motion demo on the website in place of the Overview screenshot

### Performance

- Counted tabs and filters in the background, so large crawls stay
  responsive while pages stream in
- Cached the Changes tab entries, so large comparisons no longer freeze
  the app
- Read a page's backlinks when it is selected in place of loading every
  backlink with the crawl, so crawls open faster

### Fixes

- Read robots.txt from the site root when a crawl starts in a folder
- Kept a running crawl from updating a previous crawl you switched to,
  and from comparing against the wrong baseline
- Kept the advanced settings closed when a crawl stops
- Showed backlinks in the details panel while a crawl is still running

## [0.2.2] - 2026-10-23

### Features

- Settings, Appearance has a VS Code Themes group that searches Open VSX for
  VS Code color themes, previews them, lets you try one across the app
  before installing, and installs them converted to Shouting Robin's theme
  format
- Captured each page's layout in Chrome crawls, with an option to scroll
  the page first so lazy content renders
- Compared layouts against the previous crawl in the Changes tab, telling
  layout changes apart from content changes, with a wireframe view of
  both versions side by side
- Grouped pages into templates by structured data and layout, in a new
  Layout tab, and reported product layouts without product markup in the
  Overview
- Showed each site's favicon in the crawls sidebar
- Added the layout comparison to the screenshots and the website tour
- Wider copy and text selection support

### Fixes

- Waited for every page to be processed before closing Chrome at the end
  of a crawl, so the last pages keep their web vitals and accessibility
  results

## [0.2.1] - 2026-10-09

### Performance

- Moved rendering to gpui-fast, on GPUI Kit 0.7.1

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
