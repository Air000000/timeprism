use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

fn ensure_data_dir(dir: &Path, label: &str) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("failed to create {label} dir {}: {e}", dir.display()))
}

fn legacy_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve legacy app data dir: {e}"))?;
    ensure_data_dir(&data_dir, "legacy app data")?;
    Ok(data_dir.join("timeprism.db"))
}

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("failed to resolve app local data dir: {e}"))?;
    ensure_data_dir(&data_dir, "app local data")?;
    Ok(data_dir.join("timeprism.db"))
}

pub(super) fn migrate_legacy_db_files(
    legacy_path: &Path,
    target_path: &Path,
) -> Result<(), String> {
    if target_path.exists() || !legacy_path.exists() {
        return Ok(());
    }

    fs::copy(legacy_path, target_path).map_err(|e| {
        format!(
            "failed to migrate legacy sqlite db from {} to {}: {e}",
            legacy_path.display(),
            target_path.display()
        )
    })?;

    for suffix in ["-wal", "-shm"] {
        let from = PathBuf::from(format!("{}{}", legacy_path.display(), suffix));
        if from.exists() {
            let to = PathBuf::from(format!("{}{}", target_path.display(), suffix));
            fs::copy(&from, &to).map_err(|e| {
                format!(
                    "failed to migrate sqlite sidecar from {} to {}: {e}",
                    from.display(),
                    to.display()
                )
            })?;
        }
    }

    Ok(())
}

fn try_migrate_legacy_db(app: &AppHandle, target_path: &Path) -> Result<(), String> {
    let legacy_path = legacy_db_path(app)?;
    migrate_legacy_db_files(&legacy_path, target_path)
}

pub fn open_connection(app: &AppHandle) -> Result<Connection, String> {
    let preferred_path = db_path(app)?;
    let migration_error = try_migrate_legacy_db(app, &preferred_path).err();
    let legacy_path = legacy_db_path(app)?;

    if let Some(err) = migration_error {
        return Connection::open(&legacy_path).map_err(|fallback_err| {
            format!(
                "failed to migrate legacy sqlite db to preferred location: {err}; fallback legacy db {} also failed: {fallback_err}",
                legacy_path.display()
            )
        });
    }

    match Connection::open(&preferred_path) {
        Ok(conn) => Ok(conn),
        Err(primary_err) => Connection::open(&legacy_path).map_err(|fallback_err| {
            format!(
                "failed to open preferred sqlite db {}: {primary_err}; fallback legacy db {} also failed: {fallback_err}",
                preferred_path.display(),
                legacy_path.display()
            )
        }),
    }
}
