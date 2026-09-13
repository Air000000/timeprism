const WINDOW_SERVICE_SOURCE: &str = include_str!("../src/services/window.rs");
const TAURI_CONFIG: &str = include_str!("../tauri.conf.json");

#[test]
fn pet_window_uses_logical_size_instead_of_physical_pixels() {
    assert!(
        WINDOW_SERVICE_SOURCE.contains("fn pet_window_target_size() -> tauri::Size"),
        "pet window sizing should be centralized in a logical-size helper"
    );
    assert!(
        WINDOW_SERVICE_SOURCE.matches("pet_window_target_size()").count() >= 3,
        "both settle and summon paths should use the same logical-size helper"
    );
    assert!(
        !WINDOW_SERVICE_SOURCE.contains(
            ".set_size(tauri::Size::Physical(tauri::PhysicalSize {\n            width: target_width,\n            height: target_height,\n        }))"
        ),
        "settling the pet must not force a DPI-dependent physical-pixel size"
    );
    assert!(
        !WINDOW_SERVICE_SOURCE.contains(
            ".set_size(tauri::Size::Physical(tauri::PhysicalSize {\n            width: 220,\n            height: 262,\n        }))"
        ),
        "summoning the pet must not force a DPI-dependent physical-pixel size"
    );
}

#[test]
fn configured_pet_height_matches_runtime_logical_height() {
    let config: serde_json::Value = serde_json::from_str(TAURI_CONFIG).expect("parse tauri config");
    let windows = config["app"]["windows"]
        .as_array()
        .expect("window config array");
    let pet = windows
        .iter()
        .find(|item| item["label"] == "pet")
        .expect("pet window config");

    assert_eq!(pet["width"], 220, "pet logical width changed unexpectedly");
    assert_eq!(pet["height"], 262, "pet config must match runtime logical height");
}
