CREATE TABLE IF NOT EXISTS site_icons (
    host TEXT PRIMARY KEY,
    icon_png BLOB,
    fetched_at INTEGER NOT NULL
)
