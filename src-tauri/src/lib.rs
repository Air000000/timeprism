use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;
use chrono::Local;
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod db;
mod domain;
mod services;

use db::connection::open_connection;
use db::migrations::init_database;
use domain::analytics::{
    Category, CreateCategoryInput, LearnHeatmapCell, RecentLogEntry, TodaySummary, TopApp,
    UsageStackDay,
};
use domain::idle::{IdleMemoryState, IdlePromptEntry, ResolveIdlePromptInput};
use domain::privacy::{PrivacySettings, SetWhitelistItemInput, UpdatePrivacySettingsInput};
use domain::reminders::{
    ReminderEntry, SaveReminderInput, SetReminderDoneInput, SetReminderOrderInput,
};
use domain::rules::{AppRuleEntry, DeviationCheck, PendingRuleProcess, SaveAppRuleInput};
use domain::window::{ForegroundCaptureDiagnostic, PetWindowSettleResult, SettlePetWindowInput};
use services::analytics::{
    heatmap_goal_seconds_setting, learn_heatmap, list_recent_log_entries,
    list_top_apps_all_time_entries, list_top_apps_today_entries,
    set_heatmap_goal_seconds_setting as save_heatmap_goal_seconds_setting, today_summary,
    usage_stack,
};
use services::foreground::{capture_foreground_window, current_idle_millis};
use services::privacy::{
    normalize_process_key, parse_bool_config, parse_browser_title_mode, process_log_with_privacy,
};
use services::reminders::{
    delete_reminder_entry, list_due_reminder_entries, list_reminder_entries, save_reminder_entry,
    set_reminder_done_entry, set_reminder_order_entries, snooze_reminder_entry,
};
use services::rules::{
    list_app_rule_entries, list_pending_rule_process_entries, resolve_rule_mapping,
    save_app_rule_entry, upsert_app_rule_entry,
};

static DEVIATION_STATE: Lazy<Mutex<DeviationState>> = Lazy::new(|| Mutex::new(DeviationState::default()));
static FOREGROUND_SAMPLE_STATE: Lazy<Mutex<ForegroundSampleState>> =
    Lazy::new(|| Mutex::new(ForegroundSampleState::default()));
const IDLE_PROMPT_THRESHOLD_MS: i64 = 300_000;
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

#[tauri::command]
fn list_reminders(
    app: AppHandle,
    limit: Option<i64>,
    include_completed: Option<bool>,
) -> Result<Vec<ReminderEntry>, String> {
    let conn = open_connection(&app)?;
    list_reminder_entries(&conn, limit, include_completed)
}

#[tauri::command]
fn list_due_reminders(app: AppHandle, limit: Option<i64>) -> Result<Vec<ReminderEntry>, String> {
    let conn = open_connection(&app)?;
    list_due_reminder_entries(&conn, limit)
}

#[tauri::command]
fn save_reminder(app: AppHandle, input: SaveReminderInput) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    save_reminder_entry(&conn, input)
}

#[tauri::command]
fn delete_reminder(app: AppHandle, id: i64) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    delete_reminder_entry(&conn, id)
}

#[tauri::command]
fn set_reminder_done(app: AppHandle, input: SetReminderDoneInput) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    set_reminder_done_entry(&conn, input)
}

#[tauri::command]
fn set_reminder_order(app: AppHandle, input: SetReminderOrderInput) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    set_reminder_order_entries(&conn, input)
}

#[tauri::command]
fn snooze_reminder(app: AppHandle, id: i64, snooze_seconds: Option<i64>) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    snooze_reminder_entry(&conn, id, snooze_seconds)
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
    list_recent_log_entries(&conn, limit)
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
    list_app_rule_entries(&conn, cap)
}

#[tauri::command]
fn list_pending_rule_processes(app: AppHandle, limit: Option<i64>) -> Result<Vec<PendingRuleProcess>, String> {
    let conn = open_connection(&app)?;
    let cap = limit.unwrap_or(10).clamp(1, 200);
    list_pending_rule_process_entries(&conn, cap)
}

#[tauri::command]
fn get_today_summary(app: AppHandle) -> Result<TodaySummary, String> {
    let conn = open_connection(&app)?;
    today_summary(&conn)
}

#[tauri::command]
fn list_top_apps_today(app: AppHandle, limit: Option<i64>) -> Result<Vec<TopApp>, String> {
    let conn = open_connection(&app)?;
    list_top_apps_today_entries(&conn, limit)
}

#[tauri::command]
fn list_top_apps_all_time(
    app: AppHandle,
    limit: Option<i64>,
    root_filter: Option<String>,
    include_ignore: Option<bool>,
) -> Result<Vec<TopApp>, String> {
    let conn = open_connection(&app)?;
    list_top_apps_all_time_entries(&conn, limit, root_filter, include_ignore)
}

#[tauri::command]
fn get_learn_heatmap(
    app: AppHandle,
    days: Option<i64>,
    goal_seconds: Option<i64>,
) -> Result<Vec<LearnHeatmapCell>, String> {
    let conn = open_connection(&app)?;
    learn_heatmap(&conn, days, goal_seconds)
}

#[tauri::command]
fn get_heatmap_goal_seconds_setting(app: AppHandle) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    Ok(heatmap_goal_seconds_setting(&conn))
}

#[tauri::command]
fn set_heatmap_goal_seconds_setting(app: AppHandle, goal_seconds: i64) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    save_heatmap_goal_seconds_setting(&conn, goal_seconds)
}

#[tauri::command]
fn get_usage_stack(
    app: AppHandle,
    days: Option<i64>,
    root_filter: Option<String>,
) -> Result<Vec<UsageStackDay>, String> {
    let conn = open_connection(&app)?;
    usage_stack(&conn, days, root_filter)
}

#[tauri::command]
fn save_app_rule(app: AppHandle, input: SaveAppRuleInput) -> Result<(), String> {
    let conn = open_connection(&app)?;
    save_app_rule_entry(&conn, input)
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
