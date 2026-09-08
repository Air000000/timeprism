use rusqlite::{params, Connection};

use crate::domain::tracking::TrackingState;

fn read_bool_config(conn: &Connection, key: &str) -> Result<bool, String> {
    let value: String = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .map_err(|e| format!("failed to read tracking config {key}: {e}"))?;

    match value.trim().to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(format!("invalid boolean tracking config {key}: {other}")),
    }
}

pub(crate) fn get_tracking_state_entry(conn: &Connection) -> Result<TrackingState, String> {
    Ok(TrackingState {
        onboarding_completed: read_bool_config(conn, "onboarding_completed")?,
        auto_capture_enabled: read_bool_config(conn, "auto_capture_enabled")?,
    })
}

pub(crate) fn complete_onboarding_entry(conn: &Connection) -> Result<TrackingState, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("failed to begin onboarding transaction: {e}"))?;

    let changed = tx
        .execute(
            "UPDATE app_config
             SET value = 'true'
             WHERE key IN ('onboarding_completed', 'auto_capture_enabled')",
            [],
        )
        .map_err(|e| format!("failed to complete onboarding: {e}"))?;
    if changed != 2 {
        return Err(format!(
            "failed to complete onboarding: expected 2 tracking config rows, updated {changed}"
        ));
    }

    tx.commit()
        .map_err(|e| format!("failed to commit onboarding: {e}"))?;

    get_tracking_state_entry(conn)
}

pub(crate) fn set_auto_capture_enabled_entry(
    conn: &Connection,
    enabled: bool,
) -> Result<TrackingState, String> {
    let current = get_tracking_state_entry(conn)?;
    if enabled && !current.onboarding_completed {
        return Err("cannot enable auto capture before onboarding is complete".to_string());
    }

    let changed = conn
        .execute(
            "UPDATE app_config SET value = ?1 WHERE key = 'auto_capture_enabled'",
            params![enabled.to_string()],
        )
        .map_err(|e| format!("failed to update auto capture state: {e}"))?;
    if changed != 1 {
        return Err(format!(
            "failed to update auto capture state: expected 1 config row, updated {changed}"
        ));
    }

    get_tracking_state_entry(conn)
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::{
        complete_onboarding_entry, get_tracking_state_entry, set_auto_capture_enabled_entry,
    };

    fn tracking_conn(onboarding: bool, enabled: bool) -> Connection {
        let conn = Connection::open_in_memory().expect("open tracking test db");
        conn.execute_batch(
            "CREATE TABLE app_config (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .expect("create app_config");
        for (key, value) in [
            ("onboarding_completed", onboarding.to_string()),
            ("auto_capture_enabled", enabled.to_string()),
        ] {
            conn.execute(
                "INSERT INTO app_config (key, value) VALUES (?1, ?2)",
                params![key, value],
            )
            .expect("seed tracking state");
        }
        conn
    }

    #[test]
    fn completing_onboarding_atomically_enables_capture() {
        let conn = tracking_conn(false, false);
        let state = complete_onboarding_entry(&conn).expect("complete onboarding");
        assert!(state.onboarding_completed);
        assert!(state.auto_capture_enabled);
        assert_eq!(get_tracking_state_entry(&conn).unwrap(), state);
    }

    #[test]
    fn pause_and_resume_persist_after_onboarding() {
        let conn = tracking_conn(true, true);
        assert!(!set_auto_capture_enabled_entry(&conn, false)
            .unwrap()
            .auto_capture_enabled);
        assert!(set_auto_capture_enabled_entry(&conn, true)
            .unwrap()
            .auto_capture_enabled);
    }

    #[test]
    fn enabling_capture_before_onboarding_is_rejected() {
        let conn = tracking_conn(false, false);
        let err = set_auto_capture_enabled_entry(&conn, true).unwrap_err();
        assert!(err.contains("onboarding"));
        assert!(!get_tracking_state_entry(&conn)
            .unwrap()
            .auto_capture_enabled);
    }

    #[test]
    fn completing_onboarding_rejects_missing_tracking_key() {
        let conn = tracking_conn(false, false);
        conn.execute(
            "DELETE FROM app_config WHERE key = 'auto_capture_enabled'",
            [],
        )
        .expect("remove tracking key");

        let err = complete_onboarding_entry(&conn).unwrap_err();
        assert!(err.contains("expected 2 tracking config rows"));
    }
}
