use chrono::Local;
use rusqlite::{params, Connection};
use tauri::AppHandle;

use super::connection::open_connection;

fn ensure_app_rules_time_columns(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(app_rules)")
        .map_err(|e| format!("failed to prepare app_rules table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query app_rules table info rows: {e}"))?;

    let mut has_created = false;
    let mut has_updated = false;
    for row in rows {
        let col = row.map_err(|e| format!("failed to parse app_rules table info row: {e}"))?;
        if col == "created_at" {
            has_created = true;
        }
        if col == "updated_at" {
            has_updated = true;
        }
    }

    if !has_created {
        conn.execute(
            "ALTER TABLE app_rules ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| format!("failed to add created_at to app_rules: {e}"))?;
    }
    if !has_updated {
        conn.execute(
            "ALTER TABLE app_rules ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| format!("failed to add updated_at to app_rules: {e}"))?;
    }

    let now_ts = Local::now().timestamp();
    conn.execute(
        "UPDATE app_rules
         SET created_at = CASE WHEN created_at <= 0 THEN ?1 ELSE created_at END,
             updated_at = CASE WHEN updated_at <= 0 THEN ?1 ELSE updated_at END",
        [now_ts],
    )
    .map_err(|e| format!("failed to backfill app_rules timestamps: {e}"))?;

    Ok(())
}

fn ensure_reminders_weekly_columns(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(reminders)")
        .map_err(|e| format!("failed to prepare reminders table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query reminders table info rows: {e}"))?;

    let mut has_weekly_days = false;
    for row in rows {
        let col = row.map_err(|e| format!("failed to parse reminders table info row: {e}"))?;
        if col == "weekly_days" {
            has_weekly_days = true;
        }
    }

    if !has_weekly_days {
        conn.execute_batch(
            r#"
            ALTER TABLE reminders RENAME TO reminders_legacy;
            CREATE TABLE reminders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE', 'DAILY', 'WEEKLY')),
                sort_order INTEGER NOT NULL DEFAULT 0,
                remind_at INTEGER NULL,
                daily_time_minutes INTEGER NULL,
                weekly_days TEXT NULL,
                is_completed INTEGER NOT NULL DEFAULT 0,
                completed_day_key TEXT NULL,
                completed_at INTEGER NULL,
                snooze_until INTEGER NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            INSERT INTO reminders (
                id, content, repeat_rule, sort_order, remind_at, daily_time_minutes, weekly_days,
                is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at
            )
            SELECT
                id, content, repeat_rule, 0, remind_at, daily_time_minutes, NULL,
                is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at
            FROM reminders_legacy;
            DROP TABLE reminders_legacy;
            "#,
        )
        .map_err(|e| format!("failed to migrate reminders for weekly support: {e}"))?;
        conn.execute_batch(
            r#"
            CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
            CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
            CREATE INDEX IF NOT EXISTS idx_reminders_due_weekly ON reminders(repeat_rule, daily_time_minutes, weekly_days, completed_day_key);
            "#,
        )
        .map_err(|e| format!("failed to recreate reminders indexes after migration: {e}"))?;
    }

    Ok(())
}

fn ensure_reminders_sort_order(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("PRAGMA table_info(reminders)")
        .map_err(|e| format!("failed to prepare reminders sort-order table info query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| format!("failed to query reminders sort-order table info rows: {e}"))?;

    let mut has_sort_order = false;
    for row in rows {
        let col =
            row.map_err(|e| format!("failed to parse reminders sort-order table info row: {e}"))?;
        if col == "sort_order" {
            has_sort_order = true;
            break;
        }
    }

    if !has_sort_order {
        conn.execute(
            "ALTER TABLE reminders ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0",
            [],
        )
        .map_err(|e| format!("failed to add sort_order to reminders: {e}"))?;
    }

    let mut order_stmt = conn
        .prepare("SELECT id FROM reminders ORDER BY sort_order ASC, created_at ASC, id ASC")
        .map_err(|e| format!("failed to prepare reminder sort backfill query: {e}"))?;
    let ids = order_stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| format!("failed to query reminder ids for sort backfill: {e}"))?;

    for (index, row) in ids.enumerate() {
        let id = row.map_err(|e| format!("failed to parse reminder id for sort backfill: {e}"))?;
        conn.execute(
            "UPDATE reminders SET sort_order = ?1 WHERE id = ?2",
            params![index as i64, id],
        )
        .map_err(|e| format!("failed to backfill reminder sort order: {e}"))?;
    }

    Ok(())
}

pub fn ensure_heatmap_snapshot_table(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS daily_heatmap_snapshot (
            day_key TEXT PRIMARY KEY,
            learn_seconds INTEGER NOT NULL,
            goal_seconds INTEGER NOT NULL,
            level TEXT NOT NULL CHECK(level IN ('GRAY', 'YELLOW', 'GREEN')),
            sealed_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_daily_heatmap_snapshot_day_key ON daily_heatmap_snapshot(day_key);",
    )
    .map_err(|e| format!("failed to ensure daily_heatmap_snapshot table: {e}"))?;
    Ok(())
}

fn has_recognizable_timeprism_schema(conn: &Connection) -> Result<bool, String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*)
             FROM sqlite_master
             WHERE type = 'table'
               AND name IN ('categories', 'task_sessions', 'app_usage_logs', 'app_rules', 'app_config', 'reminders')",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed to inspect existing TimePrism schema: {e}"))?;
    Ok(count > 0)
}

pub(super) fn initialize_connection(conn: &Connection) -> Result<(), String> {
    let was_existing_timeprism_db = has_recognizable_timeprism_schema(conn)?;

    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("failed to enable foreign keys: {e}"))?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS categories (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          parent_id INTEGER NULL REFERENCES categories(id),
          name TEXT NOT NULL,
          color_hex TEXT NOT NULL DEFAULT '#4ade80',
          root_type TEXT NOT NULL CHECK(root_type IN ('LEARN', 'REST'))
        );

        CREATE TABLE IF NOT EXISTS task_sessions (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          category_id INTEGER NOT NULL REFERENCES categories(id),
          start_time INTEGER NOT NULL,
          end_time INTEGER NULL,
          is_flow_target INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS app_usage_logs (
          id INTEGER PRIMARY KEY AUTOINCREMENT,
          process_name TEXT NOT NULL,
          window_title TEXT NOT NULL,
          start_timestamp INTEGER NOT NULL,
          duration_ms INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_rules (
          process_name TEXT PRIMARY KEY,
          mapped_type TEXT NOT NULL CHECK(mapped_type IN ('LEARN', 'REST', 'IGNORE')),
          privacy_level TEXT NOT NULL CHECK(privacy_level IN ('NORMAL', 'BLUR_TITLE', 'WHITELIST_ONLY'))
        );

        CREATE TABLE IF NOT EXISTS app_config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_whitelist (
            process_name TEXT PRIMARY KEY
        );

        CREATE TABLE IF NOT EXISTS reminders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            repeat_rule TEXT NOT NULL CHECK(repeat_rule IN ('NONE', 'DAILY', 'WEEKLY')),
            sort_order INTEGER NOT NULL DEFAULT 0,
            remind_at INTEGER NULL,
            daily_time_minutes INTEGER NULL,
            weekly_days TEXT NULL,
            is_completed INTEGER NOT NULL DEFAULT 0,
            completed_day_key TEXT NULL,
            completed_at INTEGER NULL,
            snooze_until INTEGER NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_task_sessions_start_time ON task_sessions(start_time);
        CREATE INDEX IF NOT EXISTS idx_app_usage_logs_start ON app_usage_logs(start_timestamp);
        CREATE INDEX IF NOT EXISTS idx_app_usage_logs_process ON app_usage_logs(process_name);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
        "#,
    )
    .map_err(|e| format!("failed to run schema init: {e}"))?;

    ensure_app_rules_time_columns(conn)?;
    ensure_heatmap_snapshot_table(conn)?;
    ensure_reminders_weekly_columns(conn)?;
    ensure_reminders_sort_order(conn)?;
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_reminders_due_none ON reminders(repeat_rule, remind_at, is_completed);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_daily ON reminders(repeat_rule, daily_time_minutes, completed_day_key);
        CREATE INDEX IF NOT EXISTS idx_reminders_due_weekly ON reminders(repeat_rule, daily_time_minutes, weekly_days, completed_day_key);
        "#,
    )
    .map_err(|e| format!("failed to ensure reminder indexes: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO categories (id, parent_id, name, color_hex, root_type) VALUES (1, NULL, '学习', '#22c55e', 'LEARN')",
        [],
    )
    .map_err(|e| format!("failed to seed LEARN root: {e}"))?;

    conn.execute(
        "INSERT OR IGNORE INTO categories (id, parent_id, name, color_hex, root_type) VALUES (2, NULL, '休息', '#f97316', 'REST')",
        [],
    )
    .map_err(|e| format!("failed to seed REST root: {e}"))?;

    conn.execute(
        "UPDATE categories SET name = '学习' WHERE id = 1 AND root_type = 'LEARN' AND name = '瀛︿範'",
        [],
    )
    .map_err(|e| format!("failed to repair LEARN root name: {e}"))?;

    conn.execute(
        "UPDATE categories SET name = '休息' WHERE id = 2 AND root_type = 'REST' AND name = '浼戞伅'",
        [],
    )
    .map_err(|e| format!("failed to repair REST root name: {e}"))?;

    let default_rules = [
        ("code.exe", "LEARN", "NORMAL"),
        ("pycharm64.exe", "LEARN", "NORMAL"),
        ("idea64.exe", "LEARN", "NORMAL"),
        ("devenv.exe", "LEARN", "NORMAL"),
        ("desktop.shell.exe", "IGNORE", "NORMAL"),
        ("steam.exe", "REST", "NORMAL"),
        ("dota2.exe", "REST", "NORMAL"),
        ("bilibili.exe", "REST", "NORMAL"),
        ("msedge.exe", "IGNORE", "BLUR_TITLE"),
        ("chrome.exe", "IGNORE", "BLUR_TITLE"),
    ];

    let now_ts = Local::now().timestamp();
    for (process_name, mapped_type, privacy_level) in default_rules {
        conn.execute(
            "INSERT OR IGNORE INTO app_rules (process_name, mapped_type, privacy_level, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![process_name, mapped_type, privacy_level, now_ts],
        )
        .map_err(|e| format!("failed to seed app rule {process_name}: {e}"))?;
    }

    let default_config = [
        ("curtain_enabled", "false"),
        ("browser_title_mode", "BLUR"),
        ("browser_blur_enabled", "true"),
        ("whitelist_only_enabled", "false"),
        ("heatmap_goal_seconds", "7200"),
    ];
    for (key, value) in default_config {
        conn.execute(
            "INSERT OR IGNORE INTO app_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("failed to seed app config {key}: {e}"))?;
    }

    let tracking_defaults = if was_existing_timeprism_db {
        [
            ("onboarding_completed", "true"),
            ("auto_capture_enabled", "true"),
        ]
    } else {
        [
            ("onboarding_completed", "false"),
            ("auto_capture_enabled", "false"),
        ]
    };
    for (key, value) in tracking_defaults {
        conn.execute(
            "INSERT OR IGNORE INTO app_config (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("failed to seed tracking config {key}: {e}"))?;
    }

    let default_whitelist = [
        "code.exe",
        "pycharm64.exe",
        "idea64.exe",
        "devenv.exe",
        "explorer.exe",
    ];
    for process_name in default_whitelist {
        conn.execute(
            "INSERT OR IGNORE INTO app_whitelist (process_name) VALUES (?1)",
            params![process_name],
        )
        .map_err(|e| format!("failed to seed whitelist {process_name}: {e}"))?;
    }

    Ok(())
}

pub fn init_database(app: &AppHandle) -> Result<(), String> {
    let conn = open_connection(app)?;
    initialize_connection(&conn)
}
