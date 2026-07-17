use super::QuotaError;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindowSnapshot {
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_after_seconds: i64,
    pub window_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountQuota {
    pub created_at: String,
    pub five_hour: Option<QuotaWindowSnapshot>,
    pub weekly: Option<QuotaWindowSnapshot>,
    pub status: String,
}

fn window_at<'a>(value: &'a Value, names: &[&str]) -> Option<&'a Value> {
    names.iter().find_map(|name| {
        [
            format!("/{name}"),
            format!("/rate_limit/{name}"),
            format!("/rate_limits/{name}"),
            format!("/usage/{name}"),
        ]
        .iter()
        .find_map(|pointer| value.pointer(pointer))
        .filter(|window| !window.is_null())
    })
}

fn number_at(value: &Value, names: &[&str]) -> Option<f64> {
    names.iter().find_map(|name| value.get(*name)?.as_f64())
}

fn parse_window(value: &Value) -> Result<QuotaWindowSnapshot, QuotaError> {
    let used_percent = number_at(value, &["used_percent", "usedPercent", "used_percentage"])
        .ok_or(QuotaError::QuotaUnavailable)?
        .clamp(0.0, 100.0);
    let reset_after_seconds = number_at(
        value,
        &["reset_after_seconds", "resetAfterSeconds", "reset_seconds"],
    )
    .map(|seconds| seconds.max(0.0) as i64)
    .or_else(|| {
        number_at(value, &["resets_at", "reset_at", "resetAt"])
            .map(|timestamp| (timestamp as i64 - Utc::now().timestamp()).max(0))
    })
    .ok_or(QuotaError::QuotaUnavailable)?;
    let window_seconds = number_at(value, &["limit_window_seconds", "window_seconds"])
        .map(|seconds| seconds.max(0.0) as i64)
        .or_else(|| {
            number_at(value, &["window_minutes", "limit_window_minutes"])
                .map(|minutes| (minutes.max(0.0) * 60.0) as i64)
        });
    Ok(QuotaWindowSnapshot {
        used_percent,
        remaining_percent: (100.0 - used_percent).max(0.0),
        reset_after_seconds,
        window_seconds,
    })
}

pub fn parse_quota_response(value: &Value) -> Result<AccountQuota, QuotaError> {
    let primary = window_at(value, &["primary_window", "primary"])
        .map(parse_window)
        .transpose()?;
    let secondary = window_at(value, &["secondary_window", "secondary"])
        .map(parse_window)
        .transpose()?;
    if primary.is_none() && secondary.is_none() {
        return Err(QuotaError::QuotaUnavailable);
    }
    let mut five_hour = None;
    let mut weekly = None;
    for (index, window) in [primary, secondary].into_iter().enumerate() {
        let Some(window) = window else { continue };
        let is_weekly = window
            .window_seconds
            .map(|seconds| seconds >= 24 * 60 * 60)
            .unwrap_or(index == 1);
        if is_weekly {
            weekly = Some(window);
        } else {
            five_hour = Some(window);
        }
    }
    Ok(AccountQuota {
        created_at: Utc::now().to_rfc3339(),
        five_hour,
        weekly,
        status: "ok".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_rate_limit_windows() {
        let value: Value = serde_json::from_str(
            r#"{"rate_limit":{"primary_window":{"used_percent":35.5,"reset_after_seconds":5700},"secondary_window":{"used_percent":60,"reset_after_seconds":302400}}}"#,
        )
        .unwrap();
        let quota = parse_quota_response(&value).unwrap();
        assert_eq!(quota.five_hour.as_ref().unwrap().used_percent, 35.5);
        assert_eq!(quota.five_hour.as_ref().unwrap().remaining_percent, 64.5);
        assert_eq!(quota.weekly.unwrap().reset_after_seconds, 302400);
    }

    #[test]
    fn rejects_responses_without_primary_window() {
        let value: Value = serde_json::json!({"secondary_window":{"used_percent":1}});
        assert!(matches!(
            parse_quota_response(&value),
            Err(QuotaError::QuotaUnavailable)
        ));
    }

    #[test]
    fn parses_current_codex_primary_secondary_shape() {
        let reset_at = Utc::now().timestamp() + 3600;
        let value = serde_json::json!({
            "rate_limits": {
                "primary": {"used_percent": 12.0, "resets_at": reset_at, "window_minutes": 300},
                "secondary": {"used_percent": 48.0, "resets_at": reset_at + 86_400, "window_minutes": 10_080}
            }
        });
        let quota = parse_quota_response(&value).unwrap();
        assert_eq!(quota.five_hour.as_ref().unwrap().used_percent, 12.0);
        assert!((3599..=3600).contains(&quota.five_hour.unwrap().reset_after_seconds));
        assert_eq!(quota.weekly.unwrap().remaining_percent, 52.0);
    }

    #[test]
    fn accepts_null_secondary_window_without_estimating_it() {
        let value = serde_json::json!({
            "rate_limit": {
                "primary_window": {"used_percent": 21.0, "reset_after_seconds": 900},
                "secondary_window": null
            }
        });
        let quota = parse_quota_response(&value).unwrap();
        assert_eq!(quota.five_hour.unwrap().used_percent, 21.0);
        assert!(quota.weekly.is_none());
    }

    #[test]
    fn classifies_a_week_long_primary_as_weekly() {
        let value = serde_json::json!({
            "rate_limit": {
                "primary_window": {
                    "used_percent": 41.0,
                    "reset_after_seconds": 580_000,
                    "limit_window_seconds": 604_800
                },
                "secondary_window": null
            }
        });
        let quota = parse_quota_response(&value).unwrap();
        assert!(quota.five_hour.is_none());
        assert_eq!(quota.weekly.unwrap().used_percent, 41.0);
    }
}
