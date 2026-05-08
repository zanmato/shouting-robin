ALTER TABLE pages ADD COLUMN link_score REAL;

CREATE TABLE IF NOT EXISTS redirect_hops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    hop_index INTEGER NOT NULL,
    url TEXT NOT NULL,
    status INTEGER NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_redirect_hops_crawl_page ON redirect_hops(crawl_id, page_url);
