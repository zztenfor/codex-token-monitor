ALTER TABLE sessions ADD COLUMN context_tokens INTEGER NOT NULL DEFAULT 0;
INSERT INTO schema_migrations(version, applied_at) VALUES (2, datetime('now'));
