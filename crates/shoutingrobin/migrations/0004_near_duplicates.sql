ALTER TABLE pages ADD COLUMN simhash INTEGER;

ALTER TABLE pages ADD COLUMN closest_similarity INTEGER;

ALTER TABLE pages ADD COLUMN near_duplicate_count INTEGER;
