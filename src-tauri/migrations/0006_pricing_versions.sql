CREATE TABLE IF NOT EXISTS pricing_versions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  version TEXT NOT NULL,
  source TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_pricing_versions_updated_at
ON pricing_versions(updated_at DESC);

INSERT INTO schema_migrations(version, applied_at) VALUES (6, datetime('now'));
