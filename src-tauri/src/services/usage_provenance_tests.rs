use rusqlite::{params, Connection};

use super::usage::{
    append_usage_log_record, persist_idle_prompt_app_decision, persist_idle_prompt_decision,
};
use crate::domain::idle::IdlePromptEntry;

fn provenance_test_conn() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory provenance db");
    conn.execute_batch(
        r#"
        CREATE TABLE app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE app_rules (
            process_name TEXT PRIMARY KEY,
            mapped_type TEXT NOT NULL,
            privacy_level TEXT NOT NULL,
            created_at INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE app_whitelist (
            process_name TEXT PRIMARY KEY
        );
        CREATE TABLE app_usage_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            process_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            start_timestamp INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            source TEXT NOT NULL DEFAULT 'FOREGROUND'
                CHECK(source IN ('FOREGROUND', 'IDLE_CONFIRMED'))
        );
        "#,
    )
    .expect("create provenance test schema");
    conn
}

#[test]
fn generic_idle_decision_is_marked_idle_confirmed() {
    let conn = provenance_test_conn();
    let prompt = IdlePromptEntry {
        id: 1,
        start_timestamp: 300,
        end_timestamp: 600,
        duration_ms: 300_000,
        deferred_until_timestamp: None,
        attribution_process_name: None,
    };

    persist_idle_prompt_decision(&conn, &prompt, "LEARN").expect("persist idle decision");

    let source: String = conn
        .query_row(
            "SELECT source FROM app_usage_logs WHERE start_timestamp = 300",
            [],
            |row| row.get(0),
        )
        .expect("read idle source");
    assert_eq!(source, "IDLE_CONFIRMED");
}

#[test]
fn app_attributed_idle_decision_is_marked_idle_confirmed() {
    let conn = provenance_test_conn();
    conn.execute(
        "INSERT INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
         VALUES ('code.exe', 'LEARN', 'NORMAL', 1, 1)",
        [],
    )
    .expect("seed app rule");
    conn.execute(
        "INSERT INTO app_usage_logs (
            process_name, window_title, start_timestamp, duration_ms, source
         ) VALUES (?1, ?2, ?3, ?4, 'FOREGROUND')",
        params!["code.exe", "Project", 100_i64, 400_000_i64],
    )
    .expect("seed foreground segment");

    let prompt = IdlePromptEntry {
        id: 2,
        start_timestamp: 300,
        end_timestamp: 600,
        duration_ms: 300_000,
        deferred_until_timestamp: None,
        attribution_process_name: Some("code.exe".to_string()),
    };

    persist_idle_prompt_app_decision(&conn, &prompt, "code.exe")
        .expect("persist app-attributed idle decision");

    let source: String = conn
        .query_row(
            "SELECT source FROM app_usage_logs WHERE start_timestamp = 300",
            [],
            |row| row.get(0),
        )
        .expect("read attributed idle source");
    assert_eq!(source, "IDLE_CONFIRMED");
}

#[test]
fn foreground_append_does_not_merge_into_idle_confirmed_segment() {
    let conn = provenance_test_conn();
    conn.execute(
        "INSERT INTO app_usage_logs (
            process_name, window_title, start_timestamp, duration_ms, source
         ) VALUES ('code.exe', 'Project', 100, 5000, 'IDLE_CONFIRMED')",
        [],
    )
    .expect("seed user-confirmed segment");

    append_usage_log_record(&conn, "code.exe", "Project", 105, 5000)
        .expect("append foreground segment");

    let rows = conn
        .prepare("SELECT source, duration_ms FROM app_usage_logs ORDER BY id")
        .expect("prepare provenance rows")
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .expect("query provenance rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect provenance rows");

    assert_eq!(
        rows,
        vec![
            ("IDLE_CONFIRMED".to_string(), 5000),
            ("FOREGROUND".to_string(), 5000),
        ]
    );
}
