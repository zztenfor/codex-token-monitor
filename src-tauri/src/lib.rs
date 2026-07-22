mod commands;
mod domain;
mod error;
mod infrastructure;
mod i18n;
mod services;

use commands::{configured_codex_home, perform_account_quota_sync, perform_sync, AppState};
use i18n::{normalize_language, tray_text};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::{path::PathBuf, sync::mpsc, thread, time::Duration};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn translated_tray_menu(app: &tauri::AppHandle, language: &str, paused: bool) -> Result<Menu<tauri::Wry>, Box<dyn std::error::Error>> {
    let open = MenuItem::with_id(app, "open", tray_text(language, "open"), true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", tray_text(language, "settings"), true, None::<&str>)?;
    let widget = MenuItem::with_id(app, "widget", tray_text(language, "floating_window"), true, None::<&str>)?;
    let pause_key = if paused { "resume_sync" } else { "pause_sync" };
    let pause = MenuItem::with_id(app, "pause", tray_text(language, pause_key), true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", tray_text(language, "exit"), true, None::<&str>)?;
    Ok(Menu::with_items(app, &[&open, &settings, &widget, &pause, &exit])?)
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let db = state.database.lock().map_err(|_| "Database lock is poisoned")?;
    let settings = db.settings()?;
    let language = normalize_language(&settings.language);
    let paused = db.sync_paused()?;
    drop(db);
    let menu = translated_tray_menu(app.handle(), language, paused)?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("Application icon is unavailable")?;
    TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Codex Token Monitor")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => show_main(app),
            "settings" => {
                show_main(app);
                let _ = app.emit("open-settings", ());
            }
            "widget" => {
                let _ = commands::toggle_floating_window(app.clone());
            }
            "pause" => {
                let state = app.state::<AppState>();
                if let Ok(db) = state.database.lock() {
                    if let Ok(value) = db.sync_paused() {
                        let _ = db.set_sync_paused(!value);
                        let _ = app.emit("usage-updated", ());
                    }
                };
                let _ = refresh_tray_menu(app);
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

pub fn refresh_tray_menu(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let db = state.database.lock().map_err(|_| "Database lock is poisoned")?;
    let settings = db.settings()?;
    let paused = db.sync_paused()?;
    let language = normalize_language(&settings.language);
    drop(db);
    if let Some(tray) = app.tray_by_id("main") {
        tray.set_menu(Some(translated_tray_menu(app, language, paused)?))?;
    }
    Ok(())
}

fn start_watcher(app: tauri::AppHandle) {
    thread::spawn(move || {
        let (tx, rx) = mpsc::channel();
        let mut watcher: Option<RecommendedWatcher> = None;
        let mut watched = PathBuf::new();
        loop {
            let (root, interval, paused) = {
                let state = app.state::<AppState>();
                let snapshot = match state.database.lock() {
                    Ok(db) => {
                        let root = configured_codex_home(&db).unwrap_or_default();
                        let settings = db.settings().unwrap_or_default();
                        let paused = db.sync_paused().unwrap_or(false);
                        (root, settings.refresh_interval_seconds, paused)
                    }
                    Err(_) => return,
                };
                snapshot
            };
            if root != watched {
                watcher = None;
                if root.is_dir() {
                    let sender = tx.clone();
                    if let Ok(mut next) =
                        notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                            let _ = sender.send(event);
                        })
                    {
                        if next.watch(&root, RecursiveMode::Recursive).is_ok() {
                            watcher = Some(next);
                            watched = root;
                        }
                    }
                }
            }
            let changed = rx
                .recv_timeout(Duration::from_secs(interval.max(1)))
                .is_ok();
            if !paused && (changed || watcher.is_some()) {
                if let Err(error) = perform_sync(&app) {
                    tracing::warn!(%error,"background sync failed");
                }
            }
        }
    });
}

fn start_quota_worker(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        if let Err(error) = perform_account_quota_sync(&app) {
            tracing::warn!(%error, "quota worker persistence failed");
        }
        thread::sleep(Duration::from_secs(5 * 60));
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("codex_token_monitor=info".parse().unwrap()),
        )
        .init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let data_dir = std::env::var_os("CODEX_TOKEN_MONITOR_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or(app.path().app_data_dir()?);
            let db = infrastructure::database::Database::open(&data_dir.join("token_usage.db"))?;
            app.manage(AppState::new(db, data_dir));
            let initial_settings = app
                .state::<AppState>()
                .database
                .lock()
                .ok()
                .and_then(|database| database.settings().ok());
            if let Some(settings) = initial_settings {
                let _ = commands::apply_floating_window_settings(app.handle(), &settings);
            }
            setup_tray(app)?;
            if std::env::var_os("CODEX_TOKEN_MONITOR_SHOW_WIDGET").as_deref()
                == Some(std::ffi::OsStr::new("1"))
            {
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(750));
                    let _ = commands::toggle_floating_window(app_handle);
                });
            }
            start_watcher(app.handle().clone());
            start_quota_worker(app.handle().clone());
            show_main(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard,
            commands::get_settings,
            commands::save_settings,
            commands::sync_now,
            commands::sync_account_quota,
            commands::fetch_pricing_source,
            commands::fetch_model_catalog_source,
            commands::read_pricing_cache,
            commands::save_pricing_cache,
            commands::set_sync_paused,
            commands::export_csv,
            commands::clear_usage_data,
            commands::detect_codex_path,
            commands::toggle_floating_window,
            commands::set_floating_window_expanded,
            commands::hide_floating_window,
            commands::open_main_window
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Codex Token Monitor");
}
