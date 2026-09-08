use chrono::Local;
use rusqlite::{params, Connection};

pub(crate) fn active_root_type(conn: &Connection) -> Result<Option<String>, String> {
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

pub(crate) fn start_session_entry(conn: &Connection, category_id: i64) -> Result<i64, String> {
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

pub(crate) fn stop_active_session_entry(conn: &Connection) -> Result<bool, String> {
    let now_ts = Local::now().timestamp();
    let changed = conn
        .execute(
            "UPDATE task_sessions SET end_time = ?1 WHERE end_time IS NULL",
            [now_ts],
        )
        .map_err(|e| format!("failed to stop active session: {e}"))?;

    Ok(changed > 0)
}
