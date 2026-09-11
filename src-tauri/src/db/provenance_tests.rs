use rusqlite::Connection;

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("prepare table info");
    let has_column = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query table info")
        .map(|row| row.expect("read column name"))
        .any(|name| name == column);
    has_column
}

#[test]
fn fresh_usage_schema_records_foreground_source_by_default() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    super::migrations::initialize_connection(&conn).expect("initialize fresh database");

    assert!(column_exists(&conn, "app_usage_logs", "source"));

    conn.execute(
        "INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
         VALUES ('code.exe', 'Project', 100, 5000)",
        [],
    )
    .expect("insert usage row with schema default");

    let source: String = conn
        .query_row(
            "SELECT source FROM app_usage_logs WHERE process_name = 'code.exe'",
            [],
            |row| row.get(0),
        )
        .expect("read default source");
    assert_eq!(source, "FOREGROUND");
}

#[test]
fn legacy_usage_rows_gain_source_and_known_idle_rows_are_backfilled() {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    conn.execute_batch(
        r#"
        CREATE TABLE app_usage_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            process_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            start_timestamp INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL
        );
        INSERT INTO app_usage_logs (process_name, window_title, start_timestamp, duration_ms)
        VALUES
            ('code.exe', 'Project', 100, 5000),
            ('__idle_learn__.exe', 'Idle Segment · Learn', 200, 5000),
            ('code.exe', 'Idle Confirmed · Previous App', 300, 5000);
        "#,
    )
    .expect("create pre-provenance usage schema");

    super::migrations::initialize_connection(&conn).expect("upgrade database");

    assert!(column_exists(&conn, "app_usage_logs", "source"));

    let mut stmt = conn
        .prepare(
            "SELECT process_name, window_title, source
             FROM app_usage_logs
             ORDER BY id",
        )
        .expect("prepare migrated usage query");
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .expect("query migrated usage rows")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect migrated usage rows");

    assert_eq!(rows[0].2, "FOREGROUND");
    assert_eq!(rows[1].2, "IDLE_CONFIRMED");
    assert_eq!(rows[2].2, "IDLE_CONFIRMED");
}
