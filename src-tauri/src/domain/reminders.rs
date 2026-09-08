use serde::{Deserialize, Serialize};

#[derive(Serialize, Clone)]
pub(crate) struct ReminderEntry {
    pub(crate) id: i64,
    pub(crate) content: String,
    pub(crate) repeat_rule: String,
    pub(crate) sort_order: i64,
    pub(crate) remind_at: Option<i64>,
    pub(crate) daily_time_minutes: Option<i64>,
    pub(crate) weekly_days: Option<Vec<i64>>,
    pub(crate) snooze_until: Option<i64>,
    pub(crate) next_due_timestamp: i64,
    pub(crate) done: bool,
    pub(crate) completed_day_key: Option<String>,
    pub(crate) completed_at: Option<i64>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Deserialize)]
pub(crate) struct SaveReminderInput {
    pub(crate) id: Option<i64>,
    pub(crate) content: String,
    pub(crate) repeat_rule: String,
    pub(crate) remind_at: Option<i64>,
    pub(crate) daily_time_minutes: Option<i64>,
    pub(crate) weekly_days: Option<Vec<i64>>,
}

#[derive(Deserialize)]
pub(crate) struct SetReminderDoneInput {
    pub(crate) id: i64,
    pub(crate) done: bool,
}

#[derive(Deserialize)]
pub(crate) struct SetReminderOrderInput {
    pub(crate) ordered_ids: Vec<i64>,
}
