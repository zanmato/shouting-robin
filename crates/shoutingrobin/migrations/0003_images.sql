CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    src TEXT NOT NULL,
    alt TEXT,
    width INTEGER,
    height INTEGER,
    has_alt_attr INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_images_crawl_src ON images(crawl_id, src);

CREATE INDEX IF NOT EXISTS idx_images_crawl_page ON images(crawl_id, page_url)
