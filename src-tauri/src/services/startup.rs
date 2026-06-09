#[cfg(target_os = "windows")]
use std::process::Command;

#[cfg(target_os = "windows")]
const WINDOWS_RUN_REGISTRY_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const WINDOWS_RUN_VALUE_NAME: &str = "TimePrism";

#[cfg(target_os = "windows")]
pub(crate) fn get_auto_start_enabled_state() -> Result<bool, String> {
    let output = Command::new("reg")
        .args(["query", WINDOWS_RUN_REGISTRY_PATH, "/v", WINDOWS_RUN_VALUE_NAME])
        .output()
        .map_err(|e| format!("failed to query Windows startup entry: {e}"))?;

    Ok(output.status.success())
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn get_auto_start_enabled_state() -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "windows")]
pub(crate) fn set_auto_start_enabled_state(enabled: bool) -> Result<bool, String> {
    if enabled {
        let exe = std::env::current_exe()
            .map_err(|e| format!("failed to locate current executable: {e}"))?;
        let exe_arg = format!("\"{}\"", exe.display());
        let status = Command::new("reg")
            .args([
                "add",
                WINDOWS_RUN_REGISTRY_PATH,
                "/v",
                WINDOWS_RUN_VALUE_NAME,
                "/t",
                "REG_SZ",
                "/d",
                exe_arg.as_str(),
                "/f",
            ])
            .status()
            .map_err(|e| format!("failed to enable Windows startup entry: {e}"))?;
        if !status.success() {
            return Err("failed to enable Windows startup entry".to_string());
        }
    } else {
        let status = Command::new("reg")
            .args([
                "delete",
                WINDOWS_RUN_REGISTRY_PATH,
                "/v",
                WINDOWS_RUN_VALUE_NAME,
                "/f",
            ])
            .status()
            .map_err(|e| format!("failed to disable Windows startup entry: {e}"))?;
        if !status.success() {
            return Ok(false);
        }
    }

    get_auto_start_enabled_state()
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn set_auto_start_enabled_state(_enabled: bool) -> Result<bool, String> {
    Ok(false)
}
