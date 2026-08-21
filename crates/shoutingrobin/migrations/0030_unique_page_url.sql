DELETE FROM pages WHERE id NOT IN (SELECT MIN(id) FROM pages GROUP BY crawl_id, url);
CREATE UNIQUE INDEX IF NOT EXISTS idx_pages_crawl_url_unique ON pages(crawl_id, url);
DROP INDEX IF EXISTS idx_pages_crawl_url;
DROP INDEX IF EXISTS idx_pages_crawl_status;
CREATE INDEX IF NOT EXISTS idx_crawls_root_url ON crawls(root_url);
