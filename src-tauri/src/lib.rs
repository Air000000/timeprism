use std::collections::HashMap;
use std::sync::Mutex;
use chrono::Local;
use once_cell::sync::Lazy;
use tauri::{AppHandle, Manager};
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
use services::categories::{create_category_entry, list_category_entries};
use services::foreground::{capture_foreground_window, current_idle_millis};
use services::privacy::{
    get_privacy_settings_entry, list_whitelist_entries, normalize_process_key,
    set_whitelist_item_entry, update_privacy_settings_entry,
};
use services::reminders::{
    delete_reminder_entry, list_due_reminder_entries, list_reminder_entries, save_reminder_entry,
    set_reminder_done_entry, set_reminder_order_entries, snooze_reminder_entry,
};
use services::rules::{
    list_app_rule_entries, list_pending_rule_process_entries, resolve_rule_mapping,
    save_app_rule_entry,
};
use services::focus::{
    check_focus_deviation_state, snooze_focus_guard_state,
};
use services::sessions::{start_session_entry, stop_active_session_entry};
use services::startup::{get_auto_start_enabled_state, set_auto_start_enabled_state};
use services::usage::{append_usage_log_record, persist_idle_prompt_decision};
use services::window as window_service;

static FOREGROUND_SAMPLE_STATE: Lazy<Mutex<ForegroundSampleState>> =
    Lazy::new(|| Mutex::new(ForegroundSampleState::default()));
const IDLE_PROMPT_THRESHOLD_MS: i64 = 300_000;

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
    list_category_entries(&conn)
}

#[tauri::command]
fn create_category(app: AppHandle, input: CreateCategoryInput) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    create_category_entry(&conn, input)
}

#[tauri::command]
fn start_session(app: AppHandle, category_id: i64) -> Result<i64, String> {
    let conn = open_connection(&app)?;
    start_session_entry(&conn, category_id)
}

#[tauri::command]
fn stop_active_session(app: AppHandle) -> Result<bool, String> {
    let conn = open_connection(&app)?;
    stop_active_session_entry(&conn)
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
    let conn = open_connection(&app)?;
    check_focus_deviation_state(&conn, process_name, debounce_seconds)
}

#[tauri::command]
fn snooze_focus_guard(cooldown_seconds: Option<i64>) -> Result<(), String> {
    snooze_focus_guard_state(cooldown_seconds)
}

#[tauri::command]
fn get_privacy_settings(app: AppHandle) -> Result<PrivacySettings, String> {
    let conn = open_connection(&app)?;
    get_privacy_settings_entry(&conn)
}

#[tauri::command]
fn update_privacy_settings(app: AppHandle, input: UpdatePrivacySettingsInput) -> Result<(), String> {
    let conn = open_connection(&app)?;
    update_privacy_settings_entry(&conn, input)
}

#[tauri::command]
fn get_auto_start_enabled() -> Result<bool, String> {
    get_auto_start_enabled_state()
}

#[tauri::command]
fn set_auto_start_enabled(enabled: bool) -> Result<bool, String> {
    set_auto_start_enabled_state(enabled)
}

#[tauri::command]
fn list_whitelist(app: AppHandle) -> Result<Vec<String>, String> {
    let conn = open_connection(&app)?;
    list_whitelist_entries(&conn)
}

#[tauri::command]
fn set_whitelist_item(app: AppHandle, input: SetWhitelistItemInput) -> Result<(), String> {
    let conn = open_connection(&app)?;
    set_whitelist_item_entry(&conn, input)
}

#[tauri::command]
fn summon_pet_window(app: AppHandle) -> Result<(), String> {
    window_service::summon_pet_window(&app)
}

#[tauri::command]
fn sync_pet_window_layout(app: AppHandle) -> Result<(), String> {
    window_service::sync_pet_window_layout(&app)
}

#[tauri::command]
fn settle_pet_window(
    app: AppHandle,
    input: SettlePetWindowInput,
) -> Result<PetWindowSettleResult, String> {
    window_service::settle_pet_window(&app, input)
}

#[tauri::command]
fn show_pet_panel(app: AppHandle, mode: String) -> Result<(), String> {
    window_service::show_pet_panel(&app, mode)
}

#[tauri::command]
fn hide_pet_panel(app: AppHandle) -> Result<(), String> {
    window_service::hide_pet_panel(&app)
}

#[tauri::command]
fn sync_pet_panel_position(app: AppHandle) -> Result<(), String> {
    window_service::sync_pet_panel_position(&app)
}

#[tauri::command]
fn resize_pet_panel(app: AppHandle, width: i32, height: i32) -> Result<(), String> {
    window_service::resize_pet_panel(&app, width, height)
}

#[tauri::command]
fn move_pet_window(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    window_service::move_pet_window(&app, x, y)
}

#[tauri::command]
fn close_pet_window(app: AppHandle) -> Result<(), String> {
    window_service::close_pet_window(&app)
}

#[tauri::command]
fn hide_pet_window(app: AppHandle) -> Result<(), String> {
    window_service::hide_pet_window(&app)
}

#[tauri::command]
fn begin_pet_drag(app: AppHandle) -> Result<(), String> {
    window_service::begin_pet_drag(&app)
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    window_service::show_main_window(&app)
}

#[tauri::command]
fn show_main_window_section(app: AppHandle, section: String) -> Result<(), String> {
    window_service::show_main_window_section(&app, section)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = window_service::summon_pet_window(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            if let Err(err) = init_database(app.handle()) {
                eprintln!("database init warning: {err}");
            }

            if let Some(main_window) = app.get_webview_window("main") {
                window_service::set_main_close_behavior(&main_window);
                window_service::reveal_main_window(app.handle());
            }

            let _ = window_service::ensure_pet_window_position(app.handle(), true);

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
