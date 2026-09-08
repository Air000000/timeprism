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
}
