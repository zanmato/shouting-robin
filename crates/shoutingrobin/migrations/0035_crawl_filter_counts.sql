CREATE TABLE IF NOT EXISTS crawl_filter_counts (
    crawl_id INTEGER PRIMARY KEY,
    filter_version INTEGER NOT NULL,
    counts_json TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
