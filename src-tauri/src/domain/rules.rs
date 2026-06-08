use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct AppRuleEntry {
    pub(crate) process_name: String,
    pub(crate) mapped_type: String,
    pub(crate) privacy_level: String,
    pub(crate) updated_at: i64,
}

#[derive(Serialize)]
pub(crate) struct PendingRuleProcess {
    pub(crate) process_name: String,
    pub(crate) last_seen_timestamp: i64,
    pub(crate) last_window_title: String,
    pub(crate) total_seconds: i64,
}

#[derive(Deserialize)]
pub(crate) struct SaveAppRuleInput {
    pub(crate) process_name: String,
    pub(crate) mapped_type: String,
    pub(crate) privacy_level: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct DeviationCheck {
    pub(crate) triggered: bool,
    pub(crate) process_name: String,
    pub(crate) reason: String,
    pub(crate) active_root_type: Option<String>,
    pub(crate) mapped_type: Option<String>,
    pub(crate) suggested_root_category_id: Option<i64>,
}
