CREATE TABLE IF NOT EXISTS site_settings (
    host TEXT PRIMARY KEY,
    settings_json TEXT NOT NULL,
    saved_at INTEGER NOT NULL
)
