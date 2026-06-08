use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub(crate) struct ForegroundCaptureDiagnostic {
    pub(crate) id: i64,
    pub(crate) captured_at_ms: i64,
    pub(crate) observed_process_name: String,
    pub(crate) observed_window_title: String,
    pub(crate) stored: bool,
    pub(crate) block_reason: Option<String>,
    pub(crate) rule_saved: bool,
    pub(crate) rule_mapped_type: String,
}

#[derive(Deserialize)]
pub(crate) struct SettlePetWindowInput {
    pub(crate) mode: String,
}

#[derive(Serialize)]
pub(crate) struct PetWindowSettleResult {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) state: String,
}
