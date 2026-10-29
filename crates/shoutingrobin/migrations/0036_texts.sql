DELETE FROM pages WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM links WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM images WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM sitemap_urls WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM a11y_violations WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM structured_data WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM performance WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM ecommerce WHERE crawl_id NOT IN (SELECT id FROM crawls);
DELETE FROM sd_issues WHERE crawl_id NOT IN (SELECT id FROM crawls);

CREATE TABLE IF NOT EXISTS texts (
    id INTEGER PRIMARY KEY,
    crawl_id INTEGER NOT NULL,
    text TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_texts_crawl_text ON texts(crawl_id, text);

INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, url FROM pages ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, src_url FROM links ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, dst_url FROM links ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, anchor FROM links WHERE anchor IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, rel FROM links WHERE rel IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, page_url FROM images ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, src FROM images ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, alt FROM images WHERE alt IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, sitemap_url FROM sitemap_urls ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, page_url FROM sitemap_urls ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, lastmod FROM sitemap_urls WHERE lastmod IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, page_url FROM a11y_violations ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, rule FROM a11y_violations ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, impact FROM a11y_violations ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, target FROM a11y_violations WHERE target IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, html FROM a11y_violations WHERE html IS NOT NULL ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text)
    SELECT crawl_id, page_url FROM structured_data ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, page_url FROM performance ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, page_url FROM ecommerce ORDER BY id;
INSERT OR IGNORE INTO texts (crawl_id, text) SELECT crawl_id, page_url FROM sd_issues ORDER BY id;

DROP INDEX IF EXISTS idx_pages_crawl_url_unique;
DROP INDEX IF EXISTS idx_pages_crawl_url;
ALTER TABLE pages ADD COLUMN url_id INTEGER NOT NULL DEFAULT 0;
UPDATE pages SET url_id =
    (SELECT t.id FROM texts t WHERE t.crawl_id = pages.crawl_id AND t.text = pages.url);
ALTER TABLE pages DROP COLUMN url;
CREATE UNIQUE INDEX idx_pages_crawl_url_unique ON pages(crawl_id, url_id);

CREATE TABLE links_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    src_url_id INTEGER NOT NULL,
    dst_url_id INTEGER NOT NULL,
    anchor_id INTEGER,
    rel_id INTEGER,
    internal INTEGER NOT NULL,
    csr_only INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO links_new (id, crawl_id, src_url_id, dst_url_id, anchor_id, rel_id, internal, csr_only)
    SELECT l.id, l.crawl_id, s.id, d.id, a.id, r.id, l.kind = 'internal', l.csr_only
    FROM links l
    JOIN texts s ON s.crawl_id = l.crawl_id AND s.text = l.src_url
    JOIN texts d ON d.crawl_id = l.crawl_id AND d.text = l.dst_url
    LEFT JOIN texts a ON a.crawl_id = l.crawl_id AND a.text = l.anchor
    LEFT JOIN texts r ON r.crawl_id = l.crawl_id AND r.text = l.rel
    ORDER BY l.id;
DROP TABLE links;
ALTER TABLE links_new RENAME TO links;
CREATE INDEX idx_links_crawl_src ON links(crawl_id, src_url_id);
CREATE INDEX idx_links_crawl_dst ON links(crawl_id, dst_url_id);

CREATE TABLE images_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    src_id INTEGER NOT NULL,
    alt_id INTEGER,
    width INTEGER,
    height INTEGER,
    has_alt_attr INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO images_new (id, crawl_id, page_url_id, src_id, alt_id, width, height, has_alt_attr)
    SELECT i.id, i.crawl_id, p.id, s.id, a.id, i.width, i.height, i.has_alt_attr
    FROM images i
    JOIN texts p ON p.crawl_id = i.crawl_id AND p.text = i.page_url
    JOIN texts s ON s.crawl_id = i.crawl_id AND s.text = i.src
    LEFT JOIN texts a ON a.crawl_id = i.crawl_id AND a.text = i.alt
    ORDER BY i.id;
DROP TABLE images;
ALTER TABLE images_new RENAME TO images;
CREATE INDEX idx_images_crawl_src ON images(crawl_id, src_id);
CREATE INDEX idx_images_crawl_page ON images(crawl_id, page_url_id);

CREATE TABLE sitemap_urls_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    sitemap_url_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    lastmod_id INTEGER,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO sitemap_urls_new (id, crawl_id, sitemap_url_id, page_url_id, lastmod_id)
    SELECT su.id, su.crawl_id, s.id, p.id, m.id
    FROM sitemap_urls su
    JOIN texts s ON s.crawl_id = su.crawl_id AND s.text = su.sitemap_url
    JOIN texts p ON p.crawl_id = su.crawl_id AND p.text = su.page_url
    LEFT JOIN texts m ON m.crawl_id = su.crawl_id AND m.text = su.lastmod
    ORDER BY su.id;
DROP TABLE sitemap_urls;
ALTER TABLE sitemap_urls_new RENAME TO sitemap_urls;
CREATE INDEX idx_sitemap_crawl_page ON sitemap_urls(crawl_id, page_url_id);
CREATE INDEX idx_sitemap_crawl_sitemap ON sitemap_urls(crawl_id, sitemap_url_id);

CREATE TABLE a11y_violations_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    rule_id INTEGER NOT NULL,
    impact_id INTEGER NOT NULL,
    target_id INTEGER,
    html_id INTEGER,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO a11y_violations_new (id, crawl_id, page_url_id, rule_id, impact_id, target_id, html_id)
    SELECT v.id, v.crawl_id, p.id, r.id, i.id, t.id, h.id
    FROM a11y_violations v
    JOIN texts p ON p.crawl_id = v.crawl_id AND p.text = v.page_url
    JOIN texts r ON r.crawl_id = v.crawl_id AND r.text = v.rule
    JOIN texts i ON i.crawl_id = v.crawl_id AND i.text = v.impact
    LEFT JOIN texts t ON t.crawl_id = v.crawl_id AND t.text = v.target
    LEFT JOIN texts h ON h.crawl_id = v.crawl_id AND h.text = v.html
    ORDER BY v.id;
DROP TABLE a11y_violations;
ALTER TABLE a11y_violations_new RENAME TO a11y_violations;
CREATE INDEX idx_a11y_crawl_url ON a11y_violations(crawl_id, page_url_id);

CREATE TABLE structured_data_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    format TEXT NOT NULL,
    type_name TEXT,
    json TEXT NOT NULL,
    errors TEXT,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO structured_data_new (id, crawl_id, page_url_id, format, type_name, json, errors)
    SELECT sd.id, sd.crawl_id, p.id, sd.format, sd.type_name, sd.json, sd.errors
    FROM structured_data sd
    JOIN texts p ON p.crawl_id = sd.crawl_id AND p.text = sd.page_url
    ORDER BY sd.id;
DROP TABLE structured_data;
ALTER TABLE structured_data_new RENAME TO structured_data;
CREATE INDEX idx_sd_crawl_url ON structured_data(crawl_id, page_url_id);

CREATE TABLE performance_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    lcp_ms INTEGER,
    cls REAL,
    ttfb_ms INTEGER,
    transfer_kb REAL,
    fcp_ms INTEGER,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO performance_new (id, crawl_id, page_url_id, lcp_ms, cls, ttfb_ms, transfer_kb, fcp_ms)
    SELECT pf.id, pf.crawl_id, p.id, pf.lcp_ms, pf.cls, pf.ttfb_ms, pf.transfer_kb, pf.fcp_ms
    FROM performance pf
    JOIN texts p ON p.crawl_id = pf.crawl_id AND p.text = pf.page_url
    ORDER BY pf.id;
DROP TABLE performance;
ALTER TABLE performance_new RENAME TO performance;
CREATE INDEX idx_perf_crawl_url ON performance(crawl_id, page_url_id);

CREATE TABLE ecommerce_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
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
    has_review_or_rating BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO ecommerce_new (
    id, crawl_id, page_url_id, page_type, price, currency, availability, sku, gtin, brand,
    has_image, has_description, has_review_or_rating
)
    SELECT e.id, e.crawl_id, p.id, e.page_type, e.price, e.currency, e.availability, e.sku,
           e.gtin, e.brand, e.has_image, e.has_description, e.has_review_or_rating
    FROM ecommerce e
    JOIN texts p ON p.crawl_id = e.crawl_id AND p.text = e.page_url
    ORDER BY e.id;
DROP TABLE ecommerce;
ALTER TABLE ecommerce_new RENAME TO ecommerce;
CREATE INDEX idx_ecom_crawl_url ON ecommerce(crawl_id, page_url_id);

CREATE TABLE sd_issues_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    crawl_id INTEGER NOT NULL,
    page_url_id INTEGER NOT NULL,
    severity TEXT NOT NULL,
    type_name TEXT NOT NULL,
    code TEXT NOT NULL,
    message TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
INSERT INTO sd_issues_new (id, crawl_id, page_url_id, severity, type_name, code, message)
    SELECT si.id, si.crawl_id, p.id, si.severity, si.type_name, si.code, si.message
    FROM sd_issues si
    JOIN texts p ON p.crawl_id = si.crawl_id AND p.text = si.page_url
    ORDER BY si.id;
DROP TABLE sd_issues;
ALTER TABLE sd_issues_new RENAME TO sd_issues;
CREATE INDEX idx_sd_issues_crawl_page ON sd_issues(crawl_id, page_url_id);
