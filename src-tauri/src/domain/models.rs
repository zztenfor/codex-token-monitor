use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTotals {
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

impl TokenTotals {
    pub fn from_inclusive_counts(
        input_tokens: i64,
        cached_input_tokens: i64,
        output_tokens: i64,
        reasoning_output_tokens: i64,
    ) -> Self {
        let cached_input_tokens = cached_input_tokens.min(input_tokens).max(0);
        let reasoning_output_tokens = reasoning_output_tokens.min(output_tokens).max(0);
        let input_tokens = input_tokens.saturating_sub(cached_input_tokens);
        let output_tokens = output_tokens.saturating_sub(reasoning_output_tokens);
        Self::from_categories(
            input_tokens,
            cached_input_tokens,
            output_tokens,
            reasoning_output_tokens,
        )
    }

    pub fn from_categories(
        input_tokens: i64,
        cached_input_tokens: i64,
        output_tokens: i64,
        reasoning_output_tokens: i64,
    ) -> Self {
        let mut totals = Self {
            input_tokens: input_tokens.max(0),
            cached_input_tokens: cached_input_tokens.max(0),
            output_tokens: output_tokens.max(0),
            reasoning_output_tokens: reasoning_output_tokens.max(0),
            total_tokens: 0,
        };
        totals.recalculate();
        totals
    }

    pub fn recalculate(&mut self) {
        self.total_tokens = self
            .input_tokens
            .saturating_add(self.cached_input_tokens)
            .saturating_add(self.output_tokens)
            .saturating_add(self.reasoning_output_tokens);
    }
}

#[derive(Debug, Clone)]
pub struct UsageEvent {
    pub event_id: String,
    pub session_id: String,
    pub timestamp: String,
    pub model: String,
    pub project_key: Option<String>,
    pub project_name: Option<String>,
    pub context_window: Option<i64>,
    pub totals: TokenTotals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentSession {
    pub session_id: String,
    pub model: String,
    pub context_window: Option<i64>,
    pub usage_percent: Option<f64>,
    pub context_tokens: i64,
    pub updated_at: String,
    #[serde(flatten)]
    pub totals: TokenTotals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagePoint {
    pub bucket: String,
    pub label: String,
    #[serde(flatten)]
    pub totals: TokenTotals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUsage {
    pub project_key: String,
    pub display_name: String,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    #[serde(flatten)]
    pub totals: TokenTotals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardData {
    pub today: TokenTotals,
    pub current_session: Option<CurrentSession>,
    pub chart: Vec<UsagePoint>,
    pub projects: Vec<ProjectUsage>,
    pub model_usage: Vec<ModelUsage>,
    pub monthly_total_tokens: i64,
    pub projected_monthly_tokens: i64,
    pub daily_average_tokens: i64,
    pub risk_level: String,
    pub five_hour_quota: QuotaWindow,
    pub weekly_quota: QuotaWindow,
    pub real_account_quota: RealAccountQuota,
    pub sync_paused: bool,
    pub last_synced_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub consumed_tokens: i64,
    pub limit_tokens: i64,
    pub remaining_tokens: i64,
    pub usage_percent: Option<f64>,
    pub reset_at: Option<String>,
    pub reset_in_seconds: Option<i64>,
    pub projected_tokens: i64,
    pub forecast_status: String,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealQuotaWindow {
    pub used_percent: f64,
    pub remaining_percent: f64,
    pub reset_after_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RealAccountQuota {
    pub created_at: Option<String>,
    pub five_hour: Option<RealQuotaWindow>,
    pub weekly: Option<RealQuotaWindow>,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub codex_path: String,
    pub refresh_interval_seconds: u64,
    pub theme: String,
    pub language: String,
    #[serde(alias = "floating_opacity")]
    pub floating_opacity: f64,
    #[serde(alias = "floating_always_on_top")]
    pub floating_always_on_top: bool,
    #[serde(alias = "floating_click_through")]
    pub floating_click_through: bool,
    #[serde(alias = "floating_mode")]
    pub floating_mode: String,
    pub daily_budget: i64,
    pub monthly_budget: i64,
    pub five_hour_limit: i64,
    pub weekly_limit: i64,
    pub alert80: bool,
    pub alert90: bool,
    pub alert95: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        #[derive(Deserialize)]
        struct QuotaDefaults {
            five_hour_limit: i64,
            weekly_limit: i64,
        }
        let quota =
            serde_json::from_str::<QuotaDefaults>(include_str!("../../config/default.json"))
                .unwrap_or(QuotaDefaults {
                    five_hour_limit: 0,
                    weekly_limit: 0,
                });
        Self {
            codex_path: String::new(),
            refresh_interval_seconds: 5,
            theme: "system".into(),
            language: crate::i18n::system_language(),
            floating_opacity: 0.85,
            floating_always_on_top: true,
            floating_click_through: false,
            floating_mode: "auto".into(),
            daily_budget: 0,
            monthly_budget: 0,
            five_hour_limit: quota.five_hour_limit,
            weekly_limit: quota.weekly_limit,
            alert80: true,
            alert90: true,
            alert95: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detailed_categories_are_disjoint_and_sum_to_total() {
        let totals = TokenTotals::from_inclusive_counts(120, 50, 80, 40);
        assert_eq!(totals.input_tokens, 70);
        assert_eq!(totals.cached_input_tokens, 50);
        assert_eq!(totals.output_tokens, 40);
        assert_eq!(totals.reasoning_output_tokens, 40);
        assert_eq!(totals.total_tokens, 200);
    }

    #[test]
    fn old_settings_json_receives_quota_defaults() {
        let settings: AppSettings = serde_json::from_str(
            r#"{"codexPath":"","refreshIntervalSeconds":5,"theme":"system","dailyBudget":0,"monthlyBudget":0,"alert80":true,"alert90":true,"alert95":true}"#,
        )
        .unwrap();
        assert_eq!(settings.five_hour_limit, 0);
        assert_eq!(settings.weekly_limit, 0);
        assert!(settings.language == crate::i18n::ZH_CN || settings.language == crate::i18n::EN_US);
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub files_scanned: usize,
    pub events_inserted: usize,
    pub events_skipped: usize,
    pub parse_errors: usize,
    pub synced_at: String,
}
