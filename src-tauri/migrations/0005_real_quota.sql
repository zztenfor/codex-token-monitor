CREATE TABLE IF NOT EXISTS quota_snapshot (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_at TEXT NOT NULL,
  five_hour_used_percent REAL,
  five_hour_remaining_percent REAL,
  five_hour_reset_seconds INTEGER,
  weekly_used_percent REAL,
  weekly_remaining_percent REAL,
  weekly_reset_seconds INTEGER,
  status TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_quota_snapshot_created_at
ON quota_snapshot(created_at DESC);

INSERT INTO schema_migrations(version, applied_at) VALUES (5, datetime('now'));
