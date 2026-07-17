pub const ZH_CN: &str = "zh-CN";
pub const EN_US: &str = "en-US";

pub fn normalize_language(language: &str) -> &'static str {
    if language.to_ascii_lowercase().starts_with("zh") { ZH_CN } else { EN_US }
}

pub fn system_language() -> String {
    sys_locale::get_locale().map(|locale| normalize_language(&locale).to_string()).unwrap_or_else(|| EN_US.to_string())
}

pub fn tray_text(language: &str, key: &str) -> &'static str {
    match (normalize_language(language), key) {
        (ZH_CN, "open") => "打开窗口",
        (ZH_CN, "settings") => "设置",
        (ZH_CN, "floating_window") => "悬浮窗",
        (ZH_CN, "pause_sync") => "暂停同步",
        (ZH_CN, "resume_sync") => "恢复同步",
        (ZH_CN, "exit") => "退出",
        (ZH_CN, "today") => "今日",
        (_, "open") => "Open",
        (_, "settings") => "Settings",
        (_, "floating_window") => "Floating Window",
        (_, "pause_sync") => "Pause Sync",
        (_, "resume_sync") => "Resume Sync",
        (_, "exit") => "Exit",
        (_, "today") => "Today",
        _ => "Codex Token Monitor",
    }
}

pub fn quota_alert(language: &str, threshold: i64) -> String {
    if normalize_language(language) == ZH_CN { format!("Codex Token 用量已达到每日预算的 {threshold}%") } else { format!("Codex token usage reached {threshold}% of the daily budget") }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalizes_supported_and_future_locales() {
        assert_eq!(normalize_language("zh-CN"), ZH_CN);
        assert_eq!(normalize_language("zh-TW"), ZH_CN);
        assert_eq!(normalize_language("ja-JP"), EN_US);
    }
    #[test]
    fn translates_tray_and_notification_text() {
        assert_eq!(tray_text(ZH_CN, "open"), "打开窗口");
        assert_eq!(tray_text(EN_US, "open"), "Open");
        assert!(quota_alert(ZH_CN, 90).contains("90%"));
        assert!(quota_alert(EN_US, 90).contains("90%"));
    }
}
