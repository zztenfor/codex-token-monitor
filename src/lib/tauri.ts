import { invoke } from "@tauri-apps/api/core";
import type { AppSettings, DashboardData, RealAccountQuota, SyncReport } from "./types";

export const api = {
  dashboard: (rangeDays: number) => invoke<DashboardData>("get_dashboard", { rangeDays }),
  settings: () => invoke<AppSettings>("get_settings"),
  saveSettings: (settings: AppSettings) => invoke<AppSettings>("save_settings", { settings }),
  syncNow: () => invoke<SyncReport>("sync_now"),
  syncAccountQuota: () => invoke<RealAccountQuota>("sync_account_quota"),
  setPaused: (paused: boolean) => invoke<boolean>("set_sync_paused", { paused }),
  exportCsv: (path: string) => invoke<number>("export_csv", { path }),
  clearData: () => invoke<void>("clear_usage_data"),
  detectCodexPath: () => invoke<string>("detect_codex_path"),
  toggleWidget: () => invoke<boolean>("toggle_floating_window"),
  hideWidget: () => invoke<void>("hide_floating_window"),
  openMain: () => invoke<void>("open_main_window"),
};
