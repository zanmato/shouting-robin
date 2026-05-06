CREATE TABLE IF NOT EXISTS crawls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_url TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    finished_at INTEGER,
    config_json TEXT
);

CREATE TABLE IF NOT EXISTS pages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    status INTEGER,
    content_type TEXT,
    size_bytes INTEGER,
    response_time_ms INTEGER,
    depth INTEGER,
    title TEXT,
    meta_description TEXT,
    h1 TEXT,
    h2 TEXT,
    canonical TEXT,
    robots TEXT,
    indexability TEXT,
    word_count INTEGER,
    hash TEXT,
    crawled_at INTEGER NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_pages_crawl_url ON pages(crawl_id, url);

CREATE INDEX IF NOT EXISTS idx_pages_crawl_status ON pages(crawl_id, status);

CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    src_url TEXT NOT NULL,
    dst_url TEXT NOT NULL,
    anchor TEXT,
    rel TEXT,
    kind TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_links_crawl_src ON links(crawl_id, src_url);

CREATE INDEX IF NOT EXISTS idx_links_crawl_dst ON links(crawl_id, dst_url);

CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    json_detail TEXT,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_issues_crawl_category ON issues(crawl_id, category);

CREATE TABLE IF NOT EXISTS structured_data (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    format TEXT NOT NULL,
    type_name TEXT,
    json TEXT NOT NULL,
    errors TEXT,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sd_crawl_url ON structured_data(crawl_id, page_url);

CREATE TABLE IF NOT EXISTS a11y_violations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    rule TEXT NOT NULL,
    impact TEXT NOT NULL,
    target TEXT,
    html TEXT,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_a11y_crawl_url ON a11y_violations(crawl_id, page_url);

CREATE TABLE IF NOT EXISTS performance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    lcp_ms INTEGER,
    cls REAL,
    inp_ms INTEGER,
    ttfb_ms INTEGER,
    transfer_kb REAL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_perf_crawl_url ON performance(crawl_id, page_url);

CREATE TABLE IF NOT EXISTS ecommerce (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url TEXT NOT NULL,
    page_type TEXT,
    price REAL,
    currency TEXT,
    availability TEXT,
    sku TEXT,
    gtin TEXT,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_ecom_crawl_url ON ecommerce(crawl_id, page_url)
