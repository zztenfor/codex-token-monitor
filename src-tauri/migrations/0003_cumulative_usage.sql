ALTER TABLE sync_sources ADD COLUMN cumulative_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sync_sources ADD COLUMN cumulative_output_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sync_sources ADD COLUMN cumulative_reasoning_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sync_sources ADD COLUMN cumulative_cached_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sync_sources ADD COLUMN cumulative_total_tokens INTEGER NOT NULL DEFAULT 0;

INSERT INTO schema_migrations(version, applied_at) VALUES (3, datetime('now'));
