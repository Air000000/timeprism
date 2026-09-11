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
    let idle_start = prompt.start_timestamp.max(0);
    let idle_end = prompt.end_timestamp.max(idle_start);
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("failed to begin idle decision transaction: {e}"))?;

    // Foreground samples can be persisted during the idle threshold window before the sampler
    // knows the interval is idle. Resolving the prompt commits the retroactive classification,
    // so replace those provisional rows instead of adding an overlapping idle row on top.
    tx.execute(
        "UPDATE app_usage_logs
         SET duration_ms = MAX(0, (?1 - start_timestamp) * 1000)
         WHERE start_timestamp < ?1
           AND (start_timestamp + (duration_ms / 1000)) > ?1",
        [idle_start],
    )
    .map_err(|e| format!("failed to trim usage at idle start: {e}"))?;
    tx.execute(
        "DELETE FROM app_usage_logs
         WHERE start_timestamp >= ?1
           AND start_timestamp < ?2",
        params![idle_start, idle_end],
    )
    .map_err(|e| format!("failed to remove usage inside idle interval: {e}"))?;

    upsert_app_rule_entry(&tx, &process_key, mapped_type, "NORMAL")?;
    let stored = append_usage_log_direct(
        &tx,
        &process_key,
        title,
        prompt.start_timestamp,
        prompt.duration_ms,
    )?;

    tx.commit()
        .map_err(|e| format!("failed to commit idle decision transaction: {e}"))?;
    Ok(stored)
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

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{persist_idle_prompt_app_decision, persist_idle_prompt_decision};
    use crate::domain::idle::IdlePromptEntry;

    fn usage_test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory usage db");
        conn.execute_batch(
            r#"
            CREATE TABLE app_rules (
                process_name TEXT PRIMARY KEY,
                mapped_type TEXT NOT NULL,
                privacy_level TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE app_usage_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                process_name TEXT NOT NULL,
                window_title TEXT NOT NULL,
                start_timestamp INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL
            );
            "#,
        )
        .expect("create usage test schema");
        conn
    }

    #[test]
    fn idle_decision_replaces_overlapping_foreground_usage() {
        let conn = usage_test_conn();
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params!["code.exe", "Project", 100_i64, 400_000_i64],
        )
        .expect("insert foreground segment crossing idle start");
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params!["lockapp.exe", "Lock Screen", 350_i64, 100_000_i64],
        )
        .expect("insert foreground segment inside idle interval");

        let prompt = IdlePromptEntry {
            id: 1,
            start_timestamp: 300,
            end_timestamp: 600,
            duration_ms: 300_000,
            deferred_until_timestamp: None,
        };

        persist_idle_prompt_decision(&conn, &prompt, "REST").expect("persist idle decision");

        let mut stmt = conn
            .prepare(
                "SELECT process_name, start_timestamp, duration_ms
                 FROM app_usage_logs
                 ORDER BY start_timestamp, id",
            )
            .expect("prepare usage rows");
        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .expect("query usage rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect usage rows");

        assert_eq!(
            rows,
            vec![
                ("code.exe".to_string(), 100, 200_000),
                ("__idle_rest__.exe".to_string(), 300, 300_000),
            ]
        );
    }

    #[test]
    fn idle_app_decision_attributes_interval_without_overwriting_app_rule() {
        let conn = usage_test_conn();
        conn.execute(
            "INSERT INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
             VALUES ('code.exe', 'REST', 'NORMAL', 1, 1)",
            [],
        )
        .expect("seed app rule");
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params!["code.exe", "Project", 100_i64, 400_000_i64],
        )
        .expect("insert foreground segment crossing idle start");
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params!["lockapp.exe", "Lock Screen", 350_i64, 100_000_i64],
        )
        .expect("insert foreground segment inside idle interval");

        let prompt = IdlePromptEntry {
            id: 2,
            start_timestamp: 300,
            end_timestamp: 600,
            duration_ms: 300_000,
            deferred_until_timestamp: None,
        };

        persist_idle_prompt_app_decision(&conn, &prompt, "code.exe")
            .expect("persist app-attributed idle decision");

        let rows = conn
            .prepare(
                "SELECT process_name, window_title, start_timestamp, duration_ms
                 FROM app_usage_logs
                 ORDER BY start_timestamp, id",
            )
            .expect("prepare usage rows")
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                ))
            })
            .expect("query usage rows")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect usage rows");

        assert_eq!(
            rows,
            vec![
                ("code.exe".to_string(), "Project".to_string(), 100, 200_000),
                (
                    "code.exe".to_string(),
                    "Idle Confirmed · Previous App".to_string(),
                    300,
                    300_000,
                ),
            ]
        );

        let mapped_type: String = conn
            .query_row(
                "SELECT mapped_type FROM app_rules WHERE process_name = 'code.exe'",
                [],
                |row| row.get(0),
            )
            .expect("query app rule");
        assert_eq!(mapped_type, "REST");
    }
}
