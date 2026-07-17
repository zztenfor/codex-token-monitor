use crate::domain::models::{TokenTotals, UsageEvent};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SessionContext {
    pub session_id: String,
    pub model: String,
    pub timestamp: String,
    pub project_key: Option<String>,
    pub project_name: Option<String>,
    pub context_window: Option<i64>,
    pub cumulative_usage: TokenTotals,
}

fn text_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn int_at(value: &Value, path: &[&str]) -> i64 {
    let mut current = value;
    for key in path {
        match current.get(*key) {
            Some(next) => current = next,
            None => return 0,
        }
    }
    current.as_i64().unwrap_or(0).max(0)
}

fn first_int(value: &Value, paths: &[&[&str]]) -> i64 {
    paths
        .iter()
        .map(|path| int_at(value, path))
        .find(|value| *value > 0)
        .unwrap_or(0)
}

fn token_totals(value: &Value) -> TokenTotals {
    let inclusive_input = first_int(value, &[&["input_tokens"], &["prompt_tokens"], &["input"]]);
    let cached_input = first_int(
        value,
        &[
            &["cached_input_tokens"],
            &["cached_tokens"],
            &["input_tokens_details", "cached_tokens"],
            &["prompt_tokens_details", "cached_tokens"],
        ],
    );
    let inclusive_output = first_int(
        value,
        &[&["output_tokens"], &["completion_tokens"], &["output"]],
    );
    let reasoning_output = first_int(
        value,
        &[
            &["reasoning_output_tokens"],
            &["reasoning_tokens"],
            &["output_tokens_details", "reasoning_tokens"],
            &["completion_tokens_details", "reasoning_tokens"],
        ],
    );
    let explicit_input = first_int(
        value,
        &[&["uncached_input_tokens"], &["non_cached_input_tokens"]],
    );
    let explicit_output = first_int(
        value,
        &[&["non_reasoning_output_tokens"], &["visible_output_tokens"]],
    );
    if explicit_input == 0 && explicit_output == 0 {
        return TokenTotals::from_inclusive_counts(
            inclusive_input,
            cached_input,
            inclusive_output,
            reasoning_output,
        );
    }
    TokenTotals::from_categories(
        if explicit_input > 0 {
            explicit_input
        } else {
            inclusive_input.saturating_sub(cached_input.min(inclusive_input))
        },
        cached_input.min(inclusive_input.max(cached_input)),
        if explicit_output > 0 {
            explicit_output
        } else {
            inclusive_output.saturating_sub(reasoning_output.min(inclusive_output))
        },
        reasoning_output.min(inclusive_output.max(reasoning_output)),
    )
}

fn usage_delta(current: &TokenTotals, previous: &TokenTotals) -> TokenTotals {
    TokenTotals {
        input_tokens: current.input_tokens.saturating_sub(previous.input_tokens),
        cached_input_tokens: current
            .cached_input_tokens
            .saturating_sub(previous.cached_input_tokens),
        output_tokens: current.output_tokens.saturating_sub(previous.output_tokens),
        reasoning_output_tokens: current
            .reasoning_output_tokens
            .saturating_sub(previous.reasoning_output_tokens),
        total_tokens: current.total_tokens.saturating_sub(previous.total_tokens),
    }
}

fn project_identity(cwd: &str, salt: &str) -> (String, String) {
    let normalized = cwd.replace('/', "\\").to_lowercase();
    let mut digest = Sha256::new();
    digest.update(salt.as_bytes());
    digest.update(normalized.as_bytes());
    let key = hex::encode(digest.finalize());
    let name = Path::new(cwd)
        .file_name()
        .and_then(|x| x.to_str())
        .filter(|x| !x.is_empty())
        .unwrap_or("Unknown project")
        .to_string();
    (key, name)
}

pub fn parse_line(
    line: &str,
    _source_key: &str,
    _byte_offset: u64,
    context: &mut SessionContext,
    salt: &str,
) -> Result<Option<UsageEvent>, serde_json::Error> {
    // Content events are never deserialized. Only known metadata-bearing rows
    // enter the JSON parser, keeping prompts and responses outside our model.
    if !line.contains("\"token_count\"")
        && !line.contains("\"session_meta\"")
        && !line.contains("\"turn_context\"")
    {
        return Ok(None);
    }
    let value: Value = serde_json::from_str(line)?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let payload = value.get("payload").unwrap_or(&Value::Null);
    let timestamp = value
        .get("timestamp")
        .and_then(Value::as_str)
        .or_else(|| payload.get("timestamp").and_then(Value::as_str))
        .unwrap_or_default();

    if event_type == "session_meta" {
        context.session_id = payload
            .get("id")
            .and_then(Value::as_str)
            .or_else(|| payload.get("session_id").and_then(Value::as_str))
            .unwrap_or_default()
            .to_string();
        context.timestamp = timestamp.to_string();
        context.model = payload
            .get("model")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        context.context_window = payload.get("context_window").and_then(Value::as_i64);
        if let Some(cwd) = payload.get("cwd").and_then(Value::as_str) {
            let (key, name) = project_identity(cwd, salt);
            context.project_key = Some(key);
            context.project_name = Some(name);
        }
        return Ok(None);
    }
    if event_type == "turn_context" {
        if let Some(model) = payload.get("model").and_then(Value::as_str) {
            context.model = model.to_string();
        }
        if let Some(window) = payload.get("model_context_window").and_then(Value::as_i64) {
            context.context_window = Some(window);
        }
        return Ok(None);
    }
    if event_type != "event_msg"
        || payload.get("type").and_then(Value::as_str) != Some("token_count")
    {
        return Ok(None);
    }

    let last_usage = payload
        .pointer("/info/last_token_usage")
        .or_else(|| payload.get("usage"))
        .unwrap_or(&Value::Null);
    let last_totals = token_totals(last_usage);
    let totals = if let Some(cumulative) = payload.pointer("/info/total_token_usage") {
        let current = token_totals(cumulative);
        if current.total_tokens == 0 {
            context.cumulative_usage = current;
            return Ok(None);
        }
        let delta = if context.cumulative_usage.total_tokens == 0
            || current.total_tokens >= context.cumulative_usage.total_tokens
        {
            usage_delta(&current, &context.cumulative_usage)
        } else {
            last_totals.clone()
        };
        context.cumulative_usage = current;
        delta
    } else {
        last_totals
    };
    if totals.total_tokens == 0 && totals.input_tokens == 0 && totals.output_tokens == 0 {
        return Ok(None);
    }
    let session_id = text_at(&value, &["session_id"])
        .unwrap_or(&context.session_id)
        .to_string();
    if session_id.is_empty() {
        return Ok(None);
    }
    let context_window = payload
        .pointer("/info/model_context_window")
        .and_then(Value::as_i64)
        .or(context.context_window);
    let mut digest = Sha256::new();
    digest.update(session_id.as_bytes());
    digest.update(timestamp.as_bytes());
    digest.update(totals.total_tokens.to_le_bytes());
    digest.update(totals.input_tokens.to_le_bytes());
    let event_id = hex::encode(digest.finalize());
    Ok(Some(UsageEvent {
        event_id,
        session_id,
        timestamp: if timestamp.is_empty() {
            context.timestamp.clone()
        } else {
            timestamp.to_string()
        },
        model: context.model.clone(),
        project_key: context.project_key.clone(),
        project_name: context.project_name.clone(),
        context_window,
        totals,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_current_codex_token_event_without_content() {
        let mut ctx = SessionContext::default();
        parse_line(r#"{"timestamp":"2026-07-16T10:00:00Z","type":"session_meta","payload":{"id":"s1","cwd":"C:\\work\\alpha","model":"gpt-codex","context_window":100000}}"#, "a", 0, &mut ctx, "salt").unwrap();
        let event = parse_line(r#"{"timestamp":"2026-07-16T10:01:00Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":20,"output_tokens":40,"reasoning_output_tokens":10,"total_tokens":140},"model_context_window":100000}}}"#, "a", 100, &mut ctx, "salt").unwrap().unwrap();
        assert_eq!(event.session_id, "s1");
        assert_eq!(event.totals.input_tokens, 80);
        assert_eq!(event.totals.cached_input_tokens, 20);
        assert_eq!(event.totals.output_tokens, 30);
        assert_eq!(event.totals.reasoning_output_tokens, 10);
        assert_eq!(event.totals.total_tokens, 140);
        assert_eq!(event.project_name.as_deref(), Some("alpha"));
    }

    #[test]
    fn maps_openai_compatible_detail_fields() {
        let value: Value = serde_json::from_str(
            r#"{"prompt_tokens":100,"completion_tokens":50,"prompt_tokens_details":{"cached_tokens":25},"completion_tokens_details":{"reasoning_tokens":15},"total_tokens":150}"#,
        )
        .unwrap();
        let totals = token_totals(&value);
        assert_eq!(totals.input_tokens, 75);
        assert_eq!(totals.cached_input_tokens, 25);
        assert_eq!(totals.output_tokens, 35);
        assert_eq!(totals.reasoning_output_tokens, 15);
        assert_eq!(totals.total_tokens, 150);
    }

    #[test]
    fn skips_zero_cumulative_initialization_and_uses_deltas() {
        let mut ctx = SessionContext::default();
        parse_line(r#"{"timestamp":"2026-07-16T10:00:00Z","type":"session_meta","payload":{"id":"s1","model":"gpt-codex"}}"#, "a", 0, &mut ctx, "salt").unwrap();
        let initial = parse_line(r#"{"timestamp":"2026-07-16T10:00:01Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":7000,"output_tokens":552,"total_tokens":7552},"total_token_usage":{"input_tokens":0,"output_tokens":0,"total_tokens":0}}}}"#, "a", 10, &mut ctx, "salt").unwrap();
        assert!(initial.is_none());

        let first = parse_line(r#"{"timestamp":"2026-07-16T10:00:02Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"output_tokens":40,"total_tokens":140},"total_token_usage":{"input_tokens":100,"output_tokens":40,"total_tokens":140}}}}"#, "a", 20, &mut ctx, "salt").unwrap().unwrap();
        let second = parse_line(r#"{"timestamp":"2026-07-16T10:00:03Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":150,"output_tokens":50,"total_tokens":200},"total_token_usage":{"input_tokens":250,"output_tokens":90,"total_tokens":340}}}}"#, "a", 30, &mut ctx, "salt").unwrap().unwrap();

        assert_eq!(first.totals.total_tokens, 140);
        assert_eq!(second.totals.total_tokens, 200);
        assert_eq!(second.totals.input_tokens, 150);
    }
}
