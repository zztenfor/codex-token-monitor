"""Print non-sensitive database health metrics for release verification."""

import os
import sqlite3


database_path = os.path.join(
    os.environ.get("CODEX_TOKEN_MONITOR_DATA_DIR")
    or os.path.join(os.environ["APPDATA"], "com.local.codextokenmonitor"),
    "token_usage.db",
)
connection = sqlite3.connect(f"file:{database_path}?mode=ro", uri=True)

tables = connection.execute(
    "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
).fetchall()
print("tables=" + ",".join(row[0] for row in tables))

for table in ("usage_events", "sessions", "projects"):
    count = connection.execute(f"SELECT COUNT(*) FROM {table}").fetchone()[0]
    print(f"{table}={count}")

quota_count = connection.execute("SELECT COUNT(*) FROM quota_snapshot").fetchone()[0]
quota = connection.execute(
    "SELECT five_hour_used_percent, five_hour_remaining_percent, "
    "weekly_used_percent, weekly_remaining_percent, status "
    "FROM quota_snapshot ORDER BY id DESC LIMIT 1"
).fetchone()
print(f"quota_snapshots={quota_count}")
if quota:
    five_used, five_remaining, weekly_used, weekly_remaining, status = quota
    print(f"quota_status={status}")
    print(f"five_hour_used_percent={five_used}")
    print(f"five_hour_remaining_percent={five_remaining}")
    print(f"weekly_used_percent={weekly_used}")
    print(f"weekly_remaining_percent={weekly_remaining}")

total = connection.execute(
    "SELECT COALESCE(SUM(total_tokens), 0) FROM usage_events"
).fetchone()[0]
columns = connection.execute("PRAGMA table_info(usage_events)").fetchall()
print(f"total_tokens={total}")
print("usage_columns=" + ",".join(row[1] for row in columns))
version = connection.execute("SELECT MAX(version) FROM schema_migrations").fetchone()[0]
session = connection.execute(
    "SELECT context_tokens, context_window FROM sessions ORDER BY updated_at DESC LIMIT 1"
).fetchone()
print(f"migration_version={version}")
if session:
    context_tokens, context_window = session
    percent = 0 if not context_window else context_tokens * 100 / context_window
    print(f"current_context_tokens={context_tokens}")
    print(f"current_context_window={context_window}")
    print(f"current_context_percent={percent:.2f}")

verify_session = os.environ.get("CODEX_VERIFY_SESSION_ID")
if verify_session:
    verified_total = connection.execute(
        "SELECT total_tokens FROM sessions WHERE session_id = ?", (verify_session,)
    ).fetchone()
    print(
        "verified_session_total="
        + (str(verified_total[0]) if verified_total else "missing")
    )
