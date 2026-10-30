CREATE TABLE texts (
    id INTEGER PRIMARY KEY,
    hash INTEGER,
    text TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_texts_hash ON texts(hash);
CREATE TABLE pages (
    id INTEGER PRIMARY KEY,
    url_id INTEGER NOT NULL,
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
    sd_errors INTEGER NOT NULL DEFAULT 0,
    sd_warnings INTEGER NOT NULL DEFAULT 0,
    hreflang_tags_json BLOB,
    sd_types_json TEXT,
    simhash INTEGER,
    closest_similarity INTEGER,
    near_duplicate_count INTEGER,
    og_type TEXT,
    a11y_errors INTEGER NOT NULL DEFAULT 0,
    a11y_warnings INTEGER NOT NULL DEFAULT 0,
    headers_json BLOB,
    redirect_url TEXT,
    title_2 TEXT,
    meta_description_2 TEXT,
    h1_2 TEXT,
    h2_2 TEXT,
    title_pixel_width INTEGER,
    meta_description_pixel_width INTEGER,
    redirect_status INTEGER,
    link_score REAL,
    hreflang_issues_json TEXT,
    ssr_word_count INTEGER,
    ssr_h1 TEXT,
    ssr_content_missing INTEGER,
    near_duplicate_urls_json BLOB,
    blocked_by_robots INTEGER,
    is_resource INTEGER NOT NULL DEFAULT 0,
    resource_initiator TEXT,
    is_page INTEGER NOT NULL DEFAULT 1,
    has_mixed_content INTEGER NOT NULL DEFAULT 0,
    hreflang_sources_json TEXT,
    has_body_tag INTEGER,
    h2_non_sequential INTEGER,
    network_fetches INTEGER,
    revalidated_fetches INTEGER,
    transfer_bytes INTEGER
);
CREATE UNIQUE INDEX idx_pages_url ON pages(url_id);
CREATE TABLE links (
    id INTEGER PRIMARY KEY,
    src_url_id INTEGER NOT NULL,
    dst_url_id INTEGER NOT NULL,
    anchor_id INTEGER,
    rel_id INTEGER,
    internal INTEGER NOT NULL,
    csr_only INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_links_dst ON links(dst_url_id);
CREATE TABLE images (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    src_id INTEGER NOT NULL,
    alt_id INTEGER,
    width INTEGER,
    height INTEGER,
    has_alt_attr INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE sitemap_urls (
    id INTEGER PRIMARY KEY,
    sitemap_url_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    lastmod_id INTEGER
);
CREATE TABLE a11y_violations (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    rule_id INTEGER NOT NULL,
    impact_id INTEGER NOT NULL,
    target_id INTEGER,
    html_id INTEGER
);
CREATE TABLE structured_data (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    format TEXT NOT NULL,
    type_name TEXT,
    json BLOB NOT NULL,
    errors TEXT
);
CREATE INDEX idx_sd_page ON structured_data(page_url_id);
CREATE TABLE performance (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    lcp_ms INTEGER,
    cls REAL,
    ttfb_ms INTEGER,
    transfer_kb REAL,
    fcp_ms INTEGER
);
CREATE TABLE ecommerce (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    page_type TEXT,
    price REAL,
    currency TEXT,
    availability TEXT,
    sku TEXT,
    gtin TEXT,
    brand TEXT,
    has_image BOOLEAN NOT NULL DEFAULT 0,
    has_description BOOLEAN NOT NULL DEFAULT 0,
    has_review_or_rating BOOLEAN NOT NULL DEFAULT 0
);
CREATE TABLE sd_issues (
    id INTEGER PRIMARY KEY,
    page_url_id INTEGER NOT NULL,
    severity TEXT NOT NULL,
    type_name TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL
);
CREATE TABLE page_layouts (
    url TEXT PRIMARY KEY,
    layout BLOB NOT NULL,
    template_id INTEGER,
    assigned_by TEXT
);
CREATE TABLE layout_keys (
    id INTEGER PRIMARY KEY,
    key TEXT NOT NULL
);
CREATE TABLE layout_styles (
    id INTEGER PRIMARY KEY,
    style_json TEXT NOT NULL
);
CREATE TABLE layout_templates (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    type_key TEXT,
    page_count INTEGER NOT NULL
);
CREATE TABLE crawl_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    finished INTEGER NOT NULL DEFAULT 0
);
INSERT INTO crawl_state (id) VALUES (1);
