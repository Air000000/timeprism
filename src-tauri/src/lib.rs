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
use services::foreground as foreground_service;
use services::privacy::{
    get_privacy_settings_entry, list_whitelist_entries, set_whitelist_item_entry,
    update_privacy_settings_entry,
};
use services::reminders::{
    delete_reminder_entry, list_due_reminder_entries, list_reminder_entries, save_reminder_entry,
    set_reminder_done_entry, set_reminder_order_entries, snooze_reminder_entry,
};
use services::rules::{list_app_rule_entries, list_pending_rule_process_entries, save_app_rule_entry};
use services::focus::{
    check_focus_deviation_state, snooze_focus_guard_state,
};
use services::sessions::{start_session_entry, stop_active_session_entry};
use services::startup::{get_auto_start_enabled_state, set_auto_start_enabled_state};
use services::usage::append_usage_log_record;
use services::window as window_service;

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
    foreground_service::capture_foreground_once(&app, duration_ms)
}

#[tauri::command]
fn list_pending_idle_prompts(limit: Option<i64>) -> Result<Vec<IdlePromptEntry>, String> {
    foreground_service::list_pending_idle_prompts(limit)
}

#[tauri::command]
fn resolve_idle_prompt(app: AppHandle, input: ResolveIdlePromptInput) -> Result<bool, String> {
    foreground_service::resolve_idle_prompt(&app, input)
}

#[tauri::command]
fn get_idle_memory_state() -> Result<IdleMemoryState, String> {
    foreground_service::get_idle_memory_state()
}

#[tauri::command]
fn clear_idle_memory_state() -> Result<(), String> {
    foreground_service::clear_idle_memory_state()
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
    foreground_service::list_foreground_capture_diagnostics(limit, unique_by_process)
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
