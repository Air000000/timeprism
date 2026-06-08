use chrono::Local;
use rusqlite::{params, Connection};

use crate::domain::rules::{AppRuleEntry, PendingRuleProcess, SaveAppRuleInput};

use super::privacy::normalize_process_key;

pub(crate) fn resolve_rule_mapping(conn: &Connection, process_name: &str) -> (bool, String) {
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

pub(crate) fn upsert_app_rule_entry(
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

pub(crate) fn save_app_rule_entry(conn: &Connection, input: SaveAppRuleInput) -> Result<(), String> {
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

    upsert_app_rule_entry(conn, &process_name, &mapped_type, &privacy_level)
}

pub(crate) fn list_app_rule_entries(
    conn: &Connection,
    cap: i64,
) -> Result<Vec<AppRuleEntry>, String> {
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

pub(crate) fn list_pending_rule_process_entries(
    conn: &Connection,
    cap: i64,
) -> Result<Vec<PendingRuleProcess>, String> {
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

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use crate::domain::rules::SaveAppRuleInput;

    use super::{list_pending_rule_process_entries, resolve_rule_mapping, save_app_rule_entry};

    fn rules_test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory rules db");
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
        .expect("create rules test schema");
        conn
    }

    #[test]
    fn resolves_missing_and_invalid_rules_as_ignore() {
        let conn = rules_test_conn();
        assert_eq!(resolve_rule_mapping(&conn, "unknown.exe"), (false, "IGNORE".to_string()));

        conn.execute(
            "INSERT INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
             VALUES ('odd.exe', 'SOMETHING', 'NORMAL', 1, 1)",
            [],
        )
        .expect("insert invalid mapped type");

        assert_eq!(resolve_rule_mapping(&conn, "odd.exe"), (true, "IGNORE".to_string()));
    }

    #[test]
    fn save_app_rule_entry_normalizes_and_validates_input() {
        let conn = rules_test_conn();

        save_app_rule_entry(
            &conn,
            SaveAppRuleInput {
                process_name: r#""C:\Tools\Code.EXE""#.to_string(),
                mapped_type: " learn ".to_string(),
                privacy_level: None,
            },
        )
        .expect("save app rule");

        let row = conn
            .query_row(
                "SELECT process_name, mapped_type, privacy_level FROM app_rules WHERE process_name = 'code.exe'",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .expect("read saved app rule");

        assert_eq!(row, ("code.exe".to_string(), "LEARN".to_string(), "NORMAL".to_string()));

        let err = save_app_rule_entry(
            &conn,
            SaveAppRuleInput {
                process_name: "bad.exe".to_string(),
                mapped_type: "BAD".to_string(),
                privacy_level: None,
            },
        )
        .expect_err("invalid mapped type");
        assert_eq!(err, "mapped_type must be LEARN, REST, or IGNORE");
    }

    #[test]
    fn pending_rule_processes_include_only_unruled_long_running_processes() {
        let conn = rules_test_conn();
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES ('unknown.exe', 'Unknown Window', 10, 180000)",
            [],
        )
        .expect("insert pending usage");
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES ('short.exe', 'Short Window', 20, 179999)",
            [],
        )
        .expect("insert short usage");
        conn.execute(
            "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
             VALUES ('known.exe', 'Known Window', 30, 180000)",
            [],
        )
        .expect("insert known usage");
        conn.execute(
            "INSERT INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
             VALUES ('known.exe', 'LEARN', 'NORMAL', 1, 1)",
            [],
        )
        .expect("insert known rule");

        let pending = list_pending_rule_process_entries(&conn, 10).expect("list pending rules");

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].process_name, "unknown.exe");
        assert_eq!(pending[0].last_seen_timestamp, 10);
        assert_eq!(pending[0].total_seconds, 180);
        assert_eq!(pending[0].last_window_title, "Unknown Window");
    }
}
