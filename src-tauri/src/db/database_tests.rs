use rusqlite::Connection;

use super::migrations::init_connection;

fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .expect("query table existence")
        == 1
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("prepare table info");
    stmt.query_map([], |row| row.get::<_, String>(1))
        .expect("query table info")
        .any(|name| name.expect("parse column name") == column)
}

#[test]
fn fresh_database_initialization_creates_schema_and_defaults() {
    let conn = Connection::open_in_memory().expect("open in-memory db");

    init_connection(&conn).expect("initialize fresh database");

    for table in [
        "categories",
        "task_sessions",
        "app_usage_logs",
        "app_rules",
        "app_config",
        "app_whitelist",
        "reminders",
        "daily_heatmap_snapshot",
    ] {
        assert!(table_exists(&conn, table), "missing table: {table}");
    }

    let learn_name: String = conn
        .query_row("SELECT name FROM categories WHERE id = 1", [], |row| row.get(0))
        .expect("load LEARN root name");
    let rest_name: String = conn
        .query_row("SELECT name FROM categories WHERE id = 2", [], |row| row.get(0))
        .expect("load REST root name");
    assert_eq!(learn_name, "学习");
    assert_eq!(rest_name, "休息");

    let browser_mode: String = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = 'browser_title_mode'",
            [],
            |row| row.get(0),
        )
        .expect("load browser_title_mode");
    assert_eq!(browser_mode, "BLUR");

    let code_rule: (String, String) = conn
        .query_row(
            "SELECT mapped_type, privacy_level FROM app_rules WHERE process_name = 'code.exe'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("load code.exe rule");
    assert_eq!(code_rule, ("LEARN".into(), "NORMAL".into()));

    let foreign_keys: i64 = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .expect("read foreign_keys pragma");
    assert_eq!(foreign_keys, 1);
}

#[test]
fn database_initialization_is_idempotent_and_preserves_user_values() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init_connection(&conn).expect("initialize database first time");

    conn.execute(
        "UPDATE app_config SET value = '3600' WHERE key = 'heatmap_goal_seconds'",
        [],
    )
    .expect("customize heatmap goal");
    conn.execute(
        "UPDATE app_rules SET mapped_type = 'REST' WHERE process_name = 'code.exe'",
        [],
    )
    .expect("customize code.exe rule");

    init_connection(&conn).expect("initialize database second time");

    let goal: String = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = 'heatmap_goal_seconds'",
            [],
            |row| row.get(0),
        )
        .expect("load customized goal");
    let code_mapping: String = conn
        .query_row(
            "SELECT mapped_type FROM app_rules WHERE process_name = 'code.exe'",
            [],
            |row| row.get(0),
        )
        .expect("load customized code mapping");

    assert_eq!(goal, "3600");
    assert_eq!(code_mapping, "REST");
}

#[test]
fn legacy_reminders_schema_is_upgraded_without_losing_rows() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        r#"
        CREATE TABLE reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE', 'DAILY')),
            remind_at INTEGER NULL,
            daily_time_minutes INTEGER NULL,
            is_completed INTEGER NOT NULL DEFAULT 0,
            completed_day_key TEXT NULL,
            completed_at INTEGER NULL,
            snooze_until INTEGER NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        INSERT INTO reminders (
            content, repeat_rule, remind_at, daily_time_minutes,
            is_completed, created_at, updated_at
        ) VALUES ('legacy reminder', 'DAILY', NULL, 540, 0, 1, 1);
        "#,
    )
    .expect("create legacy reminders schema");

    init_connection(&conn).expect("upgrade legacy database");

    assert!(column_exists(&conn, "reminders", "weekly_days"));
    assert!(column_exists(&conn, "reminders", "sort_order"));

    let reminder: (String, String, Option<String>, i64) = conn
        .query_row(
            "SELECT content, repeat_rule, weekly_days, sort_order FROM reminders WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("load migrated reminder");
    assert_eq!(reminder, ("legacy reminder".into(), "DAILY".into(), None, 0));
}
