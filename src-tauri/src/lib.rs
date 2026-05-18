use std::fs;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use chrono::{Datelike, Duration, Local, TimeZone};
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::CloseHandle;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::SystemInformation::GetTickCount;
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId,
};

static DEVIATION_STATE: Lazy<Mutex<DeviationState>> = Lazy::new(|| Mutex::new(DeviationState::default()));
static FOREGROUND_SAMPLE_STATE: Lazy<Mutex<ForegroundSampleState>> =
    Lazy::new(|| Mutex::new(ForegroundSampleState::default()));
const IDLE_PROMPT_THRESHOLD_MS: i64 = 300_000;
const BUSINESS_DAY_START_HOUR: i64 = 4;
const HEATMAP_LOCK_GOAL_SECONDS: i64 = 7200;
#[cfg(target_os = "windows")]
const WINDOWS_RUN_REGISTRY_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const WINDOWS_RUN_VALUE_NAME: &str = "TimePrism";

#[derive(Default)]
struct DeviationState {
    mismatch_since: Option<i64>,
    pending_alert: bool,
    cooldown_until: i64,
}

#[derive(Default)]
struct ForegroundSampleState {
    last: Option<ForegroundSnapshot>,
    diagnostics: Vec<ForegroundCaptureDiagnostic>,
    idle_segment_start_ms: Option<i64>,
    pending_idle_prompts: Vec<IdlePromptEntry>,
    next_idle_prompt_id: i64,
    remembered_idle_decision: Option<String>,
}

#[derive(Clone)]
struct ForegroundSnapshot {
    process_name: String,
    window_title: String,
    captured_at_ms: i64,
}

#[derive(Clone, Serialize)]
struct ForegroundCaptureDiagnostic {
    id: i64,
    captured_at_ms: i64,
    observed_process_name: String,
    observed_window_title: String,
    stored: bool,
    block_reason: Option<String>,
    rule_saved: bool,
    rule_mapped_type: String,
}

#[derive(Clone, Serialize)]
struct IdlePromptEntry {
    id: i64,
    start_timestamp: i64,
    end_timestamp: i64,
    duration_ms: i64,
    deferred_until_timestamp: Option<i64>,
}

#[derive(Serialize)]
struct Category {
    id: i64,
    parent_id: Option<i64>,
    name: String,
    root_type: String,
    color_hex: String,
}

#[derive(Deserialize)]
struct CreateCategoryInput {
    parent_id: i64,
    name: String,
    color_hex: Option<String>,
}

#[derive(Serialize)]
struct TodaySummary {
    learn_seconds: i64,
    rest_seconds: i64,
    active_session_id: Option<i64>,
}

#[derive(Deserialize)]
struct SettlePetWindowInput {
    mode: String,
}

#[derive(Serialize)]
struct PetWindowSettleResult {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    state: String,
}

#[derive(Serialize)]
struct TopApp {
    process_name: String,
    seconds: i64,
}

#[derive(Serialize)]
struct LearnHeatmapCell {
    day: String,
    learn_seconds: i64,
    level: String,
}

#[derive(Serialize)]
struct UsageStackSegment {
    name: String,
    seconds: i64,
}

#[derive(Serialize)]
struct UsageStackDay {
    day: String,
    total_seconds: i64,
    learn_seconds: i64,
    rest_seconds: i64,
    segments: Vec<UsageStackSegment>,
}

#[derive(Serialize)]
struct RecentLogEntry {
    id: i64,
    process_name: String,
    window_title: String,
    start_timestamp: i64,
    duration_ms: i64,
}

#[derive(Serialize)]
struct AppRuleEntry {
    process_name: String,
    mapped_type: String,
    privacy_level: String,
    updated_at: i64,
}

#[derive(Serialize)]
struct PendingRuleProcess {
    process_name: String,
    last_seen_timestamp: i64,
    last_window_title: String,
    total_seconds: i64,
}

#[derive(Deserialize)]
struct SaveAppRuleInput {
    process_name: String,
    mapped_type: String,
    privacy_level: Option<String>,
}

#[derive(Serialize)]
struct DeviationCheck {
    triggered: bool,
    process_name: String,
    reason: String,
    active_root_type: Option<String>,
    mapped_type: Option<String>,
    suggested_root_category_id: Option<i64>,
}

#[derive(Serialize)]
struct PrivacySettings {
    curtain_enabled: bool,
    browser_title_mode: String,
    whitelist_only_enabled: bool,
}

#[derive(Deserialize)]
struct UpdatePrivacySettingsInput {
    curtain_enabled: bool,
    browser_title_mode: String,
    whitelist_only_enabled: bool,
}

#[derive(Deserialize)]
struct SetWhitelistItemInput {
    process_name: String,
    enabled: bool,
}

#[derive(Deserialize)]
struct ResolveIdlePromptInput {
    prompt_id: i64,
    decision: String,
    remember_this_session: Option<bool>,
}

#[derive(Serialize)]
struct IdleMemoryState {
    remembered_decision: Option<String>,
}

#[derive(Serialize, Clone)]
struct ReminderEntry {
    id: i64,
    content: String,
    repeat_rule: String,
    sort_order: i64,
    remind_at: Option<i64>,
    daily_time_minutes: Option<i64>,
    weekly_days: Option<Vec<i64>>,
    snooze_until: Option<i64>,
    next_due_timestamp: i64,
    done: bool,
    completed_day_key: Option<String>,
    completed_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
struct SaveReminderInput {
    id: Option<i64>,
    content: String,
    repeat_rule: String,
    remind_at: Option<i64>,
    daily_time_minutes: Option<i64>,
    weekly_days: Option<Vec<i64>>,
}

#[derive(Deserialize)]
struct SetReminderDoneInput {
    id: i64,
    done: bool,
}

#[derive(Deserialize)]
struct SetReminderOrderInput {
    ordered_ids: Vec<i64>,
}

fn push_foreground_diagnostic(entry: ForegroundCaptureDiagnostic) -> Result<(), String> {
    let mut state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
    state.diagnostics.push(entry);
    if state.diagnostics.len() > 60 {
        let drop_count = state.diagnostics.len() - 60;
        state.diagnostics.drain(0..drop_count);
    }
    Ok(())
}

fn resolve_rule_mapping(conn: &Connection, process_name: &str) -> (bool, String) {
    let key = normalize_process_key(process_name);
    let mapped = conn
        .query_row(
            "SELECT mapped_type FROM app_rules WHERE process_name = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|v| v.trim().to_uppercase());

    if let Some(value) = mapped {
        let safe = if matches!(value.as_str(), "LEARN" | "REST" | "IGNORE") {
            value
        } else {
            "IGNORE".to_string()
        };
        return (true, safe);
    }

    (false, "IGNORE".to_string())
}

fn business_day_start_from_local(now: chrono::DateTime<Local>) -> Result<i64, String> {
    let shifted = now - Duration::hours(BUSINESS_DAY_START_HOUR);
    let start_naive = shifted
        .date_naive()
        .and_hms_opt(BUSINESS_DAY_START_HOUR as u32, 0, 0)
        .ok_or_else(|| "failed to build business day start".to_string())?;
    let start_local = Local
        .from_local_datetime(&start_naive)
        .single()
        .ok_or_else(|| "failed to convert business day start to local timestamp".to_string())?;
    Ok(start_local.timestamp())
}

fn business_day_window_from_local(now: chrono::DateTime<Local>) -> Result<(i64, i64), String> {
    let start = business_day_start_from_local(now)?;
    Ok((start, start + 86_400))
}

fn business_day_start_for_timestamp(ts: i64) -> Result<i64, String> {
    let local_dt = Local
        .timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| "failed to convert timestamp to local datetime".to_string())?;
    business_day_start_from_local(local_dt)
}

fn business_day_key_from_start(start_ts: i64) -> Result<String, String> {
    let local_dt = Local
        .timestamp_opt(start_ts, 0)
        .single()
        .ok_or_else(|| "failed to convert business day start to local datetime".to_string())?;
    Ok(local_dt.format("%Y-%m-%d").to_string())
}

fn overlap_seconds(seg_start: i64, seg_end: i64, window_start: i64, window_end: i64) -> i64 {
    let start = seg_start.max(window_start);
    let end = seg_end.min(window_end);
    (end - start).max(0)
}

fn merge_intervals_total(mut intervals: Vec<(i64, i64)>) -> i64 {
    if intervals.is_empty() {
        return 0;
    }

    intervals.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut total = 0_i64;
    let mut current = intervals[0];
    for (start, end) in intervals.into_iter().skip(1) {
        if start <= current.1 {
            current.1 = current.1.max(end);
        } else {
            total += (current.1 - current.0).max(0);
            current = (start, end);
        }
    }

    total + (current.1 - current.0).max(0)
}

fn ensure_heatmap_snapshot_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS daily_heatmap_snapshot (
            day_key TEXT PRIMARY KEY,
            learn_seconds INTEGER NOT NULL,
            goal_seconds INTEGER NOT NULL,
            level TEXT NOT NULL CHECK(level IN ('GRAY', 'YELLOW', 'GREEN')),
            sealed_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_daily_heatmap_snapshot_day_key ON daily_heatmap_snapshot(day_key);",
    )
    .map_err(|e| format!("failed to ensure daily_heatmap_snapshot table: {e}"))?;
    Ok(())
}

fn parse_i64_config(conn: &Connection, key: &str, default_value: i64) -> i64 {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<i64>().ok())
    .unwrap_or(default_value)
}

fn compute_learn_seconds_for_window(conn: &Connection, start_ts: i64, end_ts: i64) -> Result<i64, String> {
    let mut log_stmt = conn
        .prepare(
            "SELECT l.start_timestamp, l.duration_ms
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = 'LEARN'
               AND l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare learn-seconds log query: {e}"))?;

    let log_rows = log_stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?.max(0)))
        })
        .map_err(|e| format!("failed to query learn-seconds log rows: {e}"))?;

    let mut log_intervals = Vec::new();
    for row in log_rows {
        let (seg_start, duration_ms) =
            row.map_err(|e| format!("failed to parse learn-seconds log row: {e}"))?;
        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end > clip_start {
            log_intervals.push((clip_start, clip_end));
        }
    }

    let from_logs = merge_intervals_total(log_intervals).max(0);

    if from_logs > 0 {
        return Ok(from_logs);
    }

    let mut session_stmt = conn
        .prepare(
            "SELECT ts.start_time, COALESCE(ts.end_time, ?2)
             FROM task_sessions ts
             JOIN categories c ON c.id = ts.category_id
             WHERE c.root_type = 'LEARN'
               AND ts.start_time < ?2
               AND COALESCE(ts.end_time, ?2) > ?1",
        )
        .map_err(|e| format!("failed to prepare learn-seconds session query: {e}"))?;

    let session_rows = session_stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("failed to query learn-seconds session rows: {e}"))?;

    let mut session_intervals = Vec::new();
    for row in session_rows {
        let (seg_start, seg_end) =
            row.map_err(|e| format!("failed to parse learn-seconds session row: {e}"))?;
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end > clip_start {
            session_intervals.push((clip_start, clip_end));
        }
    }

    let from_sessions = merge_intervals_total(session_intervals).max(0);

    Ok(from_sessions)
}

#[cfg(target_os = "windows")]
fn query_auto_start_enabled_internal() -> Result<bool, String> {
    let output = Command::new("reg")
        .args(["query", WINDOWS_RUN_REGISTRY_PATH, "/v", WINDOWS_RUN_VALUE_NAME])
        .output()
        .map_err(|e| format!("failed to query Windows startup entry: {e}"))?;

    Ok(output.status.success())
}

#[cfg(not(target_os = "windows"))]
fn query_auto_start_enabled_internal() -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "windows")]
fn set_auto_start_enabled_internal(enabled: bool) -> Result<bool, String> {
    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| format!("failed to locate current executable: {e}"))?;
        let exe_arg = format!("\"{}\"", exe.display());
        let status = Command::new("reg")
            .args([
                "add",
                WINDOWS_RUN_REGISTRY_PATH,
                "/v",
                WINDOWS_RUN_VALUE_NAME,
                "/t",
                "REG_SZ",
                "/d",
                exe_arg.as_str(),
                "/f",
            ])
            .status()
            .map_err(|e| format!("failed to enable Windows startup entry: {e}"))?;
        if !status.success() {
            return Err("failed to enable Windows startup entry".to_string());
        }
    } else {
        let status = Command::new("reg")
            .args([
                "delete",
                WINDOWS_RUN_REGISTRY_PATH,
                "/v",
                WINDOWS_RUN_VALUE_NAME,
                "/f",
            ])
            .status()
            .map_err(|e| format!("failed to disable Windows startup entry: {e}"))?;
        if !status.success() {
            return Ok(false);
        }
    }

    query_auto_start_enabled_internal()
}

#[cfg(not(target_os = "windows"))]
fn set_auto_start_enabled_internal(_enabled: bool) -> Result<bool, String> {
    Ok(false)
}

fn seal_historical_heatmap_snapshot(
    conn: &Connection,
    day_start_ts: i64,
    goal_seconds: i64,
) -> Result<(), String> {
    let day_key = business_day_key_from_start(day_start_ts)?;
    let exists = conn
        .query_row(
            "SELECT 1 FROM daily_heatmap_snapshot WHERE day_key = ?1 LIMIT 1",
            [day_key.clone()],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    if exists {
        return Ok(());
    }

    let day_end_ts = day_start_ts + 86_400;
    let learn_seconds = compute_learn_seconds_for_window(conn, day_start_ts, day_end_ts)?.max(0);
    let level = if learn_seconds <= 0 {
        "GRAY"
    } else if learn_seconds < goal_seconds {
        "YELLOW"
    } else {
        "GREEN"
    };

    conn.execute(
        "INSERT OR IGNORE INTO daily_heatmap_snapshot (day_key, learn_seconds, goal_seconds, level, sealed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![day_key, learn_seconds, goal_seconds, level, Local::now().timestamp()],
    )
    .map_err(|e| format!("failed to insert heatmap snapshot: {e}"))?;

    Ok(())
}

fn ensure_data_dir(dir: &Path, label: &str) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create {label} dir {}: {e}", dir.display()))
}

fn legacy_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve legacy app data dir: {e}"))?;
    ensure_data_dir(&data_dir, "legacy app data")?;
    Ok(data_dir.join("timeprism.db"))
}

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app local data dir: {e}"))?;
    ensure_data_dir(&data_dir, "app local data")?;
    Ok(data_dir.join("timeprism.db"))
}

fn try_migrate_legacy_db(app: &AppHandle, target_path: &Path) -> Result<(), String> {
    if target_path.exists() {
        return Ok(());
    }

    let legacy_path = legacy_db_path(app)?;
    if !legacy_path.exists() {
        return Ok(());
    }

    fs::copy(&legacy_path, target_path).map_err(|e| {
        format!(
            "failed to migrate legacy sqlite db from {} to {}: {e}",
            legacy_path.display(),
            target_path.display()
        )
    })?;

    for suffix in ["-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{}", legacy_path.display(), suffix));
        if from.exists() {
            let to = PathBuf::from(format!("{}{}", target_path.display(), suffix));
            fs::copy(&from, &to).map_err(|e| {
                format!(
                    "failed to migrate sqlite sidecar from {} to {}: {e}",
                    from.display(),
                    to.display()
                )
            })?;
        }
    }

    Ok(())
}

fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let preferred_path = db_path(app)?;
    let migration_error = try_migrate_legacy_db(app, &preferred_path).err();
    let legacy_path = legacy_db_path(app)?;

    if let Some(err) = migration_error {
        return Connection::open(&legacy_path).map_err(|fallback_err| {
            format!(
                "failed to migrate legacy sqlite db to preferred location: {err}; fallback legacy db {} also failed: {fallback_err}",
                legacy_path.display()
            )
        });
    }

    match Connection::open(&preferred_path) {
        Ok(conn) => Ok(conn),
        Err(primary_err) => {
            Connection::open(&legacy_path).map_err(|fallback_err| {
                format!(
                    "failed to open preferred sqlite db {}: {primary_err}; fallback legacy db {} also failed: {fallback_err}",
                    preferred_path.display(),
                    legacy_path.display()
                )
            })
        }
    }
}

fn ensure_app_rules_time_columns(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(app_rules)")
        .map_err(|e| format!("failed to prepare app_rules table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query app_rules table info rows: {e}"))?;

    let mut has_created = false;
    let mut has_updated = false;
    for row in rows {
        let col = row.map_err(|e| format!("failed to parse app_rules table info row: {e}"))?;
        if col == "created_at" {
            has_created = true;
        }
        if col == "updated_at" {
            has_updated = true;
        }
    }

    if !has_created {
        conn.execute("ALTER TABLE app_rules ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0", [])
            .map_err(|e| format!("failed to add created_at to app_rules: {e}"))?;
    }
    if !has_updated {
        conn.execute("ALTER TABLE app_rules ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0", [])
            .map_err(|e| format!("failed to add updated_at to app_rules: {e}"))?;
    }

    let now_ts = Local::now().timestamp();
    conn.execute(
        "UPDATE app_rules
         SET created_at = CASE WHEN created_at <= 0 THEN ?1 ELSE created_at END,
             updated_at = CASE WHEN updated_at <= 0 THEN ?1 ELSE updated_at END",
        [now_ts],
    )
    .map_err(|e| format!("failed to backfill app_rules timestamps: {e}"))?;

    Ok(())
}

fn ensure_reminders_weekly_columns(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(reminders)")
        .map_err(|e| format!("failed to prepare reminders table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query reminders table info rows: {e}"))?;

    let mut has_weekly_days = false;
    for row in rows {
        let col = row.map_err(|e| format!("failed to parse reminders table info row: {e}"))?;
        if col == "weekly_days" {
            has_weekly_days = true;
        }
    }

    if !has_weekly_days {
        conn.execute_batch(
            r#"
            ALTER TABLE reminders RENAME TO reminders_legacy;
            CREATE TABLE reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE', 'DAILY', 'WEEKLY')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                remind_at INTEGER NULL,
                daily_time_minutes INTEGER NULL,
                weekly_days TEXT NULL,
                is_completed INTEGER NOT NULL DEFAULT 0,
                completed_day_key TEXT NULL,
                completed_at INTEGER NULL,
                snooze_until INTEGER NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            INSERT INTO reminders (
                id, content, repeat_rule, sort_order, remind_at, daily_time_minutes, weekly_days,
                is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at
            )
            SELECT
                id, content, repeat_rule, 0, remind_at, daily_time_minutes, NULL,
                is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at
            FROM reminders_legacy;
            DROP TABLE reminders_legacy;
            "#,
        )
        .map_err(|e| format!("failed to migrate reminders for weekly support: {e}"))?;
        conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
            CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
            CREATE INDEX IF NOT EXISTS idx_reminders_due_weekly ON reminders(repeat_rule, daily_time_minutes, weekly_days, completed_day_key);
            "#,
        )
        .map_err(|e| format!("failed to recreate reminders indexes after migration: {e}"))?;
    }

    Ok(())
}

fn ensure_reminders_sort_order(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(reminders)")
        .map_err(|e| format!("failed to prepare reminders sort-order table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query reminders sort-order table info rows: {e}"))?;

    let mut has_sort_order = false;
    for row in rows {
        let col = row.map_err(|e| format!("failed to parse reminders sort-order table info row: {e}"))?;
        if col == "sort_order" {
            has_sort_order = true;
            break;
        }
    }

    if !has_sort_order {
        conn.execute(
            "ALTER TABLE reminders ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| format!("failed to add sort_order to reminders: {e}"))?;
    }

    let mut order_stmt = conn
        .prepare("SELECT id FROM reminders ORDER BY sort_order ASC, created_at ASC, id ASC")
        .map_err(|e| format!("failed to prepare reminder sort backfill query: {e}"))?;
    let ids = order_stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| format!("failed to query reminder ids for sort backfill: {e}"))?;

    for (index, row) in ids.enumerate() {
        let id = row.map_err(|e| format!("failed to parse reminder id for sort backfill: {e}"))?;
        conn.execute(
            "UPDATE reminders SET sort_order = ?1 WHERE id = ?2",
            params![index as i64, id],
        )
        .map_err(|e| format!("failed to backfill reminder sort order: {e}"))?;
    }

    Ok(())
}

fn init_database(app: &AppHandle) -> Result<(), String> {
    let conn = open_connection(app)?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("failed to enable foreign keys: {e}"))?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          parent_id INTEGER NULL REFERENCES categories(id),
          name TEXT NOT NULL,
          color_hex TEXT NOT NULL DEFAULT '#4ade80',
          root_type TEXT NOT NULL CHECK(root_type IN ('LEARN', 'REST'))
        );

        CREATE TABLE IF NOT EXISTS task_sessions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          category_id INTEGER NOT NULL REFERENCES categories(id),
          start_time INTEGER NOT NULL,
          end_time INTEGER NULL,
          is_flow_target INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS app_usage_logs (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          process_name TEXT NOT NULL,
          window_title TEXT NOT NULL,
          start_timestamp INTEGER NOT NULL,
          duration_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_rules (
          process_name TEXT PRIMARY KEY,
          mapped_type TEXT NOT NULL CHECK(mapped_type IN ('LEARN', 'REST', 'IGNORE')),
          privacy_level TEXT NOT NULL CHECK(privacy_level IN ('NORMAL', 'BLUR_TITLE', 'WHITELIST_ONLY'))
        );

                CREATE TABLE IF NOT EXISTS app_config (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS app_whitelist (
                    process_name TEXT PRIMARY KEY
                );

                CREATE TABLE IF NOT EXISTS reminders (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content TEXT NOT NULL,
                    repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE', 'DAILY', 'WEEKLY')),
                    sort_order INTEGER NOT NULL DEFAULT 0,
                    remind_at INTEGER NULL,
                    daily_time_minutes INTEGER NULL,
                    weekly_days TEXT NULL,
                    is_completed INTEGER NOT NULL DEFAULT 0,
                    completed_day_key TEXT NULL,
                    completed_at INTEGER NULL,
                    snooze_until INTEGER NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );

        CREATE INDEX IF NOT EXISTS idx_task_sessions_start_time ON task_sessions(start_time);
        CREATE INDEX IF NOT EXISTS idx_app_usage_logs_start ON app_usage_logs(start_timestamp);
        CREATE INDEX IF NOT EXISTS idx_app_usage_logs_process ON app_usage_logs(process_name);
                CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
                CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
        "#,
    )
    .map_err(|e| format!("failed to run schema init: {e}"))?;

    ensure_app_rules_time_columns(&conn)?;
    ensure_heatmap_snapshot_table(&conn)?;
    ensure_reminders_weekly_columns(&conn)?;
    ensure_reminders_sort_order(&conn)?;
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_weekly ON reminders(repeat_rule, daily_time_minutes, weekly_days, completed_day_key);
        "#,
    )
    .map_err(|e| format!("failed to ensure reminder indexes: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO categories (id, parent_id, name, color_hex, root_type) VALUES (1, NULL, '学习', '#22c55e', 'LEARN')",
        [],
    )
    .map_err(|e| format!("failed to seed LEARN root: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO categories (id, parent_id, name, color_hex, root_type) VALUES (2, NULL, '休息', '#f97316', 'REST')",
        [],
    )
    .map_err(|e| format!("failed to seed REST root: {e}"))?;

    let default_rules = [
        ("code.exe", "LEARN", "NORMAL"),
        ("pycharm64.exe", "LEARN", "NORMAL"),
        ("idea64.exe", "LEARN", "NORMAL"),
        ("devenv.exe", "LEARN", "NORMAL"),
        ("desktop.shell.exe", "IGNORE", "NORMAL"),
        ("steam.exe", "REST", "NORMAL"),
        ("dota2.exe", "REST", "NORMAL"),
        ("bilibili.exe", "REST", "NORMAL"),
        ("msedge.exe", "IGNORE", "BLUR_TITLE"),
        ("chrome.exe", "IGNORE", "BLUR_TITLE"),
    ];

    let now_ts = Local::now().timestamp();
    for (process_name, mapped_type, privacy_level) in default_rules {
        conn.execute(
            "INSERT OR IGNORE INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![process_name, mapped_type, privacy_level, now_ts],
        )
        .map_err(|e| format!("failed to seed app rule {process_name}: {e}"))?;
    }

    let default_config = [
        ("curtain_enabled", "false"),
        ("browser_title_mode", "BLUR"),
        ("browser_blur_enabled", "true"),
        ("whitelist_only_enabled", "false"),
        ("heatmap_goal_seconds", "7200"),
    ];
    for (key, value) in default_config {
        conn.execute(
            "INSERT OR IGNORE INTO app_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("failed to seed app config {key}: {e}"))?;
    }

    let default_whitelist = ["code.exe", "pycharm64.exe", "idea64.exe", "devenv.exe", "explorer.exe"];
    for process_name in default_whitelist {
        conn.execute(
            "INSERT OR IGNORE INTO app_whitelist (process_name) VALUES (?1)",
            params![process_name],
        )
        .map_err(|e| format!("failed to seed whitelist {process_name}: {e}"))?;
    }

    Ok(())
}

fn parse_bool_config(conn: &Connection, key: &str, default_value: bool) -> bool {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .map(|v| v.eq_ignore_ascii_case("true"))
    .unwrap_or(default_value)
}

fn normalize_process_key(process_name: &str) -> String {
    let normalized = process_name
        .trim()
        .trim_matches('"')
        .replace('\\', "/")
        .to_lowercase();

    let from_path = Path::new(&normalized)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string());

    from_path
        .or_else(|| {
            let trimmed = normalized.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "unknown.exe".to_string())
}

fn parse_browser_title_mode(conn: &Connection) -> String {
    let from_mode_key = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = 'browser_title_mode'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|v| v.trim().to_uppercase())
        .filter(|v| matches!(v.as_str(), "FULL" | "BLUR" | "NONE"));

    if let Some(mode) = from_mode_key {
        return mode;
    }

    if parse_bool_config(conn, "browser_blur_enabled", true) {
        "BLUR".to_string()
    } else {
        "FULL".to_string()
    }
}

fn is_browser_process(process_name: &str) -> bool {
    matches!(
        process_name,
        "chrome.exe"
            | "msedge.exe"
            | "firefox.exe"
            | "opera.exe"
            | "brave.exe"
            | "vivaldi.exe"
            | "iexplore.exe"
    )
}

fn contains_incognito_keyword(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("incognito")
        || lower.contains("inprivate")
        || lower.contains("private browsing")
        || lower.contains("无痕")
        || lower.contains("隐私")
}

fn is_desktop_shell_window(process_name: &str, window_title: &str) -> bool {
    if process_name != "explorer.exe" {
        return false;
    }
    let t = window_title.trim().to_lowercase();
    t.is_empty()
        || t == "program manager"
        || t.contains("workerw")
        || t.contains("desktop")
        || t.contains("桌面")
}

fn process_log_with_privacy(
    conn: &Connection,
    process_name: &str,
    window_title: &str,
) -> Result<(Option<(String, String)>, Option<String>), String> {
    let curtain_enabled = parse_bool_config(conn, "curtain_enabled", false);
    if curtain_enabled {
        return Ok((None, Some("curtain_enabled".to_string())));
    }

    let mut final_process = normalize_process_key(process_name);
    let title_raw = window_title.trim();
    let mut final_title = if title_raw.is_empty() {
        "Untitled Window".to_string()
    } else {
        title_raw.to_string()
    };

    let browser_title_mode = parse_browser_title_mode(conn);
    let whitelist_only_enabled = parse_bool_config(conn, "whitelist_only_enabled", false);
    let is_browser = is_browser_process(&final_process);

    if is_browser && contains_incognito_keyword(title_raw) {
        return Ok((None, Some("incognito_window".to_string())));
    }

    if is_desktop_shell_window(&final_process, &final_title) {
        final_process = "desktop.shell.exe".to_string();
        final_title = "Desktop Shell".to_string();
    }

    let process_privacy = conn
        .query_row(
            "SELECT privacy_level FROM app_rules WHERE process_name = ?1",
            [final_process.clone()],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "NORMAL".to_string())
        .to_uppercase();

    if is_browser {
        match browser_title_mode.as_str() {
            "BLUR" => {
                final_title = "Web Browser".to_string();
            }
            "NONE" => {
                final_title = "Not Collected".to_string();
            }
            _ => {}
        }
    }

    if process_privacy == "BLUR_TITLE" {
        final_title = "Hidden Window".to_string();
    }

    if whitelist_only_enabled || process_privacy == "WHITELIST_ONLY" {
        let whitelisted = conn
            .query_row(
                "SELECT 1 FROM app_whitelist WHERE process_name = ?1 LIMIT 1",
                [final_process.clone()],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        if !whitelisted {
            final_process = "uncategorized.exe".to_string();
            final_title = "Hidden by Whitelist".to_string();
            return Ok((
                Some((final_process, final_title)),
                Some("whitelist_blocked".to_string()),
            ));
        }
    }

    Ok((Some((final_process, final_title)), None))
}

#[cfg(target_os = "windows")]
fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        return Ok(None);
    }

    let mut pid: u32 = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
    }
    if pid == 0 {
        return Ok(None);
    }

    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    let mut title = String::new();
    if title_len > 0 {
        let mut buffer = vec![0u16; (title_len + 1) as usize];
        let copied = unsafe { GetWindowTextW(hwnd, buffer.as_mut_ptr(), title_len + 1) };
        if copied > 0 {
            title = String::from_utf16_lossy(&buffer[..copied as usize]);
        }
    }
    if title.trim().is_empty() {
        title = "Untitled Window".to_string();
    }

    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    if process.is_null() {
        return Ok(Some((format!("pid-{pid}.exe"), title)));
    }

    let mut path_buf = vec![0u16; 1024];
    let mut size = path_buf.len() as u32;
    let query_ok = unsafe { QueryFullProcessImageNameW(process, 0, path_buf.as_mut_ptr(), &mut size) };
    unsafe {
        CloseHandle(process);
    }

    let process_name = if query_ok != 0 && size > 0 {
        String::from_utf16_lossy(&path_buf[..size as usize])
    } else {
        format!("pid-{pid}.exe")
    };

    Ok(Some((process_name, title)))
}

#[cfg(not(target_os = "windows"))]
fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
    Err("foreground capture is currently supported only on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn current_idle_millis() -> Result<i64, String> {
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };

    let ok = unsafe { GetLastInputInfo(&mut info as *mut LASTINPUTINFO) };
    if ok == 0 {
        return Err("GetLastInputInfo failed".to_string());
    }

    let now_tick = unsafe { GetTickCount() };
    let elapsed = now_tick.wrapping_sub(info.dwTime) as i64;
    Ok(elapsed.max(0))
}

#[cfg(not(target_os = "windows"))]
fn current_idle_millis() -> Result<i64, String> {
    Ok(0)
}

fn append_usage_log_direct(
    conn: &Connection,
    process_name: &str,
    window_title: &str,
    start_timestamp: i64,
    duration_ms: i64,
) -> Result<bool, String> {
    let span = duration_ms.max(0);

    let last_row = conn
        .query_row(
            "SELECT id, process_name, window_title, start_timestamp, duration_ms
             FROM app_usage_logs
             ORDER BY id DESC
             LIMIT 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            },
        )
        .ok();

    if let Some((last_id, last_process, last_title, last_start, last_duration_ms)) = last_row {
        let last_end = last_start + (last_duration_ms.max(0) / 1000);
        let is_same_signature = last_process == process_name && last_title == window_title;
        let is_contiguous = start_timestamp >= last_start && start_timestamp <= last_end + 2;

        if is_same_signature && is_contiguous {
            conn.execute(
                "UPDATE app_usage_logs
                 SET duration_ms = duration_ms + ?1
                 WHERE id = ?2",
                params![span, last_id],
            )
            .map_err(|e| format!("failed to extend app usage log segment: {e}"))?;
            return Ok(true);
        }
    }

    conn.execute(
        "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![process_name, window_title, start_timestamp, span],
    )
    .map_err(|e| format!("failed to append app usage log: {e}"))?;

    Ok(true)
}

fn upsert_app_rule_entry(
    conn: &Connection,
    process_name: &str,
    mapped_type: &str,
    privacy_level: &str,
) -> Result<(), String> {
    let now_ts = Local::now().timestamp();
    conn.execute(
        "INSERT INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)
         ON CONFLICT(process_name) DO UPDATE SET
         mapped_type=excluded.mapped_type,
         privacy_level=excluded.privacy_level,
         updated_at=excluded.updated_at",
        params![process_name, mapped_type, privacy_level, now_ts],
    )
    .map_err(|e| format!("failed to upsert app rule: {e}"))?;
    Ok(())
}

fn persist_idle_prompt_decision(
    conn: &Connection,
    prompt: &IdlePromptEntry,
    decision: &str,
) -> Result<bool, String> {
    let (process_name, mapped_type, title) = match decision {
        "LEARN" => ("__idle_learn__.exe", "LEARN", "Idle Segment · Learn"),
        "REST" => ("__idle_rest__.exe", "REST", "Idle Segment · Rest"),
        "IDLE" => ("__idle__.exe", "IGNORE", "Idle Segment · Unclassified"),
        _ => return Err("invalid idle decision".to_string()),
    };

    let process_key = normalize_process_key(process_name);
    upsert_app_rule_entry(conn, &process_key, mapped_type, "NORMAL")?;
    append_usage_log_direct(
        conn,
        &process_key,
        title,
        prompt.start_timestamp,
        prompt.duration_ms,
    )
}

fn append_usage_log_record(
    conn: &Connection,
    process_name: &str,
    window_title: &str,
    start_timestamp: i64,
    duration_ms: i64,
) -> Result<(bool, Option<String>), String> {
    let (processed, block_reason) = process_log_with_privacy(conn, process_name, window_title)?;
    let Some((safe_process_name, safe_window_title)) = processed else {
        return Ok((false, block_reason));
    };

    let stored = append_usage_log_direct(
        conn,
        &safe_process_name,
        &safe_window_title,
        start_timestamp,
        duration_ms,
    )?;
    Ok((stored, block_reason))
}

fn active_root_type(conn: &Connection) -> Result<Option<String>, String> {
    let value = conn
        .query_row(
            "SELECT c.root_type
             FROM task_sessions ts
             JOIN categories c ON c.id = ts.category_id
             WHERE ts.end_time IS NULL
             ORDER BY ts.id DESC
             LIMIT 1",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok();
    Ok(value)
}

fn normalize_repeat_rule(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "NONE" | "DAILY" | "WEEKLY") {
        Ok(normalized)
    } else {
        Err("repeat_rule must be NONE, DAILY, or WEEKLY".to_string())
    }
}

fn normalize_weekly_days(days: Option<Vec<i64>>) -> Option<Vec<i64>> {
    let mut safe = Vec::new();
    for day in days.unwrap_or_default() {
        if (0..=6).contains(&day) && !safe.contains(&day) {
            safe.push(day);
        }
    }
    safe.sort_unstable();
    if safe.is_empty() {
        None
    } else {
        Some(safe)
    }
}

fn weekly_days_to_db(days: &[i64]) -> String {
    days.iter().map(|day| day.to_string()).collect::<Vec<_>>().join(",")
}

fn parse_weekly_days_db(value: Option<String>) -> Option<Vec<i64>> {
    let parsed = value?
        .split(',')
        .filter_map(|chunk| chunk.trim().parse::<i64>().ok())
        .filter(|day| (0..=6).contains(day))
        .collect::<Vec<_>>();
    normalize_weekly_days(Some(parsed))
}

fn next_weekly_due_timestamp(
    today_start_ts: i64,
    today_weekday: i64,
    weekly_days: &[i64],
    minutes: Option<i64>,
    done_today: bool,
) -> i64 {
    let Some(clamped_minutes) = minutes.map(|v| v.clamp(0, 1439)) else {
        return NO_DUE_TIMESTAMP;
    };
    for offset in 0..14 {
        let candidate_weekday = (today_weekday + offset) % 7;
        if !weekly_days.contains(&candidate_weekday) {
            continue;
        }
        if offset == 0 && done_today {
            continue;
        }
        return today_start_ts + offset * 86_400 + clamped_minutes * 60;
    }
    NO_DUE_TIMESTAMP
}

const NO_DUE_TIMESTAMP: i64 = i64::MAX / 4;

fn collect_reminders(conn: &Connection, include_completed: bool, cap: usize) -> Result<Vec<ReminderEntry>, String> {
    let (today_start_ts, _) = business_day_window_from_local(Local::now())?;
    let today_key = business_day_key_from_start(today_start_ts)?;
    let tomorrow_start_ts = today_start_ts + 86_400;

    let query_limit = (cap as i64 * 6).clamp(30, 600);
    let mut stmt = conn
        .prepare(
            "SELECT id,
                    content,
                    repeat_rule,
                    sort_order,
                    remind_at,
                    daily_time_minutes,
                    weekly_days,
                    is_completed,
                    completed_day_key,
                    completed_at,
                    snooze_until,
                    created_at,
                    updated_at
             FROM reminders
             ORDER BY sort_order ASC, updated_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("failed to prepare reminders query: {e}"))?;

    let rows = stmt
        .query_map([query_limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, i64>(12)?,
            ))
        })
        .map_err(|e| format!("failed to query reminders rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (
            id,
            content,
            repeat_rule_raw,
            sort_order,
            remind_at,
            daily_time_minutes,
            weekly_days_raw,
            is_completed,
            completed_day_key,
            completed_at,
            snooze_until,
            created_at,
            updated_at,
        ) = row.map_err(|e| format!("failed to parse reminder row: {e}"))?;

        let repeat_rule = normalize_repeat_rule(&repeat_rule_raw).unwrap_or_else(|_| "NONE".to_string());
        let snooze_until_safe = snooze_until.filter(|v| *v > 0);
        let weekly_days = parse_weekly_days_db(weekly_days_raw);
        let today_weekday = Local
            .timestamp_opt(today_start_ts, 0)
            .single()
            .map(|dt| dt.weekday().num_days_from_sunday() as i64)
            .unwrap_or(0);

        if repeat_rule == "DAILY" {
            let minutes = daily_time_minutes.map(|v| v.clamp(0, 1439));
            let base_due = minutes.map(|v| today_start_ts + v * 60);
            let done_today = completed_day_key
                .as_deref()
                .map(|day| day == today_key)
                .unwrap_or(false);
            let mut next_due = if done_today {
                minutes.map(|v| tomorrow_start_ts + v * 60).unwrap_or(NO_DUE_TIMESTAMP)
            } else {
                base_due.unwrap_or(NO_DUE_TIMESTAMP)
            };
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            if !include_completed && done_today {
                continue;
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at: None,
                daily_time_minutes: minutes,
                weekly_days: None,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done: done_today,
                completed_day_key,
                completed_at,
                created_at,
                updated_at,
            });
        } else if repeat_rule == "WEEKLY" {
            let minutes = daily_time_minutes.map(|v| v.clamp(0, 1439));
            let days = weekly_days.clone().unwrap_or_default();
            let done_today = completed_day_key
                .as_deref()
                .map(|day| day == today_key)
                .unwrap_or(false);
            let mut next_due = next_weekly_due_timestamp(today_start_ts, today_weekday, &days, minutes, done_today);
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            if !include_completed && done_today {
                continue;
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at: None,
                daily_time_minutes: minutes,
                weekly_days,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done: done_today,
                completed_day_key,
                completed_at,
                created_at,
                updated_at,
            });
        } else {
            let done = is_completed > 0;
            if !include_completed && done {
                continue;
            }

            let mut next_due = remind_at.filter(|v| *v > 0).unwrap_or(NO_DUE_TIMESTAMP);
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at,
                daily_time_minutes: None,
                weekly_days: None,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done,
                completed_day_key: None,
                completed_at,
                created_at,
                updated_at,
            });
        }
    }

    result.sort_by(|a, b| {
        a.done
            .cmp(&b.done)
            .then_with(|| a.sort_order.cmp(&b.sort_order))
            .then_with(|| a.next_due_timestamp.cmp(&b.next_due_timestamp))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });

    if result.len() > cap {
        result.truncate(cap);
    }

    Ok(result)
}

#[tauri::command]
fn list_reminders(
    app: AppHandle,
    limit: Option<i64>,
    include_completed: Option<bool>,
) -> Result<Vec<ReminderEntry>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(50).clamp(1, 200) as usize;
    collect_reminders(&conn, include_completed.unwrap_or(false), cap)
}

#[tauri::command]
fn list_due_reminders(app: AppHandle, limit: Option<i64>) -> Result<Vec<ReminderEntry>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(6).clamp(1, 30) as usize;
    let now_ts = Local::now().timestamp();
    let mut due = collect_reminders(&conn, false, 200)?
        .into_iter()
        .filter(|item| !item.done && item.next_due_timestamp <= now_ts && item.next_due_timestamp < NO_DUE_TIMESTAMP)
        .collect::<Vec<_>>();
    if due.len() > cap {
        due.truncate(cap);
    }
    Ok(due)
}

#[tauri::command]
fn save_reminder(app: AppHandle, input: SaveReminderInput) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    let now_ts = Local::now().timestamp();
    let content = input.content.trim().to_string();
    if content.is_empty() {
        return Err("content cannot be empty".to_string());
    }

    let repeat_rule = normalize_repeat_rule(&input.repeat_rule)?;
    let next_sort_order = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM reminders",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    if repeat_rule == "DAILY" {
        let minutes = input.daily_time_minutes.map(|v| v.clamp(0, 1439));
        if let Some(id) = input.id {
            let changed = conn
                .execute(
                    "UPDATE reminders
                     SET content = ?1,
                         repeat_rule = 'DAILY',
                         remind_at = NULL,
                         daily_time_minutes = ?2,
                         weekly_days = NULL,
                         is_completed = 0,
                         completed_day_key = NULL,
                         completed_at = NULL,
                         snooze_until = NULL,
                         updated_at = ?3
                     WHERE id = ?4",
                    params![content, minutes, now_ts, id],
                )
                .map_err(|e| format!("failed to update daily reminder: {e}"))?;
            if changed == 0 {
                return Err("reminder not found".to_string());
            }
            return Ok(id);
        }

        conn.execute(
            "INSERT INTO reminders
             (content, repeat_rule, sort_order, remind_at, daily_time_minutes, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
             VALUES (?1, 'DAILY', ?2, NULL, ?3, 0, NULL, NULL, NULL, ?4, ?4)",
            params![content, next_sort_order, minutes, now_ts],
        )
        .map_err(|e| format!("failed to create daily reminder: {e}"))?;
        return Ok(conn.last_insert_rowid());
    }

    if repeat_rule == "WEEKLY" {
        let minutes = input.daily_time_minutes.map(|v| v.clamp(0, 1439));
        let weekly_days = normalize_weekly_days(input.weekly_days);
        let weekly_days_db = weekly_days.as_ref().map(|days| weekly_days_to_db(days));
        if let Some(id) = input.id {
            let changed = conn
                .execute(
                    "UPDATE reminders
                     SET content = ?1,
                         repeat_rule = 'WEEKLY',
                         remind_at = NULL,
                         daily_time_minutes = ?2,
                         weekly_days = ?3,
                         is_completed = 0,
                         completed_day_key = NULL,
                         completed_at = NULL,
                         snooze_until = NULL,
                         updated_at = ?4
                     WHERE id = ?5",
                    params![content, minutes, weekly_days_db, now_ts, id],
                )
                .map_err(|e| format!("failed to update weekly reminder: {e}"))?;
            if changed == 0 {
                return Err("reminder not found".to_string());
            }
            return Ok(id);
        }

        conn.execute(
            "INSERT INTO reminders
             (content, repeat_rule, sort_order, remind_at, daily_time_minutes, weekly_days, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
             VALUES (?1, 'WEEKLY', ?2, NULL, ?3, ?4, 0, NULL, NULL, NULL, ?5, ?5)",
            params![content, next_sort_order, minutes, weekly_days_db, now_ts],
        )
        .map_err(|e| format!("failed to create weekly reminder: {e}"))?;
        return Ok(conn.last_insert_rowid());
    }

    let remind_at = input.remind_at.map(|v| v.max(0));
    if let Some(id) = input.id {
        let changed = conn
            .execute(
                "UPDATE reminders
                 SET content = ?1,
                     repeat_rule = 'NONE',
                     remind_at = ?2,
                     daily_time_minutes = NULL,
                     weekly_days = NULL,
                     snooze_until = NULL,
                     updated_at = ?3
                  WHERE id = ?4",
                params![content, remind_at, now_ts, id],
            )
            .map_err(|e| format!("failed to update reminder: {e}"))?;
        if changed == 0 {
            return Err("reminder not found".to_string());
        }
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO reminders
         (content, repeat_rule, sort_order, remind_at, daily_time_minutes, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
         VALUES (?1, 'NONE', ?2, ?3, NULL, 0, NULL, NULL, NULL, ?4, ?4)",
        params![content, next_sort_order, remind_at, now_ts],
    )
    .map_err(|e| format!("failed to create reminder: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn delete_reminder(app: AppHandle, id: i64) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    let changed = conn
        .execute("DELETE FROM reminders WHERE id = ?1", [id])
        .map_err(|e| format!("failed to delete reminder: {e}"))?;
    Ok(changed > 0)
}

#[tauri::command]
fn set_reminder_done(app: AppHandle, input: SetReminderDoneInput) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    let repeat_rule_raw = conn
        .query_row(
            "SELECT repeat_rule FROM reminders WHERE id = ?1",
            [input.id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "reminder not found".to_string())?;
    let repeat_rule = normalize_repeat_rule(&repeat_rule_raw)?;
    let now_ts = Local::now().timestamp();

    if repeat_rule == "DAILY" || repeat_rule == "WEEKLY" {
        let today_start_ts = business_day_start_from_local(Local::now())?;
        let today_key = business_day_key_from_start(today_start_ts)?;
        let changed = if input.done {
            conn.execute(
                "UPDATE reminders
                 SET completed_day_key = ?1,
                     completed_at = ?2,
                     snooze_until = NULL,
                     updated_at = ?2
                 WHERE id = ?3",
                params![today_key, now_ts, input.id],
            )
        } else {
            conn.execute(
                "UPDATE reminders
                 SET completed_day_key = NULL,
                     completed_at = NULL,
                     updated_at = ?1
                 WHERE id = ?2",
                params![now_ts, input.id],
            )
        }
        .map_err(|e| format!("failed to toggle recurring reminder completion: {e}"))?;
        return Ok(changed > 0);
    }

    let changed = conn
        .execute(
            "UPDATE reminders
             SET is_completed = ?1,
                 completed_at = CASE WHEN ?1 = 1 THEN ?2 ELSE NULL END,
                 snooze_until = CASE WHEN ?1 = 1 THEN NULL ELSE snooze_until END,
                 updated_at = ?2
             WHERE id = ?3",
            params![if input.done { 1 } else { 0 }, now_ts, input.id],
        )
        .map_err(|e| format!("failed to toggle reminder completion: {e}"))?;
    Ok(changed > 0)
}

#[tauri::command]
fn set_reminder_order(app: AppHandle, input: SetReminderOrderInput) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    if input.ordered_ids.is_empty() {
        return Ok(true);
    }

    let now_ts = Local::now().timestamp();
    for (index, id) in input.ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE reminders SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![index as i64, now_ts, id],
        )
        .map_err(|e| format!("failed to update reminder sort order: {e}"))?;
    }
    Ok(true)
}

#[tauri::command]
fn snooze_reminder(app: AppHandle, id: i64, snooze_seconds: Option<i64>) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    let now_ts = Local::now().timestamp();
    let seconds = snooze_seconds.unwrap_or(600).clamp(60, 86_400);
    let snooze_until = now_ts + seconds;
    let changed = conn
        .execute(
            "UPDATE reminders
             SET snooze_until = ?1,
                 updated_at = ?2
             WHERE id = ?3",
            params![snooze_until, now_ts, id],
        )
        .map_err(|e| format!("failed to snooze reminder: {e}"))?;
    Ok(changed > 0)
}

#[tauri::command]
fn list_categories(app: AppHandle) -> Result<Vec<Category>, String> {
    let conn = open_connection(&app)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, parent_id, name, root_type, color_hex
             FROM categories
             ORDER BY id ASC",
        )
        .map_err(|e| format!("failed to prepare categories query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                parent_id: row.get(1)?,
                name: row.get(2)?,
                root_type: row.get(3)?,
                color_hex: row.get(4)?,
            })
        })
        .map_err(|e| format!("failed to read categories: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to decode category row: {e}"))?);
    }

    Ok(result)
}

#[tauri::command]
fn create_category(app: AppHandle, input: CreateCategoryInput) -> Result<i64, String> {
    let conn = open_connection(&app)?;

    let root_type: String = conn
        .query_row(
            "SELECT root_type FROM categories WHERE id = ?1",
            [input.parent_id],
            |row| row.get(0),
        )
        .map_err(|_| "parent category not found".to_string())?;

    conn.execute(
        "INSERT INTO categories (parent_id, name, color_hex, root_type) VALUES (?1, ?2, ?3, ?4)",
        params![
            input.parent_id,
            input.name.trim(),
            input
                .color_hex
                .unwrap_or_else(|| if root_type == "LEARN" { "#4ade80" } else { "#fb923c" }.to_string()),
            root_type
        ],
    )
    .map_err(|e| format!("failed to create category: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn start_session(app: AppHandle, category_id: i64) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    let now_ts = Local::now().timestamp();

    conn.execute(
        "UPDATE task_sessions SET end_time = ?1 WHERE end_time IS NULL",
        [now_ts],
    )
    .map_err(|e| format!("failed to close previous active session: {e}"))?;

    conn.execute(
        "INSERT INTO task_sessions (category_id, start_time, end_time, is_flow_target) VALUES (?1, ?2, NULL, 0)",
        params![category_id, now_ts],
    )
    .map_err(|e| format!("failed to start session: {e}"))?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
fn stop_active_session(app: AppHandle) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    let now_ts = Local::now().timestamp();
    let changed = conn
        .execute(
            "UPDATE task_sessions SET end_time = ?1 WHERE end_time IS NULL",
            [now_ts],
        )
        .map_err(|e| format!("failed to stop active session: {e}"))?;

    Ok(changed > 0)
}

#[tauri::command]
fn append_app_usage_log(
    app: AppHandle,
    process_name: String,
    window_title: String,
    start_timestamp: i64,
    duration_ms: i64,
) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    let (stored, _) = append_usage_log_record(&conn, &process_name, &window_title, start_timestamp, duration_ms)?;
    Ok(stored)
}

#[tauri::command]
fn capture_foreground_once(app: AppHandle, duration_ms: Option<i64>) -> Result<bool, String> {
    let now_ms = Local::now().timestamp_millis();
    let idle_ms = current_idle_millis().unwrap_or(0).max(0);

    if idle_ms >= IDLE_PROMPT_THRESHOLD_MS {
        let idle_start_ms = now_ms - idle_ms;
        {
            let mut state = FOREGROUND_SAMPLE_STATE
                .lock()
                .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
            let seg_start = state.idle_segment_start_ms.get_or_insert(idle_start_ms);
            if idle_start_ms < *seg_start {
                *seg_start = idle_start_ms;
            }
            state.last = None;
        }

        let diag = ForegroundCaptureDiagnostic {
            id: now_ms,
            captured_at_ms: now_ms,
            observed_process_name: "system.idle".to_string(),
            observed_window_title: "Idle Tracking".to_string(),
            stored: false,
            block_reason: Some("system_idle_tracking".to_string()),
            rule_saved: true,
            rule_mapped_type: "IGNORE".to_string(),
        };
        let _ = push_foreground_diagnostic(diag);
        return Ok(false);
    }

    {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        if let Some(idle_start_ms) = state.idle_segment_start_ms.take() {
            let duration = (now_ms - idle_start_ms).max(0);
            if duration >= IDLE_PROMPT_THRESHOLD_MS {
                let prompt_id = if state.next_idle_prompt_id <= 0 {
                    1
                } else {
                    state.next_idle_prompt_id
                };
                state.next_idle_prompt_id = prompt_id + 1;
                let prompt = IdlePromptEntry {
                    id: prompt_id,
                    start_timestamp: (idle_start_ms / 1000).max(0),
                    end_timestamp: (now_ms / 1000).max(0),
                    duration_ms: duration,
                    deferred_until_timestamp: None,
                };

                if let Some(remembered) = state.remembered_idle_decision.clone() {
                    drop(state);
                    let conn = open_connection(&app)?;
                    if persist_idle_prompt_decision(&conn, &prompt, &remembered).is_err() {
                        let mut fallback_state = FOREGROUND_SAMPLE_STATE
                            .lock()
                            .map_err(|e| format!("failed to relock foreground sample state: {e}"))?;
                        fallback_state.pending_idle_prompts.push(prompt);
                        if fallback_state.pending_idle_prompts.len() > 20 {
                            let drop_count = fallback_state.pending_idle_prompts.len() - 20;
                            fallback_state.pending_idle_prompts.drain(0..drop_count);
                        }
                    }
                } else {
                    state.pending_idle_prompts.push(prompt);
                    if state.pending_idle_prompts.len() > 20 {
                        let drop_count = state.pending_idle_prompts.len() - 20;
                        state.pending_idle_prompts.drain(0..drop_count);
                    }
                }
            }
        }
    }

    let Some((process_name, window_title)) = capture_foreground_window()? else {
        {
            let mut state = FOREGROUND_SAMPLE_STATE
                .lock()
                .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
            state.last = None;
        }
        let diag = ForegroundCaptureDiagnostic {
            id: now_ms,
            captured_at_ms: now_ms,
            observed_process_name: "unknown.exe".to_string(),
            observed_window_title: "N/A".to_string(),
            stored: false,
            block_reason: Some("no_foreground_window".to_string()),
            rule_saved: false,
            rule_mapped_type: "IGNORE".to_string(),
        };
        let _ = push_foreground_diagnostic(diag);
        return Ok(false);
    };

    let current = ForegroundSnapshot {
        process_name,
        window_title,
        captured_at_ms: now_ms,
    };

    let previous = {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        let prev = state.last.take();
        state.last = Some(current.clone());
        prev
    };

    let Some(prev) = previous else {
        // First sample only establishes a baseline timestamp to avoid synthetic +5s overcount.
        let normalized = normalize_process_key(&current.process_name);
        let conn = open_connection(&app)?;
        let (rule_saved, rule_mapped_type) = resolve_rule_mapping(&conn, &normalized);
        let diag = ForegroundCaptureDiagnostic {
            id: now_ms,
            captured_at_ms: now_ms,
            observed_process_name: normalized,
            observed_window_title: current.window_title.clone(),
            stored: false,
            block_reason: Some("baseline_only".to_string()),
            rule_saved,
            rule_mapped_type,
        };
        let _ = push_foreground_diagnostic(diag);
        return Ok(false);
    };

    let elapsed_ms = (now_ms - prev.captured_at_ms).max(0);
    if elapsed_ms <= 0 {
        let normalized = normalize_process_key(&prev.process_name);
        let conn = open_connection(&app)?;
        let (rule_saved, rule_mapped_type) = resolve_rule_mapping(&conn, &normalized);
        let diag = ForegroundCaptureDiagnostic {
            id: now_ms,
            captured_at_ms: now_ms,
            observed_process_name: normalized,
            observed_window_title: prev.window_title.clone(),
            stored: false,
            block_reason: Some("elapsed_too_short".to_string()),
            rule_saved,
            rule_mapped_type,
        };
        let _ = push_foreground_diagnostic(diag);
        return Ok(false);
    }

    let max_span = duration_ms.unwrap_or(60000).clamp(500, 60000);
    let span = elapsed_ms.min(max_span).max(500);
    let start_ts = ((now_ms - span) / 1000).max(0);

    let conn = open_connection(&app)?;
    let normalized = normalize_process_key(&prev.process_name);
    let (rule_saved, rule_mapped_type) = resolve_rule_mapping(&conn, &normalized);

    let (stored, block_reason) = append_usage_log_record(
        &conn,
        &prev.process_name,
        &prev.window_title,
        start_ts,
        span,
    )?;

    let diag = ForegroundCaptureDiagnostic {
        id: now_ms,
        captured_at_ms: now_ms,
        observed_process_name: normalized,
        observed_window_title: prev.window_title,
        stored,
        block_reason,
        rule_saved,
        rule_mapped_type,
    };
    let _ = push_foreground_diagnostic(diag);

    Ok(stored)
}

#[tauri::command]
fn list_pending_idle_prompts(limit: Option<i64>) -> Result<Vec<IdlePromptEntry>, String> {
    let cap = limit.unwrap_or(5).clamp(1, 20) as usize;
    let now_ts = Local::now().timestamp();
    let state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;

    let mut items: Vec<IdlePromptEntry> = state
        .pending_idle_prompts
        .iter()
        .filter(|item| {
            item.deferred_until_timestamp
                .map(|until| until <= now_ts)
                .unwrap_or(true)
        })
        .rev()
        .take(cap)
        .cloned()
        .collect();
    items.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(items)
}

#[tauri::command]
fn resolve_idle_prompt(app: AppHandle, input: ResolveIdlePromptInput) -> Result<bool, String> {
    let decision = input.decision.trim().to_uppercase();
    let remember_this_session = input.remember_this_session.unwrap_or(false);
    if !matches!(decision.as_str(), "LEARN" | "REST" | "IDLE" | "SKIP") {
        return Err("decision must be LEARN, REST, IDLE, or SKIP".to_string());
    }

    if decision == "SKIP" {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        let now_ts = Local::now().timestamp();
        let maybe_item = state
            .pending_idle_prompts
            .iter_mut()
            .find(|item| item.id == input.prompt_id);
        if let Some(item) = maybe_item {
            item.deferred_until_timestamp = Some(now_ts + 1800);
            return Ok(true);
        }
        return Ok(false);
    }

    let prompt = {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        let index = state
            .pending_idle_prompts
            .iter()
            .position(|item| item.id == input.prompt_id);
        index.map(|idx| state.pending_idle_prompts.remove(idx))
    };

    let Some(prompt) = prompt else {
        return Ok(false);
    };
    let conn = open_connection(&app)?;
    let stored = persist_idle_prompt_decision(&conn, &prompt, &decision)?;

    if remember_this_session {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        state.remembered_idle_decision = Some(decision);
    }

    Ok(stored)
}

#[tauri::command]
fn get_idle_memory_state() -> Result<IdleMemoryState, String> {
    let state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
    Ok(IdleMemoryState {
        remembered_decision: state.remembered_idle_decision.clone(),
    })
}

#[tauri::command]
fn clear_idle_memory_state() -> Result<(), String> {
    let mut state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
    state.remembered_idle_decision = None;
    Ok(())
}

#[tauri::command]
fn list_recent_logs(app: AppHandle, limit: Option<i64>) -> Result<Vec<RecentLogEntry>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(12).clamp(1, 100);
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, process_name, window_title, start_timestamp, duration_ms
             FROM app_usage_logs
             WHERE start_timestamp >= ?1 AND start_timestamp < ?2
             ORDER BY id DESC
             LIMIT ?3",
        )
        .map_err(|e| format!("failed to prepare recent logs query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts, cap], |row| {
            Ok(RecentLogEntry {
                id: row.get(0)?,
                process_name: row.get(1)?,
                window_title: row.get(2)?,
                start_timestamp: row.get(3)?,
                duration_ms: row.get(4)?,
            })
        })
        .map_err(|e| format!("failed to query recent logs rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse recent log row: {e}"))?);
    }

    Ok(result)
}

#[tauri::command]
fn list_foreground_capture_diagnostics(
    limit: Option<i64>,
    unique_by_process: Option<bool>,
) -> Result<Vec<ForegroundCaptureDiagnostic>, String> {
    let cap = limit.unwrap_or(10).clamp(1, 60) as usize;
    let unique = unique_by_process.unwrap_or(true);
    let state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;

    let mut items: Vec<ForegroundCaptureDiagnostic> = if unique {
        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut deduped = Vec::new();
        for item in state.diagnostics.iter().rev() {
            if seen.contains_key(&item.observed_process_name) {
                continue;
            }
            seen.insert(item.observed_process_name.clone(), true);
            deduped.push(item.clone());
            if deduped.len() >= cap {
                break;
            }
        }
        deduped
    } else {
        state
            .diagnostics
            .iter()
            .rev()
            .take(cap)
            .cloned()
            .collect()
    };

    items.sort_by(|a, b| {
        let a_unsaved = !a.rule_saved;
        let b_unsaved = !b.rule_saved;
        let a_unstored = !a.stored;
        let b_unstored = !b.stored;

        b_unsaved
            .cmp(&a_unsaved)
            .then_with(|| b_unstored.cmp(&a_unstored))
            .then_with(|| b.captured_at_ms.cmp(&a.captured_at_ms))
    });
    Ok(items)
}

#[tauri::command]
fn list_app_rules(app: AppHandle, limit: Option<i64>) -> Result<Vec<AppRuleEntry>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(200).clamp(1, 1000);

    let mut stmt = conn
        .prepare(
            "SELECT process_name, mapped_type, privacy_level, updated_at
             FROM app_rules
             ORDER BY process_name ASC
             LIMIT ?1",
        )
        .map_err(|e| format!("failed to prepare app rules query: {e}"))?;

    let rows = stmt
        .query_map([cap], |row| {
            Ok(AppRuleEntry {
                process_name: row.get(0)?,
                mapped_type: row.get(1)?,
                privacy_level: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })
        .map_err(|e| format!("failed to query app rules rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse app rule row: {e}"))?);
    }
    Ok(result)
}

#[tauri::command]
fn list_pending_rule_processes(app: AppHandle, limit: Option<i64>) -> Result<Vec<PendingRuleProcess>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(10).clamp(1, 200);

    let mut stmt = conn
        .prepare(
            "SELECT l.process_name,
                    MAX(l.start_timestamp) AS last_seen_timestamp,
                    SUM(l.duration_ms) / 1000 AS total_seconds,
                    (SELECT ll.window_title
                     FROM app_usage_logs ll
                     WHERE ll.process_name = l.process_name
                     ORDER BY ll.start_timestamp DESC, ll.id DESC
                     LIMIT 1) AS last_window_title
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE r.process_name IS NULL
             GROUP BY l.process_name
               HAVING SUM(l.duration_ms) >= 180000
             ORDER BY last_seen_timestamp DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("failed to prepare pending rule processes query: {e}"))?;

    let rows = stmt
        .query_map([cap], |row| {
            Ok(PendingRuleProcess {
                process_name: row.get(0)?,
                last_seen_timestamp: row.get::<_, i64>(1)?.max(0),
                total_seconds: row.get::<_, i64>(2)?.max(0),
                last_window_title: row
                    .get::<_, Option<String>>(3)?
                    .unwrap_or_else(|| "Untitled Window".to_string()),
            })
        })
        .map_err(|e| format!("failed to query pending rule processes rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse pending rule process row: {e}"))?);
    }
    Ok(result)
}

#[tauri::command]
fn get_today_summary(app: AppHandle) -> Result<TodaySummary, String> {
    let conn = open_connection(&app)?;
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;
    let mut learn_seconds = 0_i64;
    let mut rest_seconds = 0_i64;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(r.mapped_type, 'IGNORE') AS mapped_type,
                    SUM(
                        MAX(
                            0,
                            MIN(l.start_timestamp + (l.duration_ms / 1000), ?2) - MAX(l.start_timestamp, ?1)
                        )
                    ) AS duration_seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1
             GROUP BY mapped_type",
        )
        .map_err(|e| format!("failed to prepare summary query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            let root_type: String = row.get(0)?;
            let duration_seconds: i64 = row.get(1)?;
            Ok((root_type, duration_seconds.max(0)))
        })
        .map_err(|e| format!("failed to query summary rows: {e}"))?;

    for row in rows {
        let (root_type, duration_seconds) =
            row.map_err(|e| format!("failed to parse summary row: {e}"))?;
        if root_type == "LEARN" {
            learn_seconds = duration_seconds;
        } else if root_type == "REST" {
            rest_seconds = duration_seconds;
        }
    }

    Ok(TodaySummary {
        learn_seconds,
        rest_seconds,
        active_session_id: None,
    })
}

#[tauri::command]
fn list_top_apps_today(app: AppHandle, limit: Option<i64>) -> Result<Vec<TopApp>, String> {
    let conn = open_connection(&app)?;
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;

    let cap = limit.unwrap_or(5).clamp(1, 20);

    let mut stmt = conn
        .prepare(
            "SELECT l.process_name,
                    SUM(
                        MAX(
                            0,
                            MIN(l.start_timestamp + (l.duration_ms / 1000), ?2) - MAX(l.start_timestamp, ?1)
                        )
                    ) AS seconds
             FROM app_usage_logs l
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1
             GROUP BY process_name
             ORDER BY seconds DESC
             LIMIT ?3",
        )
        .map_err(|e| format!("failed to prepare top apps query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts, cap], |row| {
            Ok(TopApp {
                process_name: row.get(0)?,
                seconds: row.get::<_, i64>(1)?.max(0),
            })
        })
        .map_err(|e| format!("failed to query top apps rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse top app row: {e}"))?);
    }

    Ok(result)
}

#[tauri::command]
fn list_top_apps_all_time(
    app: AppHandle,
    limit: Option<i64>,
    root_filter: Option<String>,
    include_ignore: Option<bool>,
) -> Result<Vec<TopApp>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(10).clamp(1, 30);
    let keep_ignore = include_ignore.unwrap_or(true);
    let filter = root_filter
        .unwrap_or_else(|| "ALL".to_string())
        .trim()
        .to_uppercase();

    let (sql, bind_filter): (&str, Option<String>) = match filter.as_str() {
        "LEARN" => (
            "SELECT l.process_name,
                    SUM(MAX(0, l.duration_ms / 1000)) AS seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = ?1
             GROUP BY l.process_name
             ORDER BY seconds DESC
             LIMIT ?2",
            Some("LEARN".to_string()),
        ),
        "REST" => (
            "SELECT l.process_name,
                    SUM(MAX(0, l.duration_ms / 1000)) AS seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = ?1
             GROUP BY l.process_name
             ORDER BY seconds DESC
             LIMIT ?2",
            Some("REST".to_string()),
        ),
        _ => {
            if keep_ignore {
                (
                    "SELECT l.process_name,
                            SUM(MAX(0, l.duration_ms / 1000)) AS seconds
                     FROM app_usage_logs l
                     GROUP BY l.process_name
                     ORDER BY seconds DESC
                     LIMIT ?1",
                    None,
                )
            } else {
                (
                    "SELECT l.process_name,
                            SUM(MAX(0, l.duration_ms / 1000)) AS seconds
                     FROM app_usage_logs l
                     LEFT JOIN app_rules r ON r.process_name = l.process_name
                     WHERE COALESCE(r.mapped_type, 'IGNORE') != 'IGNORE'
                     GROUP BY l.process_name
                     ORDER BY seconds DESC
                     LIMIT ?1",
                    None,
                )
            }
        }
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("failed to prepare all-time top apps query: {e}"))?;

    if let Some(mapped_type) = bind_filter {
        let rows = stmt
            .query_map(params![mapped_type, cap], |row| {
                Ok(TopApp {
                    process_name: row.get(0)?,
                    seconds: row.get::<_, i64>(1)?.max(0),
                })
            })
            .map_err(|e| format!("failed to query filtered all-time top apps rows: {e}"))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(
                row.map_err(|e| format!("failed to parse filtered all-time top app row: {e}"))?,
            );
        }
        return Ok(result);
    }

    let rows = stmt
        .query_map([cap], |row| {
            Ok(TopApp {
                process_name: row.get(0)?,
                seconds: row.get::<_, i64>(1)?.max(0),
            })
        })
        .map_err(|e| format!("failed to query all-time top apps rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse all-time top app row: {e}"))?);
    }

    Ok(result)
}

#[tauri::command]
fn get_learn_heatmap(
    app: AppHandle,
    days: Option<i64>,
    goal_seconds: Option<i64>,
) -> Result<Vec<LearnHeatmapCell>, String> {
    let conn = open_connection(&app)?;
    ensure_heatmap_snapshot_table(&conn)?;
    let span_days = days.unwrap_or(35).clamp(7, 2000);
    let goal = goal_seconds.unwrap_or(7200).clamp(0, 86400);
    let lock_goal = parse_i64_config(&conn, "heatmap_lock_goal_seconds", HEATMAP_LOCK_GOAL_SECONDS)
        .clamp(0, 86_400);

    let (today_start_ts, today_end_ts) = business_day_window_from_local(Local::now())?;
    let start_ts = today_start_ts - (span_days - 1) * 86_400;
    let end_ts = today_end_ts;

    for i in 0..span_days {
        let day_start_ts = start_ts + i * 86_400;
        if day_start_ts >= today_start_ts {
            continue;
        }
        seal_historical_heatmap_snapshot(&conn, day_start_ts, lock_goal)?;
    }

    let start_day_key = business_day_key_from_start(start_ts)?;
    let end_day_key = business_day_key_from_start(end_ts - 1)?;
    let mut snap_stmt = conn
        .prepare(
            "SELECT day_key, learn_seconds, level
             FROM daily_heatmap_snapshot
             WHERE day_key >= ?1 AND day_key <= ?2",
        )
        .map_err(|e| format!("failed to prepare heatmap snapshot query: {e}"))?;

    let snapshot_rows = snap_stmt
        .query_map(params![start_day_key, end_day_key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("failed to query heatmap snapshots: {e}"))?;

    let mut snapshots: HashMap<String, (i64, String)> = HashMap::new();
    for row in snapshot_rows {
        let (day_key, seconds, level) = row.map_err(|e| format!("failed to parse heatmap snapshot row: {e}"))?;
        snapshots.insert(day_key, (seconds, level));
    }

    let mut stmt = conn
        .prepare(
            "SELECT l.start_timestamp,
                    l.duration_ms,
                    COALESCE(r.mapped_type, 'IGNORE') AS mapped_type
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare heatmap query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?.to_uppercase(),
            ))
        })
        .map_err(|e| format!("failed to query heatmap rows: {e}"))?;

    let mut by_day: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let (seg_start, duration_ms, mapped_type) =
            row.map_err(|e| format!("failed to parse heatmap row: {e}"))?;
        if mapped_type != "LEARN" {
            continue;
        }

        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end <= clip_start {
            continue;
        }

        let mut cursor = clip_start;
        while cursor < clip_end {
            let bucket_start = business_day_start_for_timestamp(cursor)?;
            let bucket_end = bucket_start + 86_400;
            let piece_end = clip_end.min(bucket_end);
            let piece_seconds = overlap_seconds(cursor, piece_end, bucket_start, bucket_end);
            if piece_seconds > 0 {
                let day_key = business_day_key_from_start(bucket_start)?;
                *by_day.entry(day_key).or_insert(0) += piece_seconds;
            }
            cursor = piece_end;
        }
    }

    let mut result = Vec::with_capacity(span_days as usize);
    for i in 0..span_days {
        let day_start_ts = start_ts + i * 86_400;
        let day_key = business_day_key_from_start(day_start_ts)?;
        let (learn_seconds, level) = if day_start_ts < today_start_ts {
            if let Some((seconds, locked_level)) = snapshots.get(&day_key) {
                (*seconds, locked_level.clone())
            } else {
                (0, "GRAY".to_string())
            }
        } else {
            let seconds = compute_learn_seconds_for_window(&conn, day_start_ts, day_start_ts + 86_400)?
                .max(0);
            let lv = if seconds <= 0 {
                "GRAY"
            } else if seconds < goal {
                "YELLOW"
            } else {
                "GREEN"
            }
            .to_string();
            (seconds, lv)
        };

        result.push(LearnHeatmapCell {
            day: day_key,
            learn_seconds,
            level,
        });
    }

    Ok(result)
}

#[tauri::command]
fn get_heatmap_goal_seconds_setting(app: AppHandle) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    let value = parse_i64_config(&conn, "heatmap_goal_seconds", 7_200).clamp(0, 86_400);
    Ok(value)
}

#[tauri::command]
fn set_heatmap_goal_seconds_setting(app: AppHandle, goal_seconds: i64) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    let normalized = goal_seconds.clamp(0, 86_400);
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES ('heatmap_goal_seconds', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [normalized.to_string()],
    )
    .map_err(|e| format!("failed to save heatmap goal seconds: {e}"))?;
    Ok(normalized)
}

#[tauri::command]
fn get_usage_stack(
    app: AppHandle,
    days: Option<i64>,
    root_filter: Option<String>,
) -> Result<Vec<UsageStackDay>, String> {
    let conn = open_connection(&app)?;
    let span_days = days.unwrap_or(14).clamp(3, 366);
    let filter = root_filter
        .unwrap_or_else(|| "ALL".to_string())
        .trim()
        .to_uppercase();
    if !matches!(filter.as_str(), "ALL" | "LEARN" | "REST") {
        return Err("root_filter must be ALL, LEARN, or REST".to_string());
    }

    let (today_start_ts, today_end_ts) = business_day_window_from_local(Local::now())?;
    let start_ts = today_start_ts - (span_days - 1) * 86_400;
    let end_ts = today_end_ts;

    let mut stmt = conn
        .prepare(
            "SELECT l.start_timestamp,
                    l.duration_ms,
                    COALESCE(r.mapped_type, 'IGNORE') AS mapped_type,
                    l.process_name
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare usage stack query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?.to_uppercase(),
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("failed to query usage stack rows: {e}"))?;

    #[derive(Default)]
    struct DayAcc {
        total_intervals: Vec<(i64, i64)>,
        learn_intervals: Vec<(i64, i64)>,
        rest_intervals: Vec<(i64, i64)>,
        segments: HashMap<String, i64>,
    }

    let mut by_day: BTreeMap<String, DayAcc> = BTreeMap::new();
    for row in rows {
        let (seg_start, duration_ms, mapped_type, process_name) =
            row.map_err(|e| format!("failed to parse usage stack row: {e}"))?;
        if duration_ms <= 0 {
            continue;
        }

        let include = match filter.as_str() {
            "LEARN" => mapped_type == "LEARN",
            "REST" => mapped_type == "REST",
            _ => true,
        };
        if !include {
            continue;
        }

        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end <= clip_start {
            continue;
        }

        let mut cursor = clip_start;
        while cursor < clip_end {
            let bucket_start = business_day_start_for_timestamp(cursor)?;
            let bucket_end = bucket_start + 86_400;
            let piece_end = clip_end.min(bucket_end);
            let seconds = overlap_seconds(cursor, piece_end, bucket_start, bucket_end);
            if seconds <= 0 {
                cursor = piece_end;
                continue;
            }

            let day = business_day_key_from_start(bucket_start)?;

            let day_entry = by_day.entry(day).or_default();
            day_entry.total_intervals.push((cursor, piece_end));
            if mapped_type == "LEARN" {
                day_entry.learn_intervals.push((cursor, piece_end));
            } else if mapped_type == "REST" {
                day_entry.rest_intervals.push((cursor, piece_end));
            }

            let seg_name = if mapped_type == "IGNORE" {
                format!("{process_name} (IGNORE)")
            } else {
                process_name.clone()
            };
            *day_entry.segments.entry(seg_name).or_insert(0) += seconds;
            cursor = piece_end;
        }
    }

    let mut result = Vec::new();
    for (i, (day, acc)) in by_day.into_iter().enumerate() {
        if i >= span_days as usize {
            break;
        }

        let mut segments: Vec<UsageStackSegment> = acc
            .segments
            .into_iter()
            .map(|(name, seconds)| UsageStackSegment { name, seconds })
            .collect();
        segments.sort_by(|a, b| b.seconds.cmp(&a.seconds));

        result.push(UsageStackDay {
            day,
            total_seconds: merge_intervals_total(acc.total_intervals).max(0),
            learn_seconds: merge_intervals_total(acc.learn_intervals).max(0),
            rest_seconds: merge_intervals_total(acc.rest_intervals).max(0),
            segments,
        });
    }

    // Return latest day first so callers can read index 0 as current business day.
    result.reverse();

    Ok(result)
}

#[tauri::command]
fn save_app_rule(app: AppHandle, input: SaveAppRuleInput) -> Result<(), String> {
    let process_name = normalize_process_key(&input.process_name);
    if process_name.is_empty() {
        return Err("process_name cannot be empty".to_string());
    }

    let mapped_type = input.mapped_type.trim().to_uppercase();
    if mapped_type != "LEARN" && mapped_type != "REST" && mapped_type != "IGNORE" {
        return Err("mapped_type must be LEARN, REST, or IGNORE".to_string());
    }

    let privacy_level = input
        .privacy_level
        .unwrap_or_else(|| "NORMAL".to_string())
        .trim()
        .to_uppercase();

    if privacy_level != "NORMAL" && privacy_level != "BLUR_TITLE" && privacy_level != "WHITELIST_ONLY" {
        return Err("privacy_level must be NORMAL, BLUR_TITLE, or WHITELIST_ONLY".to_string());
    }

    let conn = open_connection(&app)?;
    upsert_app_rule_entry(&conn, &process_name, &mapped_type, &privacy_level)?;

    Ok(())
}

#[tauri::command]
fn check_focus_deviation(
    app: AppHandle,
    process_name: String,
    debounce_seconds: Option<i64>,
) -> Result<DeviationCheck, String> {
    let now_ts = Local::now().timestamp();
    let deb = debounce_seconds.unwrap_or(60).clamp(5, 600);
    let process_name = normalize_process_key(&process_name);

    let conn = open_connection(&app)?;
    let current_active_root = active_root_type(&conn)?;

    if current_active_root.is_none() {
        if let Ok(mut state) = DEVIATION_STATE.lock() {
            state.mismatch_since = None;
            state.pending_alert = false;
        }
        return Ok(DeviationCheck {
            triggered: false,
            process_name,
            reason: "no_active_session".to_string(),
            active_root_type: None,
            mapped_type: None,
            suggested_root_category_id: None,
        });
    }

    let active_root = current_active_root.unwrap_or_else(|| "LEARN".to_string());
    let mapped_type = conn
        .query_row(
            "SELECT mapped_type FROM app_rules WHERE process_name = ?1",
            [process_name.clone()],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "IGNORE".to_string());

    let mut state = DEVIATION_STATE
        .lock()
        .map_err(|e| format!("failed to lock deviation state: {e}"))?;

    if now_ts < state.cooldown_until {
        return Ok(DeviationCheck {
            triggered: false,
            process_name,
            reason: "cooldown".to_string(),
            active_root_type: Some(active_root),
            mapped_type: Some(mapped_type),
            suggested_root_category_id: None,
        });
    }

    if mapped_type == "IGNORE" || mapped_type == active_root {
        state.mismatch_since = None;
        state.pending_alert = false;
        return Ok(DeviationCheck {
            triggered: false,
            process_name,
            reason: "matched_or_ignored".to_string(),
            active_root_type: Some(active_root),
            mapped_type: Some(mapped_type),
            suggested_root_category_id: None,
        });
    }

    if state.pending_alert {
        return Ok(DeviationCheck {
            triggered: false,
            process_name,
            reason: "awaiting_user_action".to_string(),
            active_root_type: Some(active_root),
            mapped_type: Some(mapped_type),
            suggested_root_category_id: None,
        });
    }

    let since = match state.mismatch_since {
        Some(since) => since,
        None => {
            state.mismatch_since = Some(now_ts);
            return Ok(DeviationCheck {
                triggered: false,
                process_name,
                reason: "debounce_started".to_string(),
                active_root_type: Some(active_root),
                mapped_type: Some(mapped_type),
                suggested_root_category_id: None,
            });
        }
    };

    if now_ts - since < deb {
        return Ok(DeviationCheck {
            triggered: false,
            process_name,
            reason: "debouncing".to_string(),
            active_root_type: Some(active_root),
            mapped_type: Some(mapped_type),
            suggested_root_category_id: None,
        });
    }

    state.pending_alert = true;
    Ok(DeviationCheck {
        triggered: true,
        process_name,
        reason: "deviation_detected".to_string(),
        active_root_type: Some(active_root),
        mapped_type: Some(mapped_type.clone()),
        suggested_root_category_id: if mapped_type == "LEARN" { Some(1) } else { Some(2) },
    })
}

#[tauri::command]
fn snooze_focus_guard(cooldown_seconds: Option<i64>) -> Result<(), String> {
    let now_ts = Local::now().timestamp();
    let cooldown = cooldown_seconds.unwrap_or(900).clamp(60, 7200);
    let mut state = DEVIATION_STATE
        .lock()
        .map_err(|e| format!("failed to lock deviation state: {e}"))?;
    state.pending_alert = false;
    state.mismatch_since = None;
    state.cooldown_until = now_ts + cooldown;
    Ok(())
}

#[tauri::command]
fn get_privacy_settings(app: AppHandle) -> Result<PrivacySettings, String> {
    let conn = open_connection(&app)?;
    Ok(PrivacySettings {
        curtain_enabled: parse_bool_config(&conn, "curtain_enabled", false),
        browser_title_mode: parse_browser_title_mode(&conn),
        whitelist_only_enabled: parse_bool_config(&conn, "whitelist_only_enabled", false),
    })
}

#[tauri::command]
fn update_privacy_settings(app: AppHandle, input: UpdatePrivacySettingsInput) -> Result<(), String> {
    let conn = open_connection(&app)?;
    let browser_title_mode = input.browser_title_mode.trim().to_uppercase();
    if !matches!(browser_title_mode.as_str(), "FULL" | "BLUR" | "NONE") {
        return Err("browser_title_mode must be FULL, BLUR, or NONE".to_string());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("failed to begin privacy settings transaction: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('curtain_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if input.curtain_enabled { "true" } else { "false" }],
    )
    .map_err(|e| format!("failed to save curtain setting: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('browser_title_mode', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [browser_title_mode.as_str()],
    )
    .map_err(|e| format!("failed to save browser title mode: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('browser_blur_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if browser_title_mode == "FULL" { "false" } else { "true" }],
    )
    .map_err(|e| format!("failed to save browser blur setting: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('whitelist_only_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if input.whitelist_only_enabled {
            "true"
        } else {
            "false"
        }],
    )
    .map_err(|e| format!("failed to save whitelist-only setting: {e}"))?;

    tx.commit()
        .map_err(|e| format!("failed to commit privacy settings transaction: {e}"))?;
    Ok(())
}

#[tauri::command]
fn get_auto_start_enabled() -> Result<bool, String> {
    query_auto_start_enabled_internal()
}

#[tauri::command]
fn set_auto_start_enabled(enabled: bool) -> Result<bool, String> {
    set_auto_start_enabled_internal(enabled)
}

#[tauri::command]
fn list_whitelist(app: AppHandle) -> Result<Vec<String>, String> {
    let conn = open_connection(&app)?;
    let mut stmt = conn
        .prepare("SELECT process_name FROM app_whitelist ORDER BY process_name")
        .map_err(|e| format!("failed to prepare whitelist query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("failed to query whitelist rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse whitelist row: {e}"))?);
    }
    Ok(result)
}

#[tauri::command]
fn set_whitelist_item(app: AppHandle, input: SetWhitelistItemInput) -> Result<(), String> {
    let conn = open_connection(&app)?;
    let process_name = normalize_process_key(&input.process_name);
    if process_name.is_empty() {
        return Err("process_name cannot be empty".to_string());
    }

    if input.enabled {
        conn.execute(
            "INSERT OR IGNORE INTO app_whitelist (process_name) VALUES (?1)",
            [process_name],
        )
        .map_err(|e| format!("failed to add whitelist item: {e}"))?;
    } else {
        conn.execute(
            "DELETE FROM app_whitelist WHERE process_name = ?1",
            [process_name],
        )
        .map_err(|e| format!("failed to remove whitelist item: {e}"))?;
    }

    Ok(())
}

fn set_main_close_behavior(main_window: &tauri::WebviewWindow) {
    let main_clone = main_window.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = main_clone.hide();
        }
    });
}

fn reveal_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.unminimize();
        let _ = main_window.center();
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }
}

fn ensure_pet_window_position(app: &AppHandle, force_to_corner: bool) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;

    let monitor = app
        .primary_monitor()
        .map_err(|e| format!("failed to get primary monitor: {e}"))?
        .ok_or_else(|| "primary monitor unavailable".to_string())?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to get pet window size: {e}"))?;
    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to get pet window position: {e}"))?;

    let desired_x = monitor_pos.x + monitor_size.width as i32 - pet_size.width as i32 - 24;
    let desired_y = monitor_pos.y + monitor_size.height as i32 - pet_size.height as i32 - 48;

    let offscreen = pet_pos.x < monitor_pos.x - 40
        || pet_pos.y < monitor_pos.y - 40
        || pet_pos.x + pet_size.width as i32 > monitor_pos.x + monitor_size.width as i32 + 40
        || pet_pos.y + pet_size.height as i32 > monitor_pos.y + monitor_size.height as i32 + 40;

    if force_to_corner || offscreen {
        pet_window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: desired_x,
                y: desired_y,
            }))
            .map_err(|e| format!("failed to move pet window: {e}"))?;
    }

    pet_window
        .set_always_on_top(true)
        .map_err(|e| format!("failed to set pet always on top: {e}"))?;

    Ok(())
}

fn pet_monitor_rect(
    app: &AppHandle,
    pet_window: &tauri::WebviewWindow,
) -> Result<(i32, i32, i32, i32), String> {
    let monitor = pet_window
        .current_monitor()
        .map_err(|e| format!("failed to get current pet monitor: {e}"))?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or_else(|| "pet monitor unavailable".to_string())?;
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    Ok((
        monitor_pos.x,
        monitor_pos.y,
        monitor_size.width as i32,
        monitor_size.height as i32,
    ))
}

fn clamp_pet_window_position(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rect_x: i32,
    rect_y: i32,
    rect_width: i32,
    rect_height: i32,
) -> (i32, i32) {
    let max_x = rect_x + (rect_width - width as i32).max(0);
    let max_y = rect_y + (rect_height - height as i32).max(0);
    (x.clamp(rect_x, max_x), y.clamp(rect_y, max_y))
}

fn normalize_pet_settle_mode(value: &str) -> Result<&'static str, String> {
    match value.trim().to_lowercase().as_str() {
        "free" => Ok("free"),
        "dock_left" => Ok("dock_left"),
        "dock_right" => Ok("dock_right"),
        other => Err(format!("invalid pet settle mode: {other}")),
    }
}

fn settle_pet_window_internal(
    app: &AppHandle,
    mode: &str,
) -> Result<PetWindowSettleResult, String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    let normalized_mode = normalize_pet_settle_mode(mode)?;
    let (target_width, target_height) = (220_u32, 262_u32);

    pet_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: target_width,
            height: target_height,
        }))
        .map_err(|e| format!("failed to resize pet window during settle: {e}"))?;

    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to read pet window size during settle: {e}"))?;
    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to read pet window position during settle: {e}"))?;
    let (rect_x, rect_y, rect_width, rect_height) = pet_monitor_rect(app, &pet_window)?;

    let desired_x = match normalized_mode {
        "dock_left" => rect_x,
        "dock_right" => rect_x + rect_width - pet_size.width as i32,
        _ => pet_pos.x,
    };
    let desired_y = pet_pos.y;
    let (next_x, next_y) = clamp_pet_window_position(
        desired_x,
        desired_y,
        pet_size.width,
        pet_size.height,
        rect_x,
        rect_y,
        rect_width,
        rect_height,
    );

    pet_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: next_x,
            y: next_y,
        }))
        .map_err(|e| format!("failed to position pet window during settle: {e}"))?;

    let _ = sync_pet_panel_position(app.clone());
    Ok(PetWindowSettleResult {
        x: next_x,
        y: next_y,
        width: pet_size.width,
        height: pet_size.height,
        state: normalized_mode.to_string(),
    })
}

fn ensure_pet_panel_position(app: &AppHandle) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;

    let panel_window = app
        .get_webview_window("pet-panel")
        .ok_or_else(|| "pet panel window not found".to_string())?;

    let monitor = pet_window
        .current_monitor()
        .map_err(|e| format!("failed to get current monitor: {e}"))?
        .ok_or_else(|| "current monitor unavailable".to_string())?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to get pet window position: {e}"))?;
    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to get pet window size: {e}"))?;
    let panel_size = panel_window
        .outer_size()
        .map_err(|e| format!("failed to get pet panel window size: {e}"))?;

    let mut target_x = pet_pos.x + pet_size.width as i32 + 8;
    let mut target_y = pet_pos.y;

    let max_x = monitor_pos.x + monitor_size.width as i32 - panel_size.width as i32 - 8;
    let min_x = monitor_pos.x + 8;
    let max_y = monitor_pos.y + monitor_size.height as i32 - panel_size.height as i32 - 8;
    let min_y = monitor_pos.y + 8;

    if target_x > max_x {
        target_x = pet_pos.x - panel_size.width as i32 - 8;
    }

    target_x = target_x.clamp(min_x, max_x.max(min_x));
    target_y = target_y.clamp(min_y, max_y.max(min_y));

    panel_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: target_x,
            y: target_y,
        }))
        .map_err(|e| format!("failed to move pet panel window: {e}"))?;

    panel_window
        .set_always_on_top(true)
        .map_err(|e| format!("failed to set pet panel always on top: {e}"))?;

    Ok(())
}

#[tauri::command]
fn summon_pet_window(app: AppHandle) -> Result<(), String> {
    let pet_window = if let Some(existing) = app.get_webview_window("pet") {
        existing
    } else {
        tauri::WebviewWindowBuilder::new(&app, "pet", tauri::WebviewUrl::App("pet.html".into()))
            .title("TimePrism Pet")
            .inner_size(220.0, 262.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .build()
            .map_err(|e| format!("failed to recreate pet window: {e}"))?
    };

    pet_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: 220,
            height: 262,
        }))
        .map_err(|e| format!("failed to resize pet window: {e}"))?;

    ensure_pet_window_position(&app, true)?;

    pet_window
        .show()
        .map_err(|e| format!("failed to show pet window: {e}"))?;
    pet_window
        .set_focus()
        .map_err(|e| format!("failed to focus pet window: {e}"))?;

    let _ = pet_window.emit("pet-summoned", true);

    Ok(())
}

#[tauri::command]
fn sync_pet_window_layout(app: AppHandle) -> Result<(), String> {
    let _ = settle_pet_window_internal(&app, "free")?;
    Ok(())
}

#[tauri::command]
fn settle_pet_window(
    app: AppHandle,
    input: SettlePetWindowInput,
) -> Result<PetWindowSettleResult, String> {
    settle_pet_window_internal(&app, &input.mode)
}

#[tauri::command]
fn show_pet_panel(app: AppHandle, mode: String) -> Result<(), String> {
    let panel_window = if let Some(existing) = app.get_webview_window("pet-panel") {
        existing
    } else {
        tauri::WebviewWindowBuilder::new(
            &app,
            "pet-panel",
            tauri::WebviewUrl::App("pet-panel.html".into()),
        )
        .title("TimePrism Panel")
        .inner_size(186.0, 128.0)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .build()
        .map_err(|e| format!("failed to create pet panel window: {e}"))?
    };

    ensure_pet_panel_position(&app)?;
    panel_window
        .show()
        .map_err(|e| format!("failed to show pet panel window: {e}"))?;
    let _ = panel_window.emit("pet-panel-active", true);
    let _ = panel_window.emit("pet-panel-mode", mode);
    Ok(())
}

#[tauri::command]
fn hide_pet_panel(app: AppHandle) -> Result<(), String> {
    if let Some(panel_window) = app.get_webview_window("pet-panel") {
        let _ = panel_window.emit("pet-panel-active", false);
        panel_window
            .hide()
            .map_err(|e| format!("failed to hide pet panel window: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
fn sync_pet_panel_position(app: AppHandle) -> Result<(), String> {
    if app.get_webview_window("pet-panel").is_none() {
        return Ok(());
    }
    ensure_pet_panel_position(&app)
}

#[tauri::command]
fn resize_pet_panel(app: AppHandle, width: i32, height: i32) -> Result<(), String> {
    let panel_window = app
        .get_webview_window("pet-panel")
        .ok_or_else(|| "pet panel window not found".to_string())?;

    let safe_w = width.clamp(150, 300) as u32;
    let safe_h = height.clamp(96, 260) as u32;
    panel_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: safe_w,
            height: safe_h,
        }))
        .map_err(|e| format!("failed to resize pet panel window: {e}"))?;

    let _ = sync_pet_panel_position(app);
    Ok(())
}

#[tauri::command]
fn move_pet_window(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| format!("failed to move pet window from command: {e}"))?;

    let _ = sync_pet_panel_position(app);
    Ok(())
}

#[tauri::command]
fn close_pet_window(app: AppHandle) -> Result<(), String> {
    let _ = hide_pet_panel(app.clone());
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .close()
        .map_err(|e| format!("failed to close pet window: {e}"))?;
    Ok(())
}

#[tauri::command]
fn hide_pet_window(app: AppHandle) -> Result<(), String> {
    let _ = hide_pet_panel(app.clone());
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .hide()
        .map_err(|e| format!("failed to hide pet window: {e}"))?;
    Ok(())
}

#[tauri::command]
fn begin_pet_drag(app: AppHandle) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .start_dragging()
        .map_err(|e| format!("failed to start pet dragging: {e}"))?;
    Ok(())
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    let window = if let Some(existing) = app.get_webview_window("main") {
        existing
    } else {
        let created = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("TimePrism")
        .inner_size(888.0, 900.0)
        .build()
        .map_err(|e| format!("failed to recreate main window: {e}"))?;
        set_main_close_behavior(&created);
        created
    };

    let _ = window.unminimize();
    window
        .show()
        .map_err(|e| format!("failed to show main window: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("failed to focus main window: {e}"))?;

    Ok(())
}

#[tauri::command]
fn show_main_window_section(app: AppHandle, section: String) -> Result<(), String> {
    let window = if let Some(existing) = app.get_webview_window("main") {
        existing
    } else {
        let created = tauri::WebviewWindowBuilder::new(
            &app,
            "main",
            tauri::WebviewUrl::App("index.html".into()),
        )
        .title("TimePrism")
        .inner_size(888.0, 900.0)
        .build()
        .map_err(|e| format!("failed to recreate main window: {e}"))?;
        set_main_close_behavior(&created);
        created
    };

    let _ = window.unminimize();
    window
        .show()
        .map_err(|e| format!("failed to show main window: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("failed to focus main window: {e}"))?;
    window
        .emit("navigate-insights-section", section)
        .map_err(|e| format!("failed to emit navigate event: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = summon_pet_window(app.clone());
                    }
                })
                .build(),
        )
        .setup(|app| {
            if let Err(err) = init_database(app.handle()) {
                eprintln!("database init warning: {err}");
            }

            if let Some(main_window) = app.get_webview_window("main") {
                set_main_close_behavior(&main_window);
                reveal_main_window(app.handle());
            }

            let _ = ensure_pet_window_position(app.handle(), true);

            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT);
            if let Err(err) = app.global_shortcut().register(shortcut) {
                eprintln!("global shortcut warning: failed to register summon hotkey: {err}");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_categories,
            create_category,
            start_session,
            stop_active_session,
            append_app_usage_log,
            capture_foreground_once,
            list_foreground_capture_diagnostics,
            list_pending_idle_prompts,
            resolve_idle_prompt,
            get_idle_memory_state,
            clear_idle_memory_state,
            list_app_rules,
            list_pending_rule_processes,
            list_reminders,
            list_due_reminders,
            save_reminder,
            delete_reminder,
            set_reminder_done,
            set_reminder_order,
            snooze_reminder,
            get_today_summary,
            list_top_apps_today,
            list_top_apps_all_time,
            get_learn_heatmap,
            get_heatmap_goal_seconds_setting,
            set_heatmap_goal_seconds_setting,
            get_usage_stack,
            list_recent_logs,
            save_app_rule,
            check_focus_deviation,
            snooze_focus_guard,
            get_privacy_settings,
            update_privacy_settings,
            get_auto_start_enabled,
            set_auto_start_enabled,
            list_whitelist,
            set_whitelist_item,
            show_main_window,
            show_main_window_section,
            summon_pet_window,
            sync_pet_window_layout,
            show_pet_panel,
            hide_pet_panel,
            sync_pet_panel_position,
            resize_pet_panel,
            settle_pet_window,
            move_pet_window,
            hide_pet_window,
            close_pet_window,
            begin_pet_drag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
