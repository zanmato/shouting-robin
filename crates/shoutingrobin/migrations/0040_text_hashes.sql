CREATE TABLE texts_hashed (
    id INTEGER PRIMARY KEY,
    crawl_id INTEGER NOT NULL,
    hash INTEGER,
    text TEXT NOT NULL,
    FOREIGN KEY (crawl_id) REFERENCES crawls(id) ON DELETE CASCADE
);
