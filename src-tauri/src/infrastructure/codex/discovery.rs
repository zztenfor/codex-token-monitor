use crate::error::{AppError, AppResult};
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn detect_codex_home() -> AppResult<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(value) = env::var_os("CODEX_HOME") {
        candidates.push(PathBuf::from(value));
    }
    if let Some(home) = env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(home).join(".codex"));
    }
    if let Some(home) = env::var_os("HOME") {
        candidates.push(PathBuf::from(home).join(".codex"));
    }
    candidates
        .into_iter()
        .find(|path| is_codex_home(path))
        .ok_or(AppError::CodexHomeNotFound)
}

pub fn is_codex_home(path: &Path) -> bool {
    path.is_dir()
        && (path.join("sessions").is_dir()
            || path.join("history.jsonl").is_file()
            || path.join("session_index.jsonl").is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_session_directory() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join("sessions")).unwrap();
        assert!(is_codex_home(temp.path()));
    }
}
