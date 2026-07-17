use crate::{
    domain::models::SyncReport,
    error::AppResult,
    infrastructure::{
        codex::parser::{parse_line, SessionContext},
        database::Database,
    },
};
use chrono::{DateTime, Utc};
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};
use walkdir::WalkDir;

fn jsonl_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for name in ["sessions", "archived_sessions", "log", "logs"] {
        let directory = root.join(name);
        if !directory.is_dir() {
            continue;
        }
        for entry in WalkDir::new(directory)
            .follow_links(false)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case("jsonl"))
            {
                files.push(entry.into_path());
            }
        }
    }
    for name in ["history.jsonl", "session_index.jsonl"] {
        let path = root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    files
}

pub fn sync_codex_home(db: &mut Database, root: &Path) -> AppResult<SyncReport> {
    let mut report = SyncReport {
        synced_at: Utc::now().to_rfc3339(),
        ..Default::default()
    };
    let salt = db.privacy_salt()?;
    for path in jsonl_files(root) {
        report.files_scanned += 1;
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = match path.metadata() {
            Ok(value) => value,
            Err(_) => continue,
        };
        let file_size = metadata.len();
        let (saved_offset, saved_context) = db.source_state(&relative)?;
        let (mut offset, mut context) = if saved_offset > file_size {
            (0, SessionContext::default())
        } else {
            (saved_offset, saved_context)
        };
        if offset == file_size {
            continue;
        }
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(offset))?;
        loop {
            let line_offset = offset;
            let mut line = String::new();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            offset += bytes as u64;
            match parse_line(&line, &relative, line_offset, &mut context, &salt) {
                Ok(Some(event)) => {
                    if db.insert_event(&event)? {
                        report.events_inserted += 1
                    } else {
                        report.events_skipped += 1
                    }
                }
                Ok(None) => report.events_skipped += 1,
                Err(_) => report.parse_errors += 1,
            }
        }
        let modified = metadata
            .modified()
            .ok()
            .and_then(|x: SystemTime| DateTime::<Utc>::from(x).to_rfc3339().into());
        db.save_source_state(&relative, offset, file_size, modified.as_deref(), &context)?;
    }
    db.set_last_synced_at(&report.synced_at)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn sync_is_incremental_and_deduplicated() {
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join(".codex");
        std::fs::create_dir_all(home.join("sessions")).unwrap();
        let file_path = home.join("sessions/sample.jsonl");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file,r#"{{"timestamp":"2026-07-16T01:00:00Z","type":"session_meta","payload":{{"id":"s1","cwd":"C:\\work\\alpha","model":"gpt-codex"}}}}"#).unwrap();
        writeln!(file,r#"{{"timestamp":"2026-07-16T01:01:00Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{{"input_tokens":5,"output_tokens":3,"total_tokens":8}}}}}}}}"#).unwrap();
        let mut db = Database::open(&temp.path().join("monitor.db")).unwrap();
        assert_eq!(sync_codex_home(&mut db, &home).unwrap().events_inserted, 1);
        assert_eq!(sync_codex_home(&mut db, &home).unwrap().events_inserted, 0);
        let session = db.dashboard(7).unwrap().current_session.unwrap();
        assert_eq!(session.totals.total_tokens, 8);
        assert_eq!(session.context_tokens, 8);
    }
}
