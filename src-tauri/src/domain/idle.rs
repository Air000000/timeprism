use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
pub(crate) struct IdlePromptEntry {
    pub(crate) id: i64,
    pub(crate) start_timestamp: i64,
    pub(crate) end_timestamp: i64,
    pub(crate) duration_ms: i64,
    pub(crate) deferred_until_timestamp: Option<i64>,
}

#[derive(Deserialize)]
pub(crate) struct ResolveIdlePromptInput {
    pub(crate) prompt_id: i64,
    pub(crate) decision: String,
    pub(crate) remember_this_session: Option<bool>,
}

#[derive(Serialize)]
pub(crate) struct IdleMemoryState {
    pub(crate) remembered_decision: Option<String>,
}
