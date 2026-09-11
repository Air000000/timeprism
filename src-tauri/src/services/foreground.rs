use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Local;
use once_cell::sync::Lazy;
use tauri::AppHandle;
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
    GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
};

use super::privacy::normalize_process_key;
use super::rules::resolve_rule_mapping;
use super::usage::{
    append_usage_log_record, persist_idle_prompt_app_decision, persist_idle_prompt_decision,
};
use crate::db::connection::open_connection;
use crate::domain::idle::{IdleMemoryState, IdlePromptEntry, ResolveIdlePromptInput};
use crate::domain::window::ForegroundCaptureDiagnostic;

static FOREGROUND_SAMPLE_STATE: Lazy<Mutex<ForegroundSampleState>> =
    Lazy::new(|| Mutex::new(ForegroundSampleState::default()));
const IDLE_PROMPT_THRESHOLD_MS: i64 = 300_000;

#[derive(Default)]
struct ForegroundSampleState {
    last: Option<ForegroundSnapshot>,
    diagnostics: Vec<ForegroundCaptureDiagnostic>,
    idle_segment_start_ms: Option<i64>,
    idle_attribution_process_name: Option<String>,
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

fn idle_attribution_candidate(snapshot: Option<&ForegroundSnapshot>) -> Option<String> {
    let process_name = snapshot.map(|item| normalize_process_key(&item.process_name))?;
    if process_name == "unknown.exe"
        || process_name == "lockapp.exe"
        || process_name == "logonui.exe"
        || process_name.starts_with("pid-")
    {
        return None;
    }
    Some(process_name)
}

fn push_foreground_diagnostic(mut entry: ForegroundCaptureDiagnostic) -> Result<(), String> {
    entry.observed_window_title = "Hidden for Privacy".to_string();

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
pub(crate) fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
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
    let query_ok =
        unsafe { QueryFullProcessImageNameW(process, 0, path_buf.as_mut_ptr(), &mut size) };
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
pub(crate) fn capture_foreground_window() -> Result<Option<(String, String)>, String> {
    Err("foreground capture is currently supported only on Windows".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn current_idle_millis() -> Result<i64, String> {
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
pub(crate) fn current_idle_millis() -> Result<i64, String> {
    Ok(0)
}

pub(crate) fn capture_foreground_once(
    app: &AppHandle,
    duration_ms: Option<i64>,
) -> Result<bool, String> {
    let now_ms = Local::now().timestamp_millis();
    let idle_ms = current_idle_millis().unwrap_or(0).max(0);

    if idle_ms >= IDLE_PROMPT_THRESHOLD_MS {
        let idle_start_ms = now_ms - idle_ms;
        {
            let mut state = FOREGROUND_SAMPLE_STATE
                .lock()
                .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
            if state.idle_segment_start_ms.is_none() {
                state.idle_attribution_process_name = idle_attribution_candidate(state.last.as_ref());
            }
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
            let attribution_process_name = state.idle_attribution_process_name.take();
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
                    attribution_process_name,
                };

                if let Some(remembered) = state.remembered_idle_decision.clone() {
                    drop(state);
                    let conn = open_connection(app)?;
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
        } else {
            state.idle_attribution_process_name = None;
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
        let conn = open_connection(app)?;
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
        let conn = open_connection(app)?;
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

    let conn = open_connection(app)?;
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

pub(crate) fn list_pending_idle_prompts(
    limit: Option<i64>,
) -> Result<Vec<IdlePromptEntry>, String> {
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

pub(crate) fn resolve_idle_prompt(
    app: &AppHandle,
    input: ResolveIdlePromptInput,
) -> Result<bool, String> {
    let decision = input.decision.trim().to_uppercase();
    let remember_this_session = input.remember_this_session.unwrap_or(false);
    if !matches!(decision.as_str(), "APP" | "LEARN" | "REST" | "IDLE" | "SKIP") {
        return Err("decision must be APP, LEARN, REST, IDLE, or SKIP".to_string());
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

    if decision == "APP" {
        let state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        let has_candidate = state
            .pending_idle_prompts
            .iter()
            .find(|item| item.id == input.prompt_id)
            .and_then(|item| item.attribution_process_name.as_ref())
            .is_some();
        if !has_candidate {
            return Err("idle prompt has no previous-app attribution candidate".to_string());
        }
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
    let conn = open_connection(app)?;
    let stored = if decision == "APP" {
        let process_name = prompt
            .attribution_process_name
            .as_deref()
            .ok_or_else(|| "idle prompt has no previous-app attribution candidate".to_string())?;
        persist_idle_prompt_app_decision(&conn, &prompt, process_name)?
    } else {
        persist_idle_prompt_decision(&conn, &prompt, &decision)?
    };

    if remember_this_session && matches!(decision.as_str(), "LEARN" | "REST" | "IDLE") {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
        state.remembered_idle_decision = Some(decision);
    }

    Ok(stored)
}

pub(crate) fn get_idle_memory_state() -> Result<IdleMemoryState, String> {
    let state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
    Ok(IdleMemoryState {
        remembered_decision: state.remembered_idle_decision.clone(),
    })
}

pub(crate) fn clear_idle_memory_state() -> Result<(), String> {
    let mut state = FOREGROUND_SAMPLE_STATE
        .lock()
        .map_err(|e| format!("failed to lock foreground sample state: {e}"))?;
    state.remembered_idle_decision = None;
    Ok(())
}

pub(crate) fn list_foreground_capture_diagnostics(
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

#[cfg(test)]
mod tests {
    use super::{
        idle_attribution_candidate, list_foreground_capture_diagnostics, push_foreground_diagnostic,
        ForegroundSnapshot, FOREGROUND_SAMPLE_STATE,
    };
    use crate::domain::window::ForegroundCaptureDiagnostic;

    fn clear_diagnostics() {
        let mut state = FOREGROUND_SAMPLE_STATE
            .lock()
            .expect("lock foreground sample state");
        state.diagnostics.clear();
    }

    #[test]
    fn idle_attribution_uses_normalized_previous_app_without_title() {
        let snapshot = ForegroundSnapshot {
            process_name: r#"C:\Program Files\Microsoft VS Code\Code.exe"#.to_string(),
            window_title: "Secret project title".to_string(),
            captured_at_ms: 1,
        };

        assert_eq!(
            idle_attribution_candidate(Some(&snapshot)),
            Some("code.exe".to_string())
        );
    }

    #[test]
    fn idle_attribution_rejects_lock_and_unknown_processes() {
        for process_name in ["LockApp.exe", "LogonUI.exe", "unknown.exe", "pid-42.exe"] {
            let snapshot = ForegroundSnapshot {
                process_name: process_name.to_string(),
                window_title: "Hidden".to_string(),
                captured_at_ms: 1,
            };
            assert_eq!(idle_attribution_candidate(Some(&snapshot)), None);
        }
    }

    #[test]
    fn diagnostics_do_not_expose_raw_window_titles() {
        clear_diagnostics();
        push_foreground_diagnostic(ForegroundCaptureDiagnostic {
            id: 1,
            captured_at_ms: 1,
            observed_process_name: "code.exe".to_string(),
            observed_window_title: "secret-project-name".to_string(),
            stored: false,
            block_reason: Some("baseline_only".to_string()),
            rule_saved: false,
            rule_mapped_type: "IGNORE".to_string(),
        })
        .expect("push diagnostic");

        let items = list_foreground_capture_diagnostics(Some(10), Some(false))
            .expect("list diagnostics");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].observed_window_title, "Hidden for Privacy");

        clear_diagnostics();
    }
}
