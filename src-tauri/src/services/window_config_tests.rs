#[test]
fn configured_product_windows_require_manual_creation() {
    let config: serde_json::Value = serde_json::from_str(include_str!("../../tauri.conf.json"))
        .expect("parse tauri config");
    let windows = config["app"]["windows"]
        .as_array()
        .expect("window config array");

    for label in ["main", "pet", "pet-panel"] {
        let item = windows
            .iter()
            .find(|item| item["label"] == label)
            .unwrap_or_else(|| panic!("missing window config {label}"));
        assert_eq!(
            item["create"], false,
            "{label} must not auto-create before DB init"
        );
    }
}
