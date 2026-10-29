ALTER TABLE crawls ADD COLUMN page_count INTEGER;
ALTER TABLE crawls ADD COLUMN document_count INTEGER;
ALTER TABLE crawls ADD COLUMN unfetched_count INTEGER;
UPDATE crawls SET
    page_count = (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = crawls.id),
    document_count = (SELECT COUNT(*) FROM pages p WHERE p.crawl_id = crawls.id AND p.is_page = 1),
    unfetched_count = (SELECT COUNT(*) FROM pages p
        WHERE p.crawl_id = crawls.id AND p.status IS NULL AND p.redirect_url IS NULL)
WHERE finished_at IS NOT NULL;
