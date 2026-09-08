use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct Category {
    pub(crate) id: i64,
    pub(crate) parent_id: Option<i64>,
    pub(crate) name: String,
    pub(crate) root_type: String,
    pub(crate) color_hex: String,
}

#[derive(Deserialize)]
pub(crate) struct CreateCategoryInput {
    pub(crate) parent_id: i64,
    pub(crate) name: String,
    pub(crate) color_hex: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TodaySummary {
    pub(crate) learn_seconds: i64,
    pub(crate) rest_seconds: i64,
    pub(crate) active_session_id: Option<i64>,
}

#[derive(Serialize)]
pub(crate) struct TopApp {
    pub(crate) process_name: String,
    pub(crate) seconds: i64,
}

#[derive(Serialize)]
pub(crate) struct LearnHeatmapCell {
    pub(crate) day: String,
    pub(crate) learn_seconds: i64,
    pub(crate) level: String,
}

#[derive(Serialize)]
pub(crate) struct UsageStackSegment {
    pub(crate) name: String,
    pub(crate) seconds: i64,
}

#[derive(Serialize)]
pub(crate) struct UsageStackDay {
    pub(crate) day: String,
    pub(crate) total_seconds: i64,
    pub(crate) learn_seconds: i64,
    pub(crate) rest_seconds: i64,
    pub(crate) segments: Vec<UsageStackSegment>,
}

#[derive(Serialize)]
pub(crate) struct RecentLogEntry {
    pub(crate) id: i64,
    pub(crate) process_name: String,
    pub(crate) window_title: String,
    pub(crate) start_timestamp: i64,
    pub(crate) duration_ms: i64,
}
