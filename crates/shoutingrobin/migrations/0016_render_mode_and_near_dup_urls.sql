ALTER TABLE crawls ADD COLUMN render_mode TEXT NOT NULL DEFAULT 'http';

ALTER TABLE pages ADD COLUMN near_duplicate_urls_json TEXT;
