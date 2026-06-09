use rusqlite::{params, Connection};

use super::privacy::{normalize_process_key, process_log_with_privacy};
use super::rules::upsert_app_rule_entry;
use crate::domain::idle::IdlePromptEntry;

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

pub(crate) fn persist_idle_prompt_decision(
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

pub(crate) fn append_usage_log_record(
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
