use tauri::{AppHandle, Emitter, Manager};

use crate::domain::window::{PetWindowSettleResult, SettlePetWindowInput};

pub(crate) fn set_main_close_behavior(main_window: &tauri::WebviewWindow) {
    let main_clone = main_window.clone();
    main_window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = main_clone.hide();
        }
    });
}

pub(crate) fn reveal_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        let _ = main_window.unminimize();
        let _ = main_window.center();
        let _ = main_window.show();
        let _ = main_window.set_focus();
    }
}

pub(crate) fn ensure_pet_window_position(
    app: &AppHandle,
    force_to_corner: bool,
) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;

    let monitor = app
        .primary_monitor()
        .map_err(|e| format!("failed to get primary monitor: {e}"))?
        .ok_or_else(|| "primary monitor unavailable".to_string())?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to get pet window size: {e}"))?;
    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to get pet window position: {e}"))?;

    let desired_x = monitor_pos.x + monitor_size.width as i32 - pet_size.width as i32 - 24;
    let desired_y = monitor_pos.y + monitor_size.height as i32 - pet_size.height as i32 - 48;

    let offscreen = pet_pos.x < monitor_pos.x - 40
        || pet_pos.y < monitor_pos.y - 40
        || pet_pos.x + pet_size.width as i32 > monitor_pos.x + monitor_size.width as i32 + 40
        || pet_pos.y + pet_size.height as i32 > monitor_pos.y + monitor_size.height as i32 + 40;

    if force_to_corner || offscreen {
        pet_window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: desired_x,
                y: desired_y,
            }))
            .map_err(|e| format!("failed to move pet window: {e}"))?;
    }

    pet_window
        .set_always_on_top(true)
        .map_err(|e| format!("failed to set pet always on top: {e}"))?;

    Ok(())
}

fn pet_monitor_rect(
    app: &AppHandle,
    pet_window: &tauri::WebviewWindow,
) -> Result<(i32, i32, i32, i32), String> {
    let monitor = pet_window
        .current_monitor()
        .map_err(|e| format!("failed to get current pet monitor: {e}"))?
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or_else(|| "pet monitor unavailable".to_string())?;
    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();
    Ok((
        monitor_pos.x,
        monitor_pos.y,
        monitor_size.width as i32,
        monitor_size.height as i32,
    ))
}

fn clamp_pet_window_position(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    rect_x: i32,
    rect_y: i32,
    rect_width: i32,
    rect_height: i32,
) -> (i32, i32) {
    let max_x = rect_x + (rect_width - width as i32).max(0);
    let max_y = rect_y + (rect_height - height as i32).max(0);
    (x.clamp(rect_x, max_x), y.clamp(rect_y, max_y))
}

fn normalize_pet_settle_mode(value: &str) -> Result<&'static str, String> {
    match value.trim().to_lowercase().as_str() {
        "free" => Ok("free"),
        "dock_left" => Ok("dock_left"),
        "dock_right" => Ok("dock_right"),
        other => Err(format!("invalid pet settle mode: {other}")),
    }
}

fn settle_pet_window_internal(
    app: &AppHandle,
    mode: &str,
) -> Result<PetWindowSettleResult, String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    let normalized_mode = normalize_pet_settle_mode(mode)?;
    let (target_width, target_height) = (220_u32, 262_u32);

    pet_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: target_width,
            height: target_height,
        }))
        .map_err(|e| format!("failed to resize pet window during settle: {e}"))?;

    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to read pet window size during settle: {e}"))?;
    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to read pet window position during settle: {e}"))?;
    let (rect_x, rect_y, rect_width, rect_height) = pet_monitor_rect(app, &pet_window)?;

    let desired_x = match normalized_mode {
        "dock_left" => rect_x,
        "dock_right" => rect_x + rect_width - pet_size.width as i32,
        _ => pet_pos.x,
    };
    let desired_y = pet_pos.y;
    let (next_x, next_y) = clamp_pet_window_position(
        desired_x,
        desired_y,
        pet_size.width,
        pet_size.height,
        rect_x,
        rect_y,
        rect_width,
        rect_height,
    );

    pet_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: next_x,
            y: next_y,
        }))
        .map_err(|e| format!("failed to position pet window during settle: {e}"))?;

    let _ = sync_pet_panel_position(app);
    Ok(PetWindowSettleResult {
        x: next_x,
        y: next_y,
        width: pet_size.width,
        height: pet_size.height,
        state: normalized_mode.to_string(),
    })
}

fn ensure_pet_panel_position(app: &AppHandle) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;

    let panel_window = app
        .get_webview_window("pet-panel")
        .ok_or_else(|| "pet panel window not found".to_string())?;

    let monitor = pet_window
        .current_monitor()
        .map_err(|e| format!("failed to get current monitor: {e}"))?
        .ok_or_else(|| "current monitor unavailable".to_string())?;

    let monitor_pos = monitor.position();
    let monitor_size = monitor.size();

    let pet_pos = pet_window
        .outer_position()
        .map_err(|e| format!("failed to get pet window position: {e}"))?;
    let pet_size = pet_window
        .outer_size()
        .map_err(|e| format!("failed to get pet window size: {e}"))?;
    let panel_size = panel_window
        .outer_size()
        .map_err(|e| format!("failed to get pet panel window size: {e}"))?;

    let mut target_x = pet_pos.x + pet_size.width as i32 + 8;
    let mut target_y = pet_pos.y;

    let max_x = monitor_pos.x + monitor_size.width as i32 - panel_size.width as i32 - 8;
    let min_x = monitor_pos.x + 8;
    let max_y = monitor_pos.y + monitor_size.height as i32 - panel_size.height as i32 - 8;
    let min_y = monitor_pos.y + 8;

    if target_x > max_x {
        target_x = pet_pos.x - panel_size.width as i32 - 8;
    }

    target_x = target_x.clamp(min_x, max_x.max(min_x));
    target_y = target_y.clamp(min_y, max_y.max(min_y));

    panel_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: target_x,
            y: target_y,
        }))
        .map_err(|e| format!("failed to move pet panel window: {e}"))?;

    panel_window
        .set_always_on_top(true)
        .map_err(|e| format!("failed to set pet panel always on top: {e}"))?;

    Ok(())
}

pub(crate) fn summon_pet_window(app: &AppHandle) -> Result<(), String> {
    let pet_window = if let Some(existing) = app.get_webview_window("pet") {
        existing
    } else {
        tauri::WebviewWindowBuilder::new(app, "pet", tauri::WebviewUrl::App("pet.html".into()))
            .title("TimePrism Pet")
            .inner_size(220.0, 262.0)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .build()
            .map_err(|e| format!("failed to recreate pet window: {e}"))?
    };

    pet_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: 220,
            height: 262,
        }))
        .map_err(|e| format!("failed to resize pet window: {e}"))?;

    ensure_pet_window_position(app, true)?;

    pet_window
        .show()
        .map_err(|e| format!("failed to show pet window: {e}"))?;
    pet_window
        .set_focus()
        .map_err(|e| format!("failed to focus pet window: {e}"))?;

    let _ = pet_window.emit("pet-summoned", true);

    Ok(())
}

pub(crate) fn sync_pet_window_layout(app: &AppHandle) -> Result<(), String> {
    let _ = settle_pet_window_internal(app, "free")?;
    Ok(())
}

pub(crate) fn settle_pet_window(
    app: &AppHandle,
    input: SettlePetWindowInput,
) -> Result<PetWindowSettleResult, String> {
    settle_pet_window_internal(app, &input.mode)
}

pub(crate) fn show_pet_panel(app: &AppHandle, mode: String) -> Result<(), String> {
    let panel_window = if let Some(existing) = app.get_webview_window("pet-panel") {
        existing
    } else {
        tauri::WebviewWindowBuilder::new(
            app,
            "pet-panel",
            tauri::WebviewUrl::App("pet-panel.html".into()),
        )
        .title("TimePrism Panel")
        .inner_size(186.0, 128.0)
        .decorations(false)
        .transparent(true)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .build()
        .map_err(|e| format!("failed to create pet panel window: {e}"))?
    };

    ensure_pet_panel_position(app)?;
    panel_window
        .show()
        .map_err(|e| format!("failed to show pet panel window: {e}"))?;
    let _ = panel_window.emit("pet-panel-active", true);
    let _ = panel_window.emit("pet-panel-mode", mode);
    Ok(())
}

pub(crate) fn hide_pet_panel(app: &AppHandle) -> Result<(), String> {
    if let Some(panel_window) = app.get_webview_window("pet-panel") {
        let _ = panel_window.emit("pet-panel-active", false);
        panel_window
            .hide()
            .map_err(|e| format!("failed to hide pet panel window: {e}"))?;
    }
    Ok(())
}

pub(crate) fn sync_pet_panel_position(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window("pet-panel").is_none() {
        return Ok(());
    }
    ensure_pet_panel_position(app)
}

pub(crate) fn resize_pet_panel(app: &AppHandle, width: i32, height: i32) -> Result<(), String> {
    let panel_window = app
        .get_webview_window("pet-panel")
        .ok_or_else(|| "pet panel window not found".to_string())?;

    let safe_w = width.clamp(150, 300) as u32;
    let safe_h = height.clamp(96, 260) as u32;
    panel_window
        .set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: safe_w,
            height: safe_h,
        }))
        .map_err(|e| format!("failed to resize pet panel window: {e}"))?;

    let _ = sync_pet_panel_position(app);
    Ok(())
}

pub(crate) fn move_pet_window(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| format!("failed to move pet window from command: {e}"))?;

    let _ = sync_pet_panel_position(app);
    Ok(())
}

pub(crate) fn close_pet_window(app: &AppHandle) -> Result<(), String> {
    let _ = hide_pet_panel(app);
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .close()
        .map_err(|e| format!("failed to close pet window: {e}"))?;
    Ok(())
}

pub(crate) fn hide_pet_window(app: &AppHandle) -> Result<(), String> {
    let _ = hide_pet_panel(app);
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .hide()
        .map_err(|e| format!("failed to hide pet window: {e}"))?;
    Ok(())
}

pub(crate) fn begin_pet_drag(app: &AppHandle) -> Result<(), String> {
    let pet_window = app
        .get_webview_window("pet")
        .ok_or_else(|| "pet window not found".to_string())?;
    pet_window
        .start_dragging()
        .map_err(|e| format!("failed to start pet dragging: {e}"))?;
    Ok(())
}

pub(crate) fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let window = if let Some(existing) = app.get_webview_window("main") {
        existing
    } else {
        let created =
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                .title("TimePrism")
                .inner_size(888.0, 900.0)
                .build()
                .map_err(|e| format!("failed to recreate main window: {e}"))?;
        set_main_close_behavior(&created);
        created
    };

    let _ = window.unminimize();
    window
        .show()
        .map_err(|e| format!("failed to show main window: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("failed to focus main window: {e}"))?;

    Ok(())
}

pub(crate) fn show_main_window_section(app: &AppHandle, section: String) -> Result<(), String> {
    let window = if let Some(existing) = app.get_webview_window("main") {
        existing
    } else {
        let created =
            tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
                .title("TimePrism")
                .inner_size(888.0, 900.0)
                .build()
                .map_err(|e| format!("failed to recreate main window: {e}"))?;
        set_main_close_behavior(&created);
        created
    };

    let _ = window.unminimize();
    window
        .show()
        .map_err(|e| format!("failed to show main window: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("failed to focus main window: {e}"))?;
    window
        .emit("navigate-insights-section", section)
        .map_err(|e| format!("failed to emit navigate event: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{clamp_pet_window_position, normalize_pet_settle_mode};

    #[test]
    fn normalizes_pet_settle_modes() {
        assert_eq!(normalize_pet_settle_mode(" free "), Ok("free"));
        assert_eq!(normalize_pet_settle_mode("DOCK_LEFT"), Ok("dock_left"));
        assert_eq!(normalize_pet_settle_mode("dock_right"), Ok("dock_right"));
        assert!(normalize_pet_settle_mode("center").is_err());
    }

    #[test]
    fn clamps_pet_position_to_monitor_rect() {
        assert_eq!(
            clamp_pet_window_position(-50, 500, 100, 100, 0, 0, 800, 600),
            (0, 500)
        );
        assert_eq!(
            clamp_pet_window_position(760, 590, 100, 100, 0, 0, 800, 600),
            (700, 500)
        );
    }
}
