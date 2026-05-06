CREATE TABLE IF NOT EXISTS sd_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    severity TEXT NOT NULL,
    type_name TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sd_issues_crawl_page ON sd_issues(crawl_id, page_url);
