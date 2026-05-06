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
