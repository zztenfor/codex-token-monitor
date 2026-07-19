use crate::{
    domain::models::{
        AppSettings, CurrentSession, DashboardData, ModelUsage, ProjectUsage, QuotaWindow,
        RealAccountQuota, RealQuotaWindow, TokenTotals, UsageEvent, UsagePoint,
    },
    error::{AppError, AppResult},
    infrastructure::codex::parser::SessionContext,
    services::quota::AccountQuota,
};
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Timelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::{fs, path::Path};
use uuid::Uuid;

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.execute_batch(include_str!("../../../migrations/0001_initial.sql"))?;
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if version < 2 {
            connection
                .execute_batch(include_str!("../../../migrations/0002_session_context.sql"))?;
        }
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if version < 3 {
            connection.execute_batch(include_str!(
                "../../../migrations/0003_cumulative_usage.sql"
            ))?;
        }
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if version < 4 {
            connection.execute_batch(include_str!(
                "../../../migrations/0004_detailed_accounting.sql"
            ))?;
        }
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if version < 5 {
            connection.execute_batch(include_str!("../../../migrations/0005_real_quota.sql"))?;
        }
        let version: i64 = connection.query_row(
            "SELECT COALESCE(MAX(version),0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?;
        if version < 6 {
            connection.execute_batch(include_str!("../../../migrations/0006_pricing_versions.sql"))?;
        }
        let db = Self { connection };
        db.ensure_defaults()?;
        Ok(db)
    }

    fn ensure_defaults(&self) -> AppResult<()> {
        if self.get_value("privacy_salt")?.is_none() {
            self.set_value("privacy_salt", &Uuid::new_v4().to_string())?;
        }
        if self.get_value("app_settings")?.is_none() {
            self.save_settings(&AppSettings::default())?;
        }
        Ok(())
    }

    fn get_value(&self, key: &str) -> AppResult<Option<String>> {
        Ok(self
            .connection
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |row| {
                row.get(0)
            })
            .optional()?)
    }

    fn set_value(&self, key: &str, value: &str) -> AppResult<()> {
        self.connection.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value])?;
        Ok(())
    }

    pub fn privacy_salt(&self) -> AppResult<String> {
        Ok(self
            .get_value("privacy_salt")?
            .unwrap_or_else(|| "local".into()))
    }
    pub fn settings(&self) -> AppResult<AppSettings> {
        self.get_value("app_settings")?
            .map(|x| serde_json::from_str(&x))
            .transpose()?
            .ok_or_else(|| AppError::Other("Settings are unavailable".into()))
    }
    pub fn save_settings(&self, settings: &AppSettings) -> AppResult<()> {
        self.set_value("app_settings", &serde_json::to_string(settings)?)
    }
    pub fn sync_paused(&self) -> AppResult<bool> {
        Ok(self.get_value("sync_paused")?.as_deref() == Some("true"))
    }
    pub fn set_sync_paused(&self, paused: bool) -> AppResult<()> {
        self.set_value("sync_paused", if paused { "true" } else { "false" })
    }
    pub fn last_synced_at(&self) -> AppResult<Option<String>> {
        self.get_value("last_synced_at")
    }
    pub fn set_last_synced_at(&self, value: &str) -> AppResult<()> {
        self.set_value("last_synced_at", value)
    }

    pub fn source_state(&self, relative_path: &str) -> AppResult<(u64, SessionContext)> {
        let result = self.connection.query_row("SELECT byte_offset,session_id,model,project_key,project_name,context_window,session_timestamp,cumulative_input_tokens,cumulative_output_tokens,cumulative_reasoning_tokens,cumulative_cached_tokens,cumulative_total_tokens FROM sync_sources WHERE relative_path=?1", [relative_path], |row| {
            Ok((row.get::<_, i64>(0)? as u64, SessionContext { session_id: row.get(1)?, model: row.get(2)?, project_key: row.get(3)?, project_name: row.get(4)?, context_window: row.get(5)?, timestamp: row.get(6)?, cumulative_usage: TokenTotals { input_tokens: row.get(7)?, cached_input_tokens: row.get(10)?, output_tokens: row.get(8)?, reasoning_output_tokens: row.get(9)?, total_tokens: row.get(11)? } }))
        }).optional()?;
        Ok(result.unwrap_or_default())
    }

    pub fn save_source_state(
        &self,
        relative_path: &str,
        offset: u64,
        size: u64,
        modified_at: Option<&str>,
        context: &SessionContext,
    ) -> AppResult<()> {
        self.connection.execute("INSERT INTO sync_sources(relative_path,byte_offset,file_size,modified_at,parser_version,session_id,model,project_key,project_name,context_window,session_timestamp,cumulative_input_tokens,cumulative_output_tokens,cumulative_reasoning_tokens,cumulative_cached_tokens,cumulative_total_tokens,last_synced_at) VALUES(?1,?2,?3,?4,3,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16) ON CONFLICT(relative_path) DO UPDATE SET byte_offset=excluded.byte_offset,file_size=excluded.file_size,modified_at=excluded.modified_at,parser_version=excluded.parser_version,session_id=excluded.session_id,model=excluded.model,project_key=excluded.project_key,project_name=excluded.project_name,context_window=excluded.context_window,session_timestamp=excluded.session_timestamp,cumulative_input_tokens=excluded.cumulative_input_tokens,cumulative_output_tokens=excluded.cumulative_output_tokens,cumulative_reasoning_tokens=excluded.cumulative_reasoning_tokens,cumulative_cached_tokens=excluded.cumulative_cached_tokens,cumulative_total_tokens=excluded.cumulative_total_tokens,last_synced_at=excluded.last_synced_at", params![relative_path, offset as i64, size as i64, modified_at, context.session_id, context.model, context.project_key, context.project_name, context.context_window, context.timestamp, context.cumulative_usage.input_tokens, context.cumulative_usage.output_tokens, context.cumulative_usage.reasoning_output_tokens, context.cumulative_usage.cached_input_tokens, context.cumulative_usage.total_tokens, Utc::now().to_rfc3339()])?;
        Ok(())
    }

    pub fn insert_event(&mut self, event: &UsageEvent) -> AppResult<bool> {
        let tx = self.connection.transaction()?;
        if let (Some(key), Some(name)) = (&event.project_key, &event.project_name) {
            tx.execute("INSERT OR IGNORE INTO projects(project_key,display_name,created_at) VALUES(?1,?2,?3)", params![key,name,Utc::now().to_rfc3339()])?;
        }
        tx.execute("INSERT INTO sessions(session_id,model,project_key,context_window,context_tokens,started_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?6) ON CONFLICT(session_id) DO UPDATE SET model=CASE WHEN excluded.model='' THEN sessions.model ELSE excluded.model END,project_key=COALESCE(excluded.project_key,sessions.project_key),context_window=COALESCE(excluded.context_window,sessions.context_window),context_tokens=excluded.context_tokens,updated_at=excluded.updated_at", params![event.session_id,event.model,event.project_key,event.context_window,event.totals.total_tokens,event.timestamp])?;
        let inserted = tx.execute("INSERT OR IGNORE INTO usage_events(event_id,session_id,timestamp,model,project_key,input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,reasoning_tokens,cached_tokens,total_tokens) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?9,?7,?10)", params![event.event_id,event.session_id,event.timestamp,event.model,event.project_key,event.totals.input_tokens,event.totals.cached_input_tokens,event.totals.output_tokens,event.totals.reasoning_output_tokens,event.totals.total_tokens])? > 0;
        if inserted {
            tx.execute("UPDATE sessions SET input_tokens=input_tokens+?2,cached_input_tokens=cached_input_tokens+?3,output_tokens=output_tokens+?4,reasoning_output_tokens=reasoning_output_tokens+?5,reasoning_tokens=reasoning_tokens+?5,cached_tokens=cached_tokens+?3,total_tokens=total_tokens+?6 WHERE session_id=?1", params![event.session_id,event.totals.input_tokens,event.totals.cached_input_tokens,event.totals.output_tokens,event.totals.reasoning_output_tokens,event.totals.total_tokens])?;
            tx.execute("INSERT INTO daily_usage(usage_date,input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,reasoning_tokens,cached_tokens,total_tokens,updated_at) VALUES(date(?1,'localtime'),?2,?3,?4,?5,?5,?3,?6,?7) ON CONFLICT(usage_date) DO UPDATE SET input_tokens=input_tokens+excluded.input_tokens,cached_input_tokens=cached_input_tokens+excluded.cached_input_tokens,output_tokens=output_tokens+excluded.output_tokens,reasoning_output_tokens=reasoning_output_tokens+excluded.reasoning_output_tokens,reasoning_tokens=reasoning_tokens+excluded.reasoning_tokens,cached_tokens=cached_tokens+excluded.cached_tokens,total_tokens=total_tokens+excluded.total_tokens,updated_at=excluded.updated_at", params![event.timestamp,event.totals.input_tokens,event.totals.cached_input_tokens,event.totals.output_tokens,event.totals.reasoning_output_tokens,event.totals.total_tokens,Utc::now().to_rfc3339()])?;
        }
        tx.commit()?;
        Ok(inserted)
    }

    fn token_totals_query(
        &self,
        sql: &str,
        args: &[&dyn rusqlite::ToSql],
    ) -> AppResult<TokenTotals> {
        Ok(self.connection.query_row(sql, args, |row| {
            Ok(TokenTotals {
                input_tokens: row.get(0)?,
                cached_input_tokens: row.get(1)?,
                output_tokens: row.get(2)?,
                reasoning_output_tokens: row.get(3)?,
                total_tokens: row.get(4)?,
            })
        })?)
    }

    fn quota_window(
        &self,
        window_modifier: &str,
        window_seconds: i64,
        forecast_modifier: &str,
        forecast_seconds: i64,
        limit_tokens: i64,
    ) -> AppResult<QuotaWindow> {
        let consumed_tokens: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(total_tokens),0) FROM usage_events WHERE datetime(timestamp) >= datetime('now',?1)",
            [window_modifier],
            |row| row.get(0),
        )?;
        let earliest: Option<String> = self
            .connection
            .query_row(
                "SELECT timestamp FROM usage_events WHERE datetime(timestamp) >= datetime('now',?1) ORDER BY datetime(timestamp) LIMIT 1",
                [window_modifier],
                |row| row.get(0),
            )
            .optional()?;
        let reset_at = earliest
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc) + Duration::seconds(window_seconds));
        let reset_in_seconds = reset_at.map(|value| (value - Utc::now()).num_seconds().max(0));
        let recent_tokens: i64 = self.connection.query_row(
            "SELECT COALESCE(SUM(total_tokens),0) FROM usage_events WHERE datetime(timestamp) >= datetime('now',?1)",
            [forecast_modifier],
            |row| row.get(0),
        )?;
        let projected_tokens = consumed_tokens.saturating_add(
            recent_tokens.saturating_mul(reset_in_seconds.unwrap_or(window_seconds))
                / forecast_seconds.max(1),
        );
        let usage_percent =
            (limit_tokens > 0).then(|| consumed_tokens as f64 * 100.0 / limit_tokens as f64);
        let forecast_status = if limit_tokens <= 0 {
            "unconfigured"
        } else if projected_tokens >= limit_tokens {
            "danger"
        } else if projected_tokens >= limit_tokens * 80 / 100
            || consumed_tokens >= limit_tokens * 80 / 100
        {
            "warning"
        } else {
            "safe"
        }
        .to_string();
        Ok(QuotaWindow {
            consumed_tokens,
            limit_tokens,
            remaining_tokens: limit_tokens.saturating_sub(consumed_tokens).max(0),
            usage_percent,
            reset_at: reset_at.map(|value| value.to_rfc3339()),
            reset_in_seconds,
            projected_tokens,
            forecast_status,
        })
    }

    pub fn dashboard(&self, range_days: i64) -> AppResult<DashboardData> {
        let today = self.token_totals_query("SELECT COALESCE(SUM(input_tokens),0),COALESCE(SUM(cached_input_tokens),0),COALESCE(SUM(output_tokens),0),COALESCE(SUM(reasoning_output_tokens),0),COALESCE(SUM(total_tokens),0) FROM usage_events WHERE date(timestamp,'localtime')=date('now','localtime')", &[])?;
        let current_session = self.connection.query_row("SELECT session_id,model,context_window,context_tokens,updated_at,input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,total_tokens FROM sessions ORDER BY updated_at DESC LIMIT 1", [], |row| {
            let context_window: Option<i64> = row.get(2)?; let context_tokens: i64 = row.get(3)?; let total: i64 = row.get(9)?;
            Ok(CurrentSession { session_id: row.get(0)?, model: row.get(1)?, context_window, context_tokens, usage_percent: context_window.filter(|x| *x > 0).map(|x| context_tokens as f64 * 100.0 / x as f64), updated_at: row.get(4)?, totals: TokenTotals { input_tokens: row.get(5)?, cached_input_tokens: row.get(6)?, output_tokens: row.get(7)?, reasoning_output_tokens: row.get(8)?, total_tokens: total } })
        }).optional()?;
        let chart = self.chart(range_days)?;
        let mut stmt = self.connection.prepare("SELECT p.project_key,p.display_name,COALESCE(SUM(u.total_tokens),0) total FROM projects p JOIN usage_events u ON u.project_key=p.project_key GROUP BY p.project_key,p.display_name ORDER BY total DESC LIMIT 8")?;
        let projects = stmt
            .query_map([], |row| {
                Ok(ProjectUsage {
                    project_key: row.get(0)?,
                    display_name: row.get(1)?,
                    total_tokens: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let model_usage = self.model_usage(range_days)?;
        let seven_day_total: i64 = self.connection.query_row("SELECT COALESCE(SUM(total_tokens),0) FROM daily_usage WHERE usage_date >= date('now','localtime','-6 days')", [], |row| row.get(0))?;
        let daily_average_tokens = seven_day_total / 7;
        let now = Local::now();
        let days_in_month = match now.month() {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if now.year() % 4 == 0 && (now.year() % 100 != 0 || now.year() % 400 == 0) => 29,
            _ => 28,
        };
        let month_used: i64 = self.connection.query_row("SELECT COALESCE(SUM(total_tokens),0) FROM daily_usage WHERE usage_date >= date('now','start of month')", [], |row| row.get(0))?;
        let projected_monthly_tokens =
            month_used + daily_average_tokens * (days_in_month as i64 - now.day() as i64);
        let settings = self.settings()?;
        let five_hour_quota = self.quota_window(
            "-5 hours",
            5 * 60 * 60,
            "-1 hour",
            60 * 60,
            settings.five_hour_limit,
        )?;
        let weekly_quota = self.quota_window(
            "-7 days",
            7 * 24 * 60 * 60,
            "-1 day",
            24 * 60 * 60,
            settings.weekly_limit,
        )?;
        let risk_level = if seven_day_total == 0 {
            "insufficient"
        } else if settings.monthly_budget <= 0
            || projected_monthly_tokens < settings.monthly_budget * 80 / 100
        {
            "low"
        } else if projected_monthly_tokens < settings.monthly_budget * 95 / 100 {
            "medium"
        } else {
            "high"
        }
        .to_string();
        Ok(DashboardData {
            today,
            current_session,
            chart,
            projects,
            model_usage,
            monthly_total_tokens: month_used,
            projected_monthly_tokens,
            daily_average_tokens,
            risk_level,
            five_hour_quota,
            weekly_quota,
            real_account_quota: self.latest_account_quota()?,
            sync_paused: self.sync_paused()?,
            last_synced_at: self.last_synced_at()?,
        })
    }

    pub fn save_account_quota(&self, quota: &AccountQuota) -> AppResult<()> {
        self.connection.execute(
            "INSERT INTO quota_snapshot(created_at,five_hour_used_percent,five_hour_remaining_percent,five_hour_reset_seconds,weekly_used_percent,weekly_remaining_percent,weekly_reset_seconds,status) VALUES(?1,?2,?3,?4,?5,?6,?7,'ok')",
            params![
                quota.created_at,
                quota.five_hour.as_ref().map(|window| window.used_percent),
                quota.five_hour.as_ref().map(|window| window.remaining_percent),
                quota.five_hour.as_ref().map(|window| window.reset_after_seconds),
                quota.weekly.as_ref().map(|window| window.used_percent),
                quota.weekly.as_ref().map(|window| window.remaining_percent),
                quota.weekly.as_ref().map(|window| window.reset_after_seconds),
            ],
        )?;
        Ok(())
    }

    pub fn save_account_quota_status(&self, status: &str) -> AppResult<()> {
        self.connection.execute(
            "INSERT INTO quota_snapshot(created_at,status) VALUES(?1,?2)",
            params![Utc::now().to_rfc3339(), status],
        )?;
        Ok(())
    }

    pub fn save_pricing_version(&self, version: &str, source: &str, updated_at: &str) -> AppResult<()> {
        self.connection.execute(
            "INSERT INTO pricing_versions(version,source,updated_at) VALUES(?1,?2,?3)",
            params![version, source, updated_at],
        )?;
        Ok(())
    }

    pub fn latest_account_quota(&self) -> AppResult<RealAccountQuota> {
        let snapshot = self.connection.query_row(
            "SELECT created_at,five_hour_used_percent,five_hour_remaining_percent,five_hour_reset_seconds,weekly_used_percent,weekly_remaining_percent,weekly_reset_seconds,status FROM quota_snapshot ORDER BY id DESC LIMIT 1",
            [],
            |row| {
                let five_used: Option<f64> = row.get(1)?;
                let five_remaining: Option<f64> = row.get(2)?;
                let five_reset: Option<i64> = row.get(3)?;
                let weekly_used: Option<f64> = row.get(4)?;
                let weekly_remaining: Option<f64> = row.get(5)?;
                let weekly_reset: Option<i64> = row.get(6)?;
                let status: String = row.get(7)?;
                let message = match status.as_str() {
                    "AUTH_NOT_FOUND" | "AUTH_INVALID" => "Please login Codex first",
                    "AUTH_EXPIRED" => "Authentication expired",
                    "CONNECTION_FAILED" => "Connection failed",
                    "ok" => "",
                    _ => "Quota unavailable",
                };
                Ok(RealAccountQuota {
                    created_at: Some(row.get(0)?),
                    five_hour: five_used.map(|used_percent| RealQuotaWindow {
                        used_percent,
                        remaining_percent: five_remaining.unwrap_or(0.0),
                        reset_after_seconds: five_reset.unwrap_or(0),
                    }),
                    weekly: weekly_used.map(|used_percent| RealQuotaWindow {
                        used_percent,
                        remaining_percent: weekly_remaining.unwrap_or(0.0),
                        reset_after_seconds: weekly_reset.unwrap_or(0),
                    }),
                    status,
                    message: message.into(),
                })
            },
        ).optional()?;
        Ok(snapshot.unwrap_or(RealAccountQuota {
            status: "NOT_SYNCED".into(),
            message: "Quota unavailable".into(),
            ..Default::default()
        }))
    }

    fn chart(&self, range_days: i64) -> AppResult<Vec<UsagePoint>> {
        let days = range_days.clamp(1, 30);
        if days == 1 {
            let mut existing = std::collections::HashMap::new();
            let mut stmt = self.connection.prepare("SELECT strftime('%H',timestamp,'localtime'),SUM(input_tokens),SUM(cached_input_tokens),SUM(output_tokens),SUM(reasoning_output_tokens),SUM(total_tokens) FROM usage_events WHERE date(timestamp,'localtime')=date('now','localtime') GROUP BY 1")?;
            for row in stmt.query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    TokenTotals {
                        input_tokens: r.get(1)?,
                        cached_input_tokens: r.get(2)?,
                        output_tokens: r.get(3)?,
                        reasoning_output_tokens: r.get(4)?,
                        total_tokens: r.get(5)?,
                    },
                ))
            })? {
                let (k, v) = row?;
                existing.insert(k, v);
            }
            return Ok((0..=Local::now().hour())
                .map(|hour| {
                    let key = format!("{hour:02}");
                    UsagePoint {
                        bucket: key.clone(),
                        label: format!("{key}:00"),
                        totals: existing.remove(&key).unwrap_or_default(),
                    }
                })
                .collect());
        }
        let mut existing = std::collections::HashMap::new();
        let mut stmt = self.connection.prepare("SELECT usage_date,input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,total_tokens FROM daily_usage WHERE usage_date >= date('now','localtime',?1) ORDER BY usage_date")?;
        let modifier = format!("-{} days", days - 1);
        for row in stmt.query_map([modifier], |r| {
            Ok((
                r.get::<_, String>(0)?,
                TokenTotals {
                    input_tokens: r.get(1)?,
                    cached_input_tokens: r.get(2)?,
                    output_tokens: r.get(3)?,
                    reasoning_output_tokens: r.get(4)?,
                    total_tokens: r.get(5)?,
                },
            ))
        })? {
            let (k, v) = row?;
            existing.insert(k, v);
        }
        let today = Local::now().date_naive();
        Ok((0..days)
            .map(|offset| {
                let date: NaiveDate = today - Duration::days(days - 1 - offset);
                let key = date.to_string();
                UsagePoint {
                    bucket: key.clone(),
                    label: date.format("%m/%d").to_string(),
                    totals: existing.remove(&key).unwrap_or_default(),
                }
            })
            .collect())
    }

    fn model_usage(&self, range_days: i64) -> AppResult<Vec<ModelUsage>> {
        let days = range_days.clamp(1, 30);
        let mut models = Vec::new();
        let mut statement = if days == 1 {
            self.connection.prepare("SELECT COALESCE(model,''),SUM(input_tokens),SUM(cached_input_tokens),SUM(output_tokens),SUM(reasoning_output_tokens),SUM(total_tokens) FROM usage_events WHERE date(timestamp,'localtime')=date('now','localtime') GROUP BY model ORDER BY SUM(total_tokens) DESC")?
        } else {
            let modifier = format!("-{} days", days - 1);
            let mut statement = self.connection.prepare("SELECT COALESCE(model,''),SUM(input_tokens),SUM(cached_input_tokens),SUM(output_tokens),SUM(reasoning_output_tokens),SUM(total_tokens) FROM usage_events WHERE date(timestamp,'localtime') >= date('now','localtime',?1) GROUP BY model ORDER BY SUM(total_tokens) DESC")?;
            let rows = statement.query_map([modifier], |row| {
                Ok(ModelUsage {
                    model: row.get(0)?,
                    totals: TokenTotals {
                        input_tokens: row.get(1)?,
                        cached_input_tokens: row.get(2)?,
                        output_tokens: row.get(3)?,
                        reasoning_output_tokens: row.get(4)?,
                        total_tokens: row.get(5)?,
                    },
                })
            })?;
            for row in rows {
                models.push(row?);
            }
            return Ok(models);
        };
        let rows = statement.query_map([], |row| {
            Ok(ModelUsage {
                model: row.get(0)?,
                totals: TokenTotals {
                    input_tokens: row.get(1)?,
                    cached_input_tokens: row.get(2)?,
                    output_tokens: row.get(3)?,
                    reasoning_output_tokens: row.get(4)?,
                    total_tokens: row.get(5)?,
                },
            })
        })?;
        for row in rows {
            models.push(row?);
        }
        Ok(models)
    }

    pub fn export_csv(&self, path: &Path) -> AppResult<usize> {
        let mut writer = csv::Writer::from_path(path)?;
        writer.write_record([
            "timestamp",
            "session_id",
            "model",
            "project",
            "input_tokens",
            "cached_input_tokens",
            "output_tokens",
            "reasoning_output_tokens",
            "total_tokens",
        ])?;
        let mut stmt = self.connection.prepare("SELECT u.timestamp,u.session_id,u.model,COALESCE(p.display_name,''),u.input_tokens,u.cached_input_tokens,u.output_tokens,u.reasoning_output_tokens,u.total_tokens FROM usage_events u LEFT JOIN projects p ON p.project_key=u.project_key ORDER BY u.timestamp")?;
        let mut rows = stmt.query([])?;
        let mut count = 0;
        while let Some(row) = rows.next()? {
            writer.serialize((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, i64>(8)?,
            ))?;
            count += 1;
        }
        writer.flush()?;
        Ok(count)
    }

    pub fn clear_usage_data(&mut self) -> AppResult<()> {
        let tx = self.connection.transaction()?;
        tx.execute_batch("DELETE FROM alert_history; DELETE FROM daily_usage; DELETE FROM usage_events; DELETE FROM sessions; DELETE FROM projects; DELETE FROM sync_sources;")?;
        tx.commit()?;
        Ok(())
    }

    pub fn alert_sent(&self, key: &str) -> AppResult<bool> {
        Ok(self
            .connection
            .query_row(
                "SELECT 1 FROM alert_history WHERE alert_key=?1",
                [key],
                |r| r.get::<_, i32>(0),
            )
            .optional()?
            .is_some())
    }
    pub fn mark_alert_sent(&self, key: &str, threshold: i64) -> AppResult<()> {
        self.connection.execute("INSERT OR IGNORE INTO alert_history(alert_key,threshold,usage_date,sent_at) VALUES(?1,?2,date('now','localtime'),?3)",params![key,threshold,Utc::now().to_rfc3339()])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn migrates_and_roundtrips_settings() {
        let temp = tempfile::tempdir().unwrap();
        let db = Database::open(&temp.path().join("test.db")).unwrap();
        assert_eq!(db.settings().unwrap().refresh_interval_seconds, 5);
        assert!(!db.privacy_salt().unwrap().is_empty());
        let mut settings = db.settings().unwrap();
        settings.language = crate::i18n::ZH_CN.into();
        db.save_settings(&settings).unwrap();
        assert_eq!(db.settings().unwrap().language, crate::i18n::ZH_CN);
    }

    #[test]
    fn saves_pricing_version_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let db = Database::open(&temp.path().join("pricing.db")).unwrap();
        db.save_pricing_version("2026-07-18", "online", "2026-07-18T12:00:00Z")
            .unwrap();
        let saved: (String, String, String) = db
            .connection
            .query_row(
                "SELECT version,source,updated_at FROM pricing_versions LIMIT 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            saved,
            (
                "2026-07-18".into(),
                "online".into(),
                "2026-07-18T12:00:00Z".into()
            )
        );
    }

    #[test]
    fn migrates_v3_usage_without_deleting_rows() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("legacy.db");
        {
            let connection = Connection::open(&path).unwrap();
            connection
                .execute_batch(include_str!("../../../migrations/0001_initial.sql"))
                .unwrap();
            connection
                .execute_batch(include_str!("../../../migrations/0002_session_context.sql"))
                .unwrap();
            connection
                .execute_batch(include_str!(
                    "../../../migrations/0003_cumulative_usage.sql"
                ))
                .unwrap();
            connection.execute("INSERT INTO sessions(session_id,started_at,updated_at,input_tokens,output_tokens,reasoning_tokens,cached_tokens,total_tokens) VALUES('legacy','2026-07-16T01:00:00Z','2026-07-16T01:00:00Z',100,40,10,20,140)", []).unwrap();
            connection.execute("INSERT INTO usage_events(event_id,session_id,timestamp,input_tokens,output_tokens,reasoning_tokens,cached_tokens,total_tokens) VALUES('event','legacy','2026-07-16T01:00:00Z',100,40,10,20,140)", []).unwrap();
            connection.execute("INSERT INTO daily_usage(usage_date,input_tokens,output_tokens,reasoning_tokens,cached_tokens,total_tokens,updated_at) VALUES('2026-07-16',100,40,10,20,140,'2026-07-16T01:00:00Z')", []).unwrap();
        }
        let db = Database::open(&path).unwrap();
        let version: i64 = db
            .connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        let values: (i64, i64, i64, i64, i64) = db
            .connection
            .query_row("SELECT input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,total_tokens FROM usage_events WHERE event_id='event'", [], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .unwrap();
        assert_eq!(version, 6);
        assert_eq!(
            db.connection
                .query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='pricing_versions'", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(values, (80, 20, 30, 10, 140));
        for table in ["sessions", "daily_usage"] {
            let sql = format!("SELECT input_tokens,cached_input_tokens,output_tokens,reasoning_output_tokens,total_tokens FROM {table} LIMIT 1");
            let migrated: (i64, i64, i64, i64, i64) = db
                .connection
                .query_row(&sql, [], |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                })
                .unwrap();
            assert_eq!(migrated, values);
        }
        assert_eq!(
            db.connection
                .query_row("SELECT COUNT(*) FROM sessions", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
    }

    #[test]
    fn calculates_rolling_quota_windows_and_forecast() {
        let temp = tempfile::tempdir().unwrap();
        let mut db = Database::open(&temp.path().join("quota.db")).unwrap();
        let mut settings = db.settings().unwrap();
        settings.five_hour_limit = 1_000;
        settings.weekly_limit = 10_000;
        db.save_settings(&settings).unwrap();
        let event = UsageEvent {
            event_id: "recent".into(),
            session_id: "quota-session".into(),
            timestamp: (Utc::now() - Duration::minutes(30)).to_rfc3339(),
            model: "gpt-codex".into(),
            project_key: None,
            project_name: None,
            context_window: None,
            totals: TokenTotals::from_categories(300, 100, 80, 20),
        };
        db.insert_event(&event).unwrap();
        let dashboard = db.dashboard(7).unwrap();
        assert_eq!(dashboard.five_hour_quota.consumed_tokens, 500);
        assert_eq!(dashboard.five_hour_quota.remaining_tokens, 500);
        assert_eq!(dashboard.five_hour_quota.usage_percent, Some(50.0));
        assert!(dashboard.five_hour_quota.reset_in_seconds.is_some());
        assert_eq!(dashboard.weekly_quota.consumed_tokens, 500);
        assert_ne!(dashboard.five_hour_quota.forecast_status, "unconfigured");
        assert_eq!(dashboard.monthly_total_tokens, 500);
        assert_eq!(dashboard.model_usage.len(), 1);
        assert_eq!(dashboard.model_usage[0].model, "gpt-codex");
        assert_eq!(dashboard.model_usage[0].totals.total_tokens, 500);
    }

    #[test]
    fn saves_real_quota_snapshot_without_credentials() {
        use crate::services::quota::QuotaWindowSnapshot;

        let temp = tempfile::tempdir().unwrap();
        let db = Database::open(&temp.path().join("real-quota.db")).unwrap();
        let quota = AccountQuota {
            created_at: "2026-07-16T10:00:00Z".into(),
            five_hour: None,
            weekly: Some(QuotaWindowSnapshot {
                used_percent: 42.0,
                remaining_percent: 58.0,
                reset_after_seconds: 580_000,
                window_seconds: Some(604_800),
            }),
            status: "ok".into(),
        };
        db.save_account_quota(&quota).unwrap();
        let stored = db.latest_account_quota().unwrap();
        assert!(stored.five_hour.is_none());
        assert_eq!(stored.weekly.unwrap().used_percent, 42.0);
        let columns: String = db
            .connection
            .prepare("SELECT name FROM pragma_table_info('quota_snapshot') ORDER BY cid")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .join(",");
        assert!(!columns.contains("token"));
        assert!(!columns.contains("account_id"));
    }
}
