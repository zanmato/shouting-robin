CREATE TABLE IF NOT EXISTS page_layouts (
    crawl_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    layout BLOB NOT NULL,
    template_id INTEGER,
    assigned_by TEXT,
    PRIMARY KEY (crawl_id, url),
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS layout_keys (
    crawl_id INTEGER NOT NULL,
    id INTEGER NOT NULL,
    key TEXT NOT NULL,
    PRIMARY KEY (crawl_id, id),
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS layout_styles (
    crawl_id INTEGER NOT NULL,
    id INTEGER NOT NULL,
    style_json TEXT NOT NULL,
    PRIMARY KEY (crawl_id, id),
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS layout_templates (
    crawl_id INTEGER NOT NULL,
    id INTEGER NOT NULL,
    name TEXT NOT NULL,
    type_key TEXT,
    page_count INTEGER NOT NULL,
    PRIMARY KEY (crawl_id, id),
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
