use tauri::{AppHandle, Manager, WebviewWindow};

pub(crate) fn create_configured_window(
    app: &AppHandle,
    label: &str,
) -> Result<WebviewWindow, String> {
    if let Some(existing) = app.get_webview_window(label) {
        return Ok(existing);
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|item| item.label == label)
        .ok_or_else(|| format!("window config not found: {label}"))?;

    tauri::WebviewWindowBuilder::from_config(app, config)
        .map_err(|e| format!("failed to build window config {label}: {e}"))?
        .build()
        .map_err(|e| format!("failed to create configured window {label}: {e}"))
}
