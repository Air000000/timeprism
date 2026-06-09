use std::path::Path;

use rusqlite::Connection;

use crate::domain::privacy::{
    PrivacySettings, SetWhitelistItemInput, UpdatePrivacySettingsInput,
};

pub(crate) fn parse_bool_config(conn: &Connection, key: &str, default_value: bool) -> bool {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .map(|v| v.eq_ignore_ascii_case("true"))
    .unwrap_or(default_value)
}

pub(crate) fn normalize_process_key(process_name: &str) -> String {
    let normalized = process_name
        .trim()
        .trim_matches('"')
        .replace('\\', "/")
        .to_lowercase();

    let from_path = Path::new(&normalized)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string());

    from_path
        .or_else(|| {
            let trimmed = normalized.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_else(|| "unknown.exe".to_string())
}

pub(crate) fn parse_browser_title_mode(conn: &Connection) -> String {
    let from_mode_key = conn
        .query_row(
            "SELECT value FROM app_config WHERE key = 'browser_title_mode'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok()
        .map(|v| v.trim().to_uppercase())
        .filter(|v| matches!(v.as_str(), "FULL" | "BLUR" | "NONE"));

    if let Some(mode) = from_mode_key {
        return mode;
    }

    if parse_bool_config(conn, "browser_blur_enabled", true) {
        "BLUR".to_string()
    } else {
        "FULL".to_string()
    }
}

fn is_browser_process(process_name: &str) -> bool {
    matches!(
        process_name,
        "chrome.exe"
            | "msedge.exe"
            | "firefox.exe"
            | "opera.exe"
            | "brave.exe"
            | "vivaldi.exe"
            | "iexplore.exe"
    )
}

fn contains_incognito_keyword(title: &str) -> bool {
    let lower = title.to_lowercase();
    lower.contains("incognito")
        || lower.contains("inprivate")
        || lower.contains("private browsing")
        || lower.contains("无痕")
        || lower.contains("隐私")
}

fn is_desktop_shell_window(process_name: &str, window_title: &str) -> bool {
    if process_name != "explorer.exe" {
        return false;
    }
    let t = window_title.trim().to_lowercase();
    t.is_empty()
        || t == "program manager"
        || t.contains("workerw")
        || t.contains("desktop")
        || t.contains("桌面")
}

pub(crate) fn process_log_with_privacy(
    conn: &Connection,
    process_name: &str,
    window_title: &str,
) -> Result<(Option<(String, String)>, Option<String>), String> {
    let curtain_enabled = parse_bool_config(conn, "curtain_enabled", false);
    if curtain_enabled {
        return Ok((None, Some("curtain_enabled".to_string())));
    }

    let mut final_process = normalize_process_key(process_name);
    let title_raw = window_title.trim();
    let mut final_title = if title_raw.is_empty() {
        "Untitled Window".to_string()
    } else {
        title_raw.to_string()
    };

    let browser_title_mode = parse_browser_title_mode(conn);
    let whitelist_only_enabled = parse_bool_config(conn, "whitelist_only_enabled", false);
    let is_browser = is_browser_process(&final_process);

    if is_browser && contains_incognito_keyword(title_raw) {
        return Ok((None, Some("incognito_window".to_string())));
    }

    if is_desktop_shell_window(&final_process, &final_title) {
        final_process = "desktop.shell.exe".to_string();
        final_title = "Desktop Shell".to_string();
    }

    let process_privacy = conn
        .query_row(
            "SELECT privacy_level FROM app_rules WHERE process_name = ?1",
            [final_process.clone()],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| "NORMAL".to_string())
        .to_uppercase();

    if is_browser {
        match browser_title_mode.as_str() {
            "BLUR" => {
                final_title = "Web Browser".to_string();
            }
            "NONE" => {
                final_title = "Not Collected".to_string();
            }
            _ => {}
        }
    }

    if process_privacy == "BLUR_TITLE" {
        final_title = "Hidden Window".to_string();
    }

    if whitelist_only_enabled || process_privacy == "WHITELIST_ONLY" {
        let whitelisted = conn
            .query_row(
                "SELECT 1 FROM app_whitelist WHERE process_name = ?1 LIMIT 1",
                [final_process.clone()],
                |_row| Ok(true),
            )
            .unwrap_or(false);

        if !whitelisted {
            final_process = "uncategorized.exe".to_string();
            final_title = "Hidden by Whitelist".to_string();
            return Ok((
                Some((final_process, final_title)),
                Some("whitelist_blocked".to_string()),
            ));
        }
    }

    Ok((Some((final_process, final_title)), None))
}

pub(crate) fn get_privacy_settings_entry(conn: &Connection) -> Result<PrivacySettings, String> {
    Ok(PrivacySettings {
        curtain_enabled: parse_bool_config(conn, "curtain_enabled", false),
        browser_title_mode: parse_browser_title_mode(conn),
        whitelist_only_enabled: parse_bool_config(conn, "whitelist_only_enabled", false),
    })
}

pub(crate) fn update_privacy_settings_entry(
    conn: &Connection,
    input: UpdatePrivacySettingsInput,
) -> Result<(), String> {
    let browser_title_mode = input.browser_title_mode.trim().to_uppercase();
    if !matches!(browser_title_mode.as_str(), "FULL" | "BLUR" | "NONE") {
        return Err("browser_title_mode must be FULL, BLUR, or NONE".to_string());
    }

    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("failed to begin privacy settings transaction: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('curtain_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if input.curtain_enabled { "true" } else { "false" }],
    )
    .map_err(|e| format!("failed to save curtain setting: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('browser_title_mode', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [browser_title_mode.as_str()],
    )
    .map_err(|e| format!("failed to save browser title mode: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('browser_blur_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if browser_title_mode == "FULL" { "false" } else { "true" }],
    )
    .map_err(|e| format!("failed to save browser blur setting: {e}"))?;

    tx.execute(
        "INSERT INTO app_config (key, value) VALUES ('whitelist_only_enabled', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [if input.whitelist_only_enabled {
            "true"
        } else {
            "false"
        }],
    )
    .map_err(|e| format!("failed to save whitelist-only setting: {e}"))?;

    tx.commit()
        .map_err(|e| format!("failed to commit privacy settings transaction: {e}"))?;
    Ok(())
}

pub(crate) fn list_whitelist_entries(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT process_name FROM app_whitelist ORDER BY process_name")
        .map_err(|e| format!("failed to prepare whitelist query: {e}"))?;

    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| format!("failed to query whitelist rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse whitelist row: {e}"))?);
    }
    Ok(result)
}

pub(crate) fn set_whitelist_item_entry(
    conn: &Connection,
    input: SetWhitelistItemInput,
) -> Result<(), String> {
    let process_name = normalize_process_key(&input.process_name);
    if process_name.is_empty() {
        return Err("process_name cannot be empty".to_string());
    }

    if input.enabled {
        conn.execute(
            "INSERT OR IGNORE INTO app_whitelist (process_name) VALUES (?1)",
            [process_name],
        )
        .map_err(|e| format!("failed to add whitelist item: {e}"))?;
    } else {
        conn.execute(
            "DELETE FROM app_whitelist WHERE process_name = ?1",
            [process_name],
        )
        .map_err(|e| format!("failed to remove whitelist item: {e}"))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        contains_incognito_keyword, is_browser_process, is_desktop_shell_window,
        normalize_process_key, process_log_with_privacy,
    };

    fn privacy_test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory privacy db");
        conn.execute_batch(
            r#"
            CREATE TABLE app_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE app_rules (
                process_name TEXT PRIMARY KEY,
                mapped_type TEXT NOT NULL,
                privacy_level TEXT NOT NULL
            );

            CREATE TABLE app_whitelist (
                process_name TEXT PRIMARY KEY
            );
            "#,
        )
        .expect("create privacy test schema");
        conn
    }

    fn set_config(conn: &Connection, key: &str, value: &str) {
        conn.execute(
            "INSERT INTO app_config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            [key, value],
        )
        .expect("set config");
    }

    #[test]
    fn normalizes_paths_and_quotes_to_process_key() {
        assert_eq!(normalize_process_key(r#""C:\Program Files\App\Code.EXE""#), "code.exe");
        assert_eq!(normalize_process_key("  "), "unknown.exe");
        assert_eq!(normalize_process_key("plain.exe"), "plain.exe");
    }

    #[test]
    fn recognizes_browser_processes() {
        assert!(is_browser_process("chrome.exe"));
        assert!(is_browser_process("msedge.exe"));
        assert!(!is_browser_process("code.exe"));
    }

    #[test]
    fn recognizes_private_browser_titles() {
        assert!(contains_incognito_keyword("Example - Incognito"));
        assert!(contains_incognito_keyword("Example - InPrivate"));
        assert!(!contains_incognito_keyword("Normal Window"));
    }

    #[test]
    fn recognizes_desktop_shell_windows() {
        assert!(is_desktop_shell_window("explorer.exe", ""));
        assert!(is_desktop_shell_window("explorer.exe", "Program Manager"));
        assert!(!is_desktop_shell_window("explorer.exe", "Documents"));
        assert!(!is_desktop_shell_window("code.exe", "Desktop"));
    }

    #[test]
    fn browser_blur_mode_generalizes_titles() {
        let conn = privacy_test_conn();
        set_config(&conn, "browser_title_mode", "BLUR");

        let (processed, reason) =
            process_log_with_privacy(&conn, "chrome.exe", "Sensitive Page").expect("process log");

        assert_eq!(processed, Some(("chrome.exe".to_string(), "Web Browser".to_string())));
        assert_eq!(reason, None);
    }

    #[test]
    fn browser_none_mode_uses_safe_placeholder() {
        let conn = privacy_test_conn();
        set_config(&conn, "browser_title_mode", "NONE");

        let (processed, reason) =
            process_log_with_privacy(&conn, "msedge.exe", "Sensitive Page").expect("process log");

        assert_eq!(
            processed,
            Some(("msedge.exe".to_string(), "Not Collected".to_string()))
        );
        assert_eq!(reason, None);
    }

    #[test]
    fn private_browser_titles_are_blocked() {
        let conn = privacy_test_conn();

        let (processed, reason) =
            process_log_with_privacy(&conn, "chrome.exe", "Example - Incognito").expect("process log");

        assert_eq!(processed, None);
        assert_eq!(reason, Some("incognito_window".to_string()));
    }

    #[test]
    fn whitelist_only_hides_unlisted_processes_and_allows_listed_processes() {
        let conn = privacy_test_conn();
        set_config(&conn, "whitelist_only_enabled", "true");

        let (blocked, block_reason) =
            process_log_with_privacy(&conn, "code.exe", "Project").expect("process blocked log");
        assert_eq!(
            blocked,
            Some((
                "uncategorized.exe".to_string(),
                "Hidden by Whitelist".to_string()
            ))
        );
        assert_eq!(block_reason, Some("whitelist_blocked".to_string()));

        conn.execute(
            "INSERT INTO app_whitelist (process_name) VALUES (?1)",
            ["code.exe"],
        )
        .expect("whitelist code.exe");

        let (allowed, allow_reason) =
            process_log_with_privacy(&conn, "code.exe", "Project").expect("process allowed log");
        assert_eq!(allowed, Some(("code.exe".to_string(), "Project".to_string())));
        assert_eq!(allow_reason, None);
    }
}
