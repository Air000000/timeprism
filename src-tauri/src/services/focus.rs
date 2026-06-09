use std::sync::Mutex;

use chrono::Local;
use once_cell::sync::Lazy;
use rusqlite::Connection;

use super::privacy::normalize_process_key;
use super::sessions::active_root_type;
use crate::domain::rules::DeviationCheck;

static DEVIATION_STATE: Lazy<Mutex<DeviationState>> =
    Lazy::new(|| Mutex::new(DeviationState::default()));

#[derive(Default)]
struct DeviationState {
    mismatch_since: Option<i64>,
    pending_alert: bool,
    cooldown_until: i64,
}

pub(crate) fn check_focus_deviation_state(
    conn: &Connection,
    process_name: String,
    debounce_seconds: Option<i64>,
) -> Result<DeviationCheck, String> {
    let now_ts = Local::now().timestamp();
    let deb = debounce_seconds.unwrap_or(60).clamp(5, 600);
    let process_name = normalize_process_key(&process_name);

    let current_active_root = active_root_type(conn)?;

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

pub(crate) fn snooze_focus_guard_state(cooldown_seconds: Option<i64>) -> Result<(), String> {
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
