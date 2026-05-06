CREATE TABLE IF NOT EXISTS sitemap_urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    sitemap_url TEXT NOT NULL,
    page_url TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sitemap_crawl_page ON sitemap_urls(crawl_id, page_url);

CREATE INDEX IF NOT EXISTS idx_sitemap_crawl_sitemap ON sitemap_urls(crawl_id, sitemap_url)
