use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

fn temp_db_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "timeprism-{label}-{}-{unique}.db",
        std::process::id()
    ))
}

fn remove_sqlite_files(path: &Path) {
    let _ = fs::remove_file(path);
    for suffix in ["-wal", "-shm"] {
        let _ = fs::remove_file(format!("{}{}", path.display(), suffix));
    }
}

fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .expect("query sqlite_master")
        == 1
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("prepare table info");
    stmt.query_map([], |row| row.get::<_, String>(1))
        .expect("query table info")
        .map(|row| row.expect("read column name"))
        .any(|name| name == column)
}

#[test]
fn fresh_file_database_initialization_seeds_schema_and_defaults_and_reopens() {
    let path = temp_db_path("fresh");
    remove_sqlite_files(&path);

    {
        let conn = Connection::open(&path).expect("open fresh sqlite file");
        super::migrations::initialize_connection(&conn).expect("initialize database");

        let foreign_keys: i64 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("read foreign_keys pragma");
        assert_eq!(foreign_keys, 1);

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
            assert!(table_exists(&conn, table), "missing table {table}");
        }

        let browser_mode: String = conn
            .query_row(
                "SELECT value FROM app_config WHERE key = 'browser_title_mode'",
                [],
                |row| row.get(0),
            )
            .expect("read default browser mode");
        assert_eq!(browser_mode, "BLUR");

        let code_rule: String = conn
            .query_row(
                "SELECT mapped_type FROM app_rules WHERE process_name = 'code.exe'",
                [],
                |row| row.get(0),
            )
            .expect("read default code rule");
        assert_eq!(code_rule, "LEARN");
    }

    {
        let reopened = Connection::open(&path).expect("reopen initialized sqlite file");
        assert!(table_exists(&reopened, "app_rules"));
        let rule_count: i64 = reopened
            .query_row("SELECT COUNT(*) FROM app_rules", [], |row| row.get(0))
            .expect("count persisted rules");
        assert!(rule_count >= 10);
    }

    remove_sqlite_files(&path);
}

#[test]
fn database_initialization_is_idempotent_and_preserves_user_values() {
    let path = temp_db_path("idempotent");
    remove_sqlite_files(&path);
    let conn = Connection::open(&path).expect("open sqlite file");

    super::migrations::initialize_connection(&conn).expect("first initialization");
    conn.execute(
        "UPDATE app_config SET value = 'NONE' WHERE key = 'browser_title_mode'",
        [],
    )
    .expect("customize browser title mode");
    conn.execute(
        "UPDATE app_rules SET mapped_type = 'REST' WHERE process_name = 'code.exe'",
        [],
    )
    .expect("customize code rule");

    super::migrations::initialize_connection(&conn).expect("second initialization");

    let browser_mode: String = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = 'browser_title_mode'",
            [],
            |row| row.get(0),
        )
        .expect("read customized browser mode");
    assert_eq!(browser_mode, "NONE");

    let code_rule: String = conn
        .query_row(
            "SELECT mapped_type FROM app_rules WHERE process_name = 'code.exe'",
            [],
            |row| row.get(0),
        )
        .expect("read customized code rule");
    assert_eq!(code_rule, "REST");

    let root_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM categories WHERE id IN (1, 2)",
            [],
            |row| row.get(0),
        )
        .expect("count root categories");
    assert_eq!(root_count, 2);

    drop(conn);
    remove_sqlite_files(&path);
}

#[test]
fn legacy_schema_is_upgraded_without_losing_existing_rows() {
    let path = temp_db_path("legacy-schema");
    remove_sqlite_files(&path);
    let conn = Connection::open(&path).expect("open legacy sqlite file");

    conn.execute_batch(
        r#"
        CREATE TABLE app_rules (
            process_name TEXT PRIMARY KEY,
            mapped_type TEXT NOT NULL,
            privacy_level TEXT NOT NULL
        );
        INSERT INTO app_rules (process_name, mapped_type, privacy_level)
        VALUES ('legacy.exe', 'LEARN', 'NORMAL');

        CREATE TABLE reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            repeat_rule TEXT NOT NULL,
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
            content, repeat_rule, remind_at, daily_time_minutes, is_completed,
            completed_day_key, completed_at, snooze_until, created_at, updated_at
        ) VALUES ('legacy reminder', 'DAILY', NULL, 540, 0, NULL, NULL, NULL, 100, 100);
        "#,
    )
    .expect("create legacy schema");

    super::migrations::initialize_connection(&conn).expect("migrate legacy schema");

    assert!(column_exists(&conn, "app_rules", "created_at"));
    assert!(column_exists(&conn, "app_rules", "updated_at"));
    assert!(column_exists(&conn, "reminders", "weekly_days"));
    assert!(column_exists(&conn, "reminders", "sort_order"));

    let mapped_type: String = conn
        .query_row(
            "SELECT mapped_type FROM app_rules WHERE process_name = 'legacy.exe'",
            [],
            |row| row.get(0),
        )
        .expect("legacy app rule survived migration");
    assert_eq!(mapped_type, "LEARN");

    let reminder: (String, i64) = conn
        .query_row(
            "SELECT content, daily_time_minutes FROM reminders WHERE content = 'legacy reminder'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("legacy reminder survived migration");
    assert_eq!(reminder, ("legacy reminder".to_string(), 540));

    drop(conn);
    remove_sqlite_files(&path);
}

#[test]
fn legacy_database_file_is_copied_once_to_preferred_location() {
    let legacy_path = temp_db_path("legacy-file");
    let preferred_path = temp_db_path("preferred-file");
    remove_sqlite_files(&legacy_path);
    remove_sqlite_files(&preferred_path);

    {
        let legacy = Connection::open(&legacy_path).expect("create legacy sqlite file");
        legacy
            .execute_batch(
                "CREATE TABLE marker (value TEXT NOT NULL); INSERT INTO marker VALUES ('legacy');",
            )
            .expect("seed legacy database");
    }

    super::connection::migrate_legacy_db_files(&legacy_path, &preferred_path)
        .expect("copy legacy database");

    {
        let preferred = Connection::open(&preferred_path).expect("open copied preferred db");
        let marker: String = preferred
            .query_row("SELECT value FROM marker", [], |row| row.get(0))
            .expect("read copied marker");
        assert_eq!(marker, "legacy");
        preferred
            .execute("UPDATE marker SET value = 'preferred'", [])
            .expect("customize preferred database");
    }

    {
        let legacy = Connection::open(&legacy_path).expect("reopen legacy db");
        legacy
            .execute("UPDATE marker SET value = 'changed-legacy'", [])
            .expect("change legacy database");
    }

    super::connection::migrate_legacy_db_files(&legacy_path, &preferred_path)
        .expect("second legacy migration is a no-op");

    let preferred = Connection::open(&preferred_path).expect("reopen preferred db");
    let marker: String = preferred
        .query_row("SELECT value FROM marker", [], |row| row.get(0))
        .expect("read preserved preferred marker");
    assert_eq!(marker, "preferred");

    drop(preferred);
    remove_sqlite_files(&legacy_path);
    remove_sqlite_files(&preferred_path);
}
