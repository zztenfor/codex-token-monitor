use crate::{
    domain::models::{AppSettings, DashboardData, RealAccountQuota, SyncReport},
    error::{AppError, AppResult},
    infrastructure::{
        codex::{discovery, sync::sync_codex_home},
        database::Database,
    },
    services::quota::{QuotaError, QuotaService},
};
use crate::i18n;
use chrono::{Datelike, Local};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub struct AppState {
    pub database: Arc<Mutex<Database>>,
    pub sync_in_progress: AtomicBool,
    pub quota_sync_in_progress: AtomicBool,
}

impl AppState {
    pub fn new(database: Database) -> Self {
        Self {
            database: Arc::new(Mutex::new(database)),
            sync_in_progress: AtomicBool::new(false),
            quota_sync_in_progress: AtomicBool::new(false),
        }
    }
}

pub fn perform_account_quota_sync(app: &AppHandle) -> AppResult<RealAccountQuota> {
    let state = app.state::<AppState>();
    if state.quota_sync_in_progress.swap(true, Ordering::SeqCst) {
        return lock_db(&state)?.latest_account_quota();
    }
    let home = lock_db(&state)
        .and_then(|database| configured_codex_home(&database))
        .ok();
    let outcome = match home {
        Some(path) => QuotaService::new().and_then(|service| service.fetch(&path)),
        None => Err(QuotaError::AuthNotFound),
    };
    let result = (|| {
        let database = lock_db(&state)?;
        match outcome {
            Ok(quota) => {
                database.save_account_quota(&quota)?;
                tracing::info!("account quota sync completed");
            }
            Err(error) => {
                database.save_account_quota_status(error.code())?;
                tracing::warn!(error_code = error.code(), "account quota sync failed");
            }
        }
        database.latest_account_quota()
    })();
    state.quota_sync_in_progress.store(false, Ordering::SeqCst);
    if result.is_ok() {
        let _ = app.emit("account-quota-updated", ());
        let _ = update_tray_and_alerts(app);
    }
    result
}

fn lock_db(state: &AppState) -> AppResult<std::sync::MutexGuard<'_, Database>> {
    state
        .database
        .lock()
        .map_err(|_| AppError::Other("Database lock is poisoned".into()))
}

async fn run_db<T, F>(database: Arc<Mutex<Database>>, operation: F) -> AppResult<T>
where
    T: Send + 'static,
    F: FnOnce(&mut Database) -> AppResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let mut database = database
            .lock()
            .map_err(|_| AppError::Other("Database lock is poisoned".into()))?;
        operation(&mut database)
    })
    .await
    .map_err(|error| AppError::Other(format!("Background operation failed: {error}")))?
}

fn validate_settings(settings: &AppSettings) -> AppResult<()> {
    if ![1, 5, 10, 30].contains(&settings.refresh_interval_seconds) {
        return Err(AppError::InvalidSetting(
            "Refresh interval must be 1, 5, 10, or 30 seconds".into(),
        ));
    }
    if !["system", "light", "dark"].contains(&settings.theme.as_str()) {
        return Err(AppError::InvalidSetting("Unknown theme".into()));
    }
    if ![i18n::ZH_CN, i18n::EN_US].contains(&settings.language.as_str()) {
        return Err(AppError::InvalidSetting("Unknown language".into()));
    }
    if !settings.floating_opacity.is_finite() || !(0.2..=1.0).contains(&settings.floating_opacity) {
        return Err(AppError::InvalidSetting("Floating opacity must be between 0.2 and 1.0".into()));
    }
    if !["compact", "detailed"].contains(&settings.floating_mode.as_str()) {
        return Err(AppError::InvalidSetting("Unknown floating window mode".into()));
    }
    if settings.daily_budget < 0
        || settings.monthly_budget < 0
        || settings.five_hour_limit < 0
        || settings.weekly_limit < 0
    {
        return Err(AppError::InvalidSetting(
            "Budgets and quota limits cannot be negative".into(),
        ));
    }
    if !settings.codex_path.is_empty() && !discovery::is_codex_home(Path::new(&settings.codex_path))
    {
        return Err(AppError::InvalidSetting(
            "The selected folder is not a Codex data directory".into(),
        ));
    }
    Ok(())
}

pub fn configured_codex_home(db: &Database) -> AppResult<PathBuf> {
    let settings = db.settings()?;
    if !settings.codex_path.is_empty() && discovery::is_codex_home(Path::new(&settings.codex_path))
    {
        Ok(PathBuf::from(settings.codex_path))
    } else {
        discovery::detect_codex_home()
    }
}

pub fn perform_sync(app: &AppHandle) -> AppResult<SyncReport> {
    let state = app.state::<AppState>();
    if state.sync_in_progress.swap(true, Ordering::SeqCst) {
        return Ok(SyncReport {
            synced_at: chrono::Utc::now().to_rfc3339(),
            ..Default::default()
        });
    }
    let result = (|| {
        let mut db = lock_db(&state)?;
        if db.sync_paused()? {
            return Ok(SyncReport {
                synced_at: chrono::Utc::now().to_rfc3339(),
                ..Default::default()
            });
        }
        let home = configured_codex_home(&db)?;
        sync_codex_home(&mut db, &home)
    })();
    state.sync_in_progress.store(false, Ordering::SeqCst);
    if result.is_ok() {
        let _ = app.emit("usage-updated", ());
        let _ = update_tray_and_alerts(app);
    }
    result
}

fn compact(value: i64) -> String {
    if value >= 1_000_000 {
        format!("{:.1}M", value as f64 / 1_000_000.0)
    } else if value >= 1_000 {
        format!("{:.1}K", value as f64 / 1_000.0)
    } else {
        value.to_string()
    }
}

pub fn update_tray_and_alerts(app: &AppHandle) -> AppResult<()> {
    let state = app.state::<AppState>();
    let db = lock_db(&state)?;
    let data = db.dashboard(7)?;
    let settings = db.settings()?;
    if let Some(tray) = app.tray_by_id("main") {
        let quota_percent = |value: Option<f64>| {
            value
                .map(|percent| format!("{percent:.0}%"))
                .unwrap_or_else(|| "--".into())
        };
        let _ = tray.set_tooltip(Some(format!(
            "Codex\n5H: {}\n7D: {}\n{}: {}",
            quota_percent(
                data.real_account_quota
                    .five_hour
                    .as_ref()
                    .map(|window| window.used_percent),
            ),
            quota_percent(
                data.real_account_quota
                    .weekly
                    .as_ref()
                    .map(|window| window.used_percent),
            ),
            i18n::tray_text(&settings.language, "today"),
            compact(data.today.total_tokens)
        )));
    }
    if settings.daily_budget <= 0 {
        return Ok(());
    }
    let percent = data.today.total_tokens * 100 / settings.daily_budget;
    for (threshold, enabled) in [
        (80, settings.alert80),
        (90, settings.alert90),
        (95, settings.alert95),
    ] {
        if enabled && percent >= threshold {
            let now = Local::now();
            let key = format!(
                "daily-{:04}-{:02}-{:02}-{threshold}",
                now.year(),
                now.month(),
                now.day()
            );
            if !db.alert_sent(&key)? {
                let _ = app
                    .notification()
                    .builder()
                    .title("Codex Token Monitor")
                    .body(i18n::quota_alert(&settings.language, threshold))
                    .show();
                db.mark_alert_sent(&key, threshold)?;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_dashboard(
    state: tauri::State<'_, AppState>,
    range_days: i64,
) -> AppResult<DashboardData> {
    run_db(state.database.clone(), move |database| {
        database.dashboard(range_days)
    })
    .await
}
#[tauri::command]
pub async fn get_settings(state: tauri::State<'_, AppState>) -> AppResult<AppSettings> {
    run_db(state.database.clone(), |database| database.settings()).await
}
#[tauri::command]
pub async fn save_settings(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    settings: AppSettings,
) -> AppResult<AppSettings> {
    validate_settings(&settings)?;
    let saved = run_db(state.database.clone(), move |database| {
        database.save_settings(&settings)?;
        Ok(settings)
    })
    .await?;
    apply_floating_window_settings(&app, &saved)?;
    let _ = app.emit("language-changed", saved.language.clone());
    let _ = app.emit("floating-settings-changed", &saved);
    let _ = crate::refresh_tray_menu(&app);
    let _ = update_tray_and_alerts(&app);
    Ok(saved)
}

pub fn apply_floating_window_settings(app: &AppHandle, settings: &AppSettings) -> AppResult<()> {
    let Some(window) = app.get_webview_window("widget") else {
        return Ok(());
    };
    if let Err(error) = window.set_always_on_top(settings.floating_always_on_top) {
        tracing::warn!(%error, "floating always-on-top is unavailable");
    }
    if let Err(error) = window.set_ignore_cursor_events(settings.floating_click_through) {
        tracing::warn!(%error, "floating click-through is unavailable");
    }
    let size = if settings.floating_mode == "detailed" {
        tauri::Size::Logical(tauri::LogicalSize::new(468.0, 480.0))
    } else {
        tauri::Size::Logical(tauri::LogicalSize::new(320.0, 176.0))
    };
    if let Err(error) = window.set_size(size) {
        tracing::warn!(%error, "floating window resize is unavailable");
    }
    // Tauri 2.11 has no runtime set_opacity API. The widget applies this value as a CSS fallback.
    Ok(())
}
#[tauri::command]
pub async fn sync_now(app: AppHandle) -> AppResult<SyncReport> {
    tauri::async_runtime::spawn_blocking(move || perform_sync(&app))
        .await
        .map_err(|error| AppError::Other(format!("Background sync failed: {error}")))?
}
#[tauri::command]
pub async fn sync_account_quota(app: AppHandle) -> AppResult<RealAccountQuota> {
    tauri::async_runtime::spawn_blocking(move || perform_account_quota_sync(&app))
        .await
        .map_err(|error| AppError::Other(format!("Quota sync task failed: {error}")))?
}
#[tauri::command]
pub async fn set_sync_paused(state: tauri::State<'_, AppState>, paused: bool) -> AppResult<bool> {
    run_db(state.database.clone(), move |database| {
        database.set_sync_paused(paused)?;
        Ok(paused)
    })
    .await
}
#[tauri::command]
pub async fn export_csv(state: tauri::State<'_, AppState>, path: String) -> AppResult<usize> {
    run_db(state.database.clone(), move |database| {
        database.export_csv(Path::new(&path))
    })
    .await
}
#[tauri::command]
pub async fn clear_usage_data(state: tauri::State<'_, AppState>) -> AppResult<()> {
    run_db(state.database.clone(), |database| {
        database.clear_usage_data()
    })
    .await
}
#[tauri::command]
pub async fn detect_codex_path() -> AppResult<String> {
    tauri::async_runtime::spawn_blocking(|| {
        Ok(discovery::detect_codex_home()?
            .to_string_lossy()
            .to_string())
    })
    .await
    .map_err(|error| AppError::Other(format!("Path detection failed: {error}")))?
}

fn window_error(error: tauri::Error) -> AppError {
    AppError::Other(format!("Window operation failed: {error}"))
}

#[tauri::command]
pub fn toggle_floating_window(app: AppHandle) -> AppResult<bool> {
    let window = app
        .get_webview_window("widget")
        .ok_or_else(|| AppError::Other("Floating window is unavailable".into()))?;
    if window.is_visible().map_err(window_error)? {
        window.hide().map_err(window_error)?;
        return Ok(false);
    }
    if let Some(monitor) = window.current_monitor().map_err(window_error)? {
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let window_size = window.outer_size().map_err(window_error)?;
        let x = monitor_position.x + monitor_size.width as i32 - window_size.width as i32 - 18;
        let y = monitor_position.y + monitor_size.height as i32 - window_size.height as i32 - 58;
        window
            .set_position(tauri::PhysicalPosition::new(x, y))
            .map_err(window_error)?;
    }
    window.show().map_err(window_error)?;
    window.set_focus().map_err(window_error)?;
    Ok(true)
}

#[tauri::command]
pub fn hide_floating_window(app: AppHandle) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("widget") {
        window.hide().map_err(window_error)?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_main_window(app: AppHandle) -> AppResult<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::Other("Main window is unavailable".into()))?;
    window.show().map_err(window_error)?;
    window.unminimize().map_err(window_error)?;
    window.set_focus().map_err(window_error)?;
    Ok(())
}
