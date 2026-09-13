const WINDOW_SERVICE_SOURCE: &str = include_str!("../src/services/window.rs");
const TAURI_CONFIG: &str = include_str!("../tauri.conf.json");

#[test]
fn pet_window_uses_monitor_adaptive_logical_size_and_zoom() {
    assert!(
        WINDOW_SERVICE_SOURCE.contains("fn pet_window_target_size(scale: f64) -> tauri::Size"),
        "pet window sizing should remain centralized in logical units"
    );
    assert!(
        WINDOW_SERVICE_SOURCE.contains("fn pet_monitor_scale("),
        "pet window sizing should derive an adaptive scale from its monitor"
    );
    assert!(
        WINDOW_SERVICE_SOURCE.contains(".current_monitor()"),
        "pet sizing should use the monitor that contains the pet after a cross-display drag"
    );
    assert!(
        WINDOW_SERVICE_SOURCE.contains("monitor.work_area()")
            && WINDOW_SERVICE_SOURCE.contains("monitor.scale_factor()"),
        "pet sizing should compare effective logical monitor work areas"
    );
    assert!(
        WINDOW_SERVICE_SOURCE.contains(".set_zoom(scale)"),
        "pet content must scale with the window so prompts do not wrap and clip again"
    );
    assert!(
        !WINDOW_SERVICE_SOURCE.contains(
            ".set_size(tauri::Size::Physical(tauri::PhysicalSize {\n            width: 220,\n            height: 262,\n        }))"
        ),
        "pet window must not return to fixed physical-pixel sizing"
    );
}

#[test]
fn configured_pet_size_remains_the_base_design_canvas() {
    let config: serde_json::Value = serde_json::from_str(TAURI_CONFIG).expect("parse tauri config");
    let windows = config["app"]["windows"]
        .as_array()
        .expect("window config array");
    let pet = windows
        .iter()
        .find(|item| item["label"] == "pet")
        .expect("pet window config");

    assert_eq!(pet["width"], 220, "pet base logical width changed unexpectedly");
    assert_eq!(pet["height"], 262, "pet base logical height changed unexpectedly");
}
