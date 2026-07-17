ALTER TABLE sessions ADD COLUMN cached_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE sessions ADD COLUMN reasoning_output_tokens INTEGER NOT NULL DEFAULT 0;

ALTER TABLE usage_events ADD COLUMN cached_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE usage_events ADD COLUMN reasoning_output_tokens INTEGER NOT NULL DEFAULT 0;

ALTER TABLE daily_usage ADD COLUMN cached_input_tokens INTEGER NOT NULL DEFAULT 0;
ALTER TABLE daily_usage ADD COLUMN reasoning_output_tokens INTEGER NOT NULL DEFAULT 0;

UPDATE sessions
SET cached_input_tokens = cached_tokens,
    reasoning_output_tokens = reasoning_tokens,
    input_tokens = MAX(input_tokens - cached_tokens, 0),
    output_tokens = MAX(output_tokens - reasoning_tokens, 0);
UPDATE sessions
SET total_tokens = input_tokens + cached_input_tokens + output_tokens + reasoning_output_tokens;

UPDATE usage_events
SET cached_input_tokens = cached_tokens,
    reasoning_output_tokens = reasoning_tokens,
    input_tokens = MAX(input_tokens - cached_tokens, 0),
    output_tokens = MAX(output_tokens - reasoning_tokens, 0);
UPDATE usage_events
SET total_tokens = input_tokens + cached_input_tokens + output_tokens + reasoning_output_tokens;

UPDATE daily_usage
SET cached_input_tokens = cached_tokens,
    reasoning_output_tokens = reasoning_tokens,
    input_tokens = MAX(input_tokens - cached_tokens, 0),
    output_tokens = MAX(output_tokens - reasoning_tokens, 0);
UPDATE daily_usage
SET total_tokens = input_tokens + cached_input_tokens + output_tokens + reasoning_output_tokens;

UPDATE sync_sources
SET cumulative_input_tokens = MAX(cumulative_input_tokens - cumulative_cached_tokens, 0),
    cumulative_output_tokens = MAX(cumulative_output_tokens - cumulative_reasoning_tokens, 0);
UPDATE sync_sources
SET cumulative_total_tokens = cumulative_input_tokens + cumulative_cached_tokens
    + cumulative_output_tokens + cumulative_reasoning_tokens;

INSERT INTO schema_migrations(version, applied_at) VALUES (4, datetime('now'));
