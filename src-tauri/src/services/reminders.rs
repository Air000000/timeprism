use chrono::{Datelike, Local, TimeZone};
use rusqlite::{params, Connection};

use crate::domain::reminders::{
    ReminderEntry, SaveReminderInput, SetReminderDoneInput, SetReminderOrderInput,
};
use crate::{
    business_day_key_from_start, business_day_start_from_local, business_day_window_from_local,
};

const NO_DUE_TIMESTAMP: i64 = i64::MAX / 4;

fn normalize_repeat_rule(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "NONE" | "DAILY" | "WEEKLY") {
        Ok(normalized)
    } else {
        Err("repeat_rule must be NONE, DAILY, or WEEKLY".to_string())
    }
}

fn normalize_weekly_days(days: Option<Vec<i64>>) -> Option<Vec<i64>> {
    let mut safe = Vec::new();
    for day in days.unwrap_or_default() {
        if (0..=6).contains(&day) && !safe.contains(&day) {
            safe.push(day);
        }
    }
    safe.sort_unstable();
    if safe.is_empty() {
        None
    } else {
        Some(safe)
    }
}

fn weekly_days_to_db(days: &[i64]) -> String {
    days.iter().map(|day| day.to_string()).collect::<Vec<_>>().join(",")
}

fn parse_weekly_days_db(value: Option<String>) -> Option<Vec<i64>> {
    let parsed = value?
        .split(',')
        .filter_map(|chunk| chunk.trim().parse::<i64>().ok())
        .filter(|day| (0..=6).contains(day))
        .collect::<Vec<_>>();
    normalize_weekly_days(Some(parsed))
}

fn next_weekly_due_timestamp(
    today_start_ts: i64,
    today_weekday: i64,
    weekly_days: &[i64],
    minutes: Option<i64>,
    done_today: bool,
) -> i64 {
    let Some(clamped_minutes) = minutes.map(|v| v.clamp(0, 1439)) else {
        return NO_DUE_TIMESTAMP;
    };
    for offset in 0..14 {
        let candidate_weekday = (today_weekday + offset) % 7;
        if !weekly_days.contains(&candidate_weekday) {
            continue;
        }
        if offset == 0 && done_today {
            continue;
        }
        return today_start_ts + offset * 86_400 + clamped_minutes * 60;
    }
    NO_DUE_TIMESTAMP
}

fn collect_reminders(
    conn: &Connection,
    include_completed: bool,
    cap: usize,
) -> Result<Vec<ReminderEntry>, String> {
    let (today_start_ts, _) = business_day_window_from_local(Local::now())?;
    let today_key = business_day_key_from_start(today_start_ts)?;
    let tomorrow_start_ts = today_start_ts + 86_400;

    let query_limit = (cap as i64 * 6).clamp(30, 600);
    let mut stmt = conn
        .prepare(
            "SELECT id,
                    content,
                    repeat_rule,
                    sort_order,
                    remind_at,
                    daily_time_minutes,
                    weekly_days,
                    is_completed,
                    completed_day_key,
                    completed_at,
                    snooze_until,
                    created_at,
                    updated_at
             FROM reminders
             ORDER BY sort_order ASC, updated_at DESC
             LIMIT ?1",
        )
        .map_err(|e| format!("failed to prepare reminders query: {e}"))?;

    let rows = stmt
        .query_map([query_limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<i64>>(4)?,
                row.get::<_, Option<i64>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i64>>(9)?,
                row.get::<_, Option<i64>>(10)?,
                row.get::<_, i64>(11)?,
                row.get::<_, i64>(12)?,
            ))
        })
        .map_err(|e| format!("failed to query reminders rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        let (
            id,
            content,
            repeat_rule_raw,
            sort_order,
            remind_at,
            daily_time_minutes,
            weekly_days_raw,
            is_completed,
            completed_day_key,
            completed_at,
            snooze_until,
            created_at,
            updated_at,
        ) = row.map_err(|e| format!("failed to parse reminder row: {e}"))?;

        let repeat_rule = normalize_repeat_rule(&repeat_rule_raw).unwrap_or_else(|_| "NONE".to_string());
        let snooze_until_safe = snooze_until.filter(|v| *v > 0);
        let weekly_days = parse_weekly_days_db(weekly_days_raw);
        let today_weekday = Local
            .timestamp_opt(today_start_ts, 0)
            .single()
            .map(|dt| dt.weekday().num_days_from_sunday() as i64)
            .unwrap_or(0);

        if repeat_rule == "DAILY" {
            let minutes = daily_time_minutes.map(|v| v.clamp(0, 1439));
            let base_due = minutes.map(|v| today_start_ts + v * 60);
            let done_today = completed_day_key
                .as_deref()
                .map(|day| day == today_key)
                .unwrap_or(false);
            let mut next_due = if done_today {
                minutes.map(|v| tomorrow_start_ts + v * 60).unwrap_or(NO_DUE_TIMESTAMP)
            } else {
                base_due.unwrap_or(NO_DUE_TIMESTAMP)
            };
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            if !include_completed && done_today {
                continue;
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at: None,
                daily_time_minutes: minutes,
                weekly_days: None,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done: done_today,
                completed_day_key,
                completed_at,
                created_at,
                updated_at,
            });
        } else if repeat_rule == "WEEKLY" {
            let minutes = daily_time_minutes.map(|v| v.clamp(0, 1439));
            let days = weekly_days.clone().unwrap_or_default();
            let done_today = completed_day_key
                .as_deref()
                .map(|day| day == today_key)
                .unwrap_or(false);
            let mut next_due = next_weekly_due_timestamp(today_start_ts, today_weekday, &days, minutes, done_today);
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            if !include_completed && done_today {
                continue;
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at: None,
                daily_time_minutes: minutes,
                weekly_days,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done: done_today,
                completed_day_key,
                completed_at,
                created_at,
                updated_at,
            });
        } else {
            let done = is_completed > 0;
            if !include_completed && done {
                continue;
            }

            let mut next_due = remind_at.filter(|v| *v > 0).unwrap_or(NO_DUE_TIMESTAMP);
            if let Some(snooze) = snooze_until_safe {
                if snooze > next_due {
                    next_due = snooze;
                }
            }

            result.push(ReminderEntry {
                id,
                content,
                repeat_rule,
                sort_order,
                remind_at,
                daily_time_minutes: None,
                weekly_days: None,
                snooze_until: snooze_until_safe,
                next_due_timestamp: next_due,
                done,
                completed_day_key: None,
                completed_at,
                created_at,
                updated_at,
            });
        }
    }

    result.sort_by(|a, b| {
        a.done
            .cmp(&b.done)
            .then_with(|| a.sort_order.cmp(&b.sort_order))
            .then_with(|| a.next_due_timestamp.cmp(&b.next_due_timestamp))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
    });

    if result.len() > cap {
        result.truncate(cap);
    }

    Ok(result)
}

pub(crate) fn list_reminder_entries(
    conn: &Connection,
    limit: Option<i64>,
    include_completed: Option<bool>,
) -> Result<Vec<ReminderEntry>, String> {
    let cap = limit.unwrap_or(50).clamp(1, 200) as usize;
    collect_reminders(conn, include_completed.unwrap_or(false), cap)
}

pub(crate) fn list_due_reminder_entries(
    conn: &Connection,
    limit: Option<i64>,
) -> Result<Vec<ReminderEntry>, String> {
    let cap = limit.unwrap_or(6).clamp(1, 30) as usize;
    let now_ts = Local::now().timestamp();
    let mut due = collect_reminders(conn, false, 200)?
        .into_iter()
        .filter(|item| !item.done && item.next_due_timestamp <= now_ts && item.next_due_timestamp < NO_DUE_TIMESTAMP)
        .collect::<Vec<_>>();
    if due.len() > cap {
        due.truncate(cap);
    }
    Ok(due)
}

pub(crate) fn save_reminder_entry(conn: &Connection, input: SaveReminderInput) -> Result<i64, String> {
    let now_ts = Local::now().timestamp();
    let content = input.content.trim().to_string();
    if content.is_empty() {
        return Err("content cannot be empty".to_string());
    }

    let repeat_rule = normalize_repeat_rule(&input.repeat_rule)?;
    let next_sort_order = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM reminders",
            [],
            |row| row.get::<_, i64>(0),
        )
        .unwrap_or(0);

    if repeat_rule == "DAILY" {
        let minutes = input.daily_time_minutes.map(|v| v.clamp(0, 1439));
        if let Some(id) = input.id {
            let changed = conn
                .execute(
                    "UPDATE reminders
                     SET content = ?1,
                         repeat_rule = 'DAILY',
                         remind_at = NULL,
                         daily_time_minutes = ?2,
                         weekly_days = NULL,
                         is_completed = 0,
                         completed_day_key = NULL,
                         completed_at = NULL,
                         snooze_until = NULL,
                         updated_at = ?3
                     WHERE id = ?4",
                    params![content, minutes, now_ts, id],
                )
                .map_err(|e| format!("failed to update daily reminder: {e}"))?;
            if changed == 0 {
                return Err("reminder not found".to_string());
            }
            return Ok(id);
        }

        conn.execute(
            "INSERT INTO reminders
             (content, repeat_rule, sort_order, remind_at, daily_time_minutes, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
             VALUES (?1, 'DAILY', ?2, NULL, ?3, 0, NULL, NULL, NULL, ?4, ?4)",
            params![content, next_sort_order, minutes, now_ts],
        )
        .map_err(|e| format!("failed to create daily reminder: {e}"))?;
        return Ok(conn.last_insert_rowid());
    }

    if repeat_rule == "WEEKLY" {
        let minutes = input.daily_time_minutes.map(|v| v.clamp(0, 1439));
        let weekly_days = normalize_weekly_days(input.weekly_days);
        let weekly_days_db = weekly_days.as_ref().map(|days| weekly_days_to_db(days));
        if let Some(id) = input.id {
            let changed = conn
                .execute(
                    "UPDATE reminders
                     SET content = ?1,
                         repeat_rule = 'WEEKLY',
                         remind_at = NULL,
                         daily_time_minutes = ?2,
                         weekly_days = ?3,
                         is_completed = 0,
                         completed_day_key = NULL,
                         completed_at = NULL,
                         snooze_until = NULL,
                         updated_at = ?4
                     WHERE id = ?5",
                    params![content, minutes, weekly_days_db, now_ts, id],
                )
                .map_err(|e| format!("failed to update weekly reminder: {e}"))?;
            if changed == 0 {
                return Err("reminder not found".to_string());
            }
            return Ok(id);
        }

        conn.execute(
            "INSERT INTO reminders
             (content, repeat_rule, sort_order, remind_at, daily_time_minutes, weekly_days, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
             VALUES (?1, 'WEEKLY', ?2, NULL, ?3, ?4, 0, NULL, NULL, NULL, ?5, ?5)",
            params![content, next_sort_order, minutes, weekly_days_db, now_ts],
        )
        .map_err(|e| format!("failed to create weekly reminder: {e}"))?;
        return Ok(conn.last_insert_rowid());
    }

    let remind_at = input.remind_at.map(|v| v.max(0));
    if let Some(id) = input.id {
        let changed = conn
            .execute(
                "UPDATE reminders
                 SET content = ?1,
                     repeat_rule = 'NONE',
                     remind_at = ?2,
                     daily_time_minutes = NULL,
                     weekly_days = NULL,
                     snooze_until = NULL,
                     updated_at = ?3
                  WHERE id = ?4",
                params![content, remind_at, now_ts, id],
            )
            .map_err(|e| format!("failed to update reminder: {e}"))?;
        if changed == 0 {
            return Err("reminder not found".to_string());
        }
        return Ok(id);
    }

    conn.execute(
        "INSERT INTO reminders
         (content, repeat_rule, sort_order, remind_at, daily_time_minutes, is_completed, completed_day_key, completed_at, snooze_until, created_at, updated_at)
         VALUES (?1, 'NONE', ?2, ?3, NULL, 0, NULL, NULL, NULL, ?4, ?4)",
        params![content, next_sort_order, remind_at, now_ts],
    )
    .map_err(|e| format!("failed to create reminder: {e}"))?;

    Ok(conn.last_insert_rowid())
}

pub(crate) fn delete_reminder_entry(conn: &Connection, id: i64) -> Result<bool, String> {
    let changed = conn
        .execute("DELETE FROM reminders WHERE id = ?1", [id])
        .map_err(|e| format!("failed to delete reminder: {e}"))?;
    Ok(changed > 0)
}

pub(crate) fn set_reminder_done_entry(
    conn: &Connection,
    input: SetReminderDoneInput,
) -> Result<bool, String> {
    let repeat_rule_raw = conn
        .query_row(
            "SELECT repeat_rule FROM reminders WHERE id = ?1",
            [input.id],
            |row| row.get::<_, String>(0),
        )
        .map_err(|_| "reminder not found".to_string())?;
    let repeat_rule = normalize_repeat_rule(&repeat_rule_raw)?;
    let now_ts = Local::now().timestamp();

    if repeat_rule == "DAILY" || repeat_rule == "WEEKLY" {
        let today_start_ts = business_day_start_from_local(Local::now())?;
        let today_key = business_day_key_from_start(today_start_ts)?;
        let changed = if input.done {
            conn.execute(
                "UPDATE reminders
                 SET completed_day_key = ?1,
                     completed_at = ?2,
                     snooze_until = NULL,
                     updated_at = ?2
                 WHERE id = ?3",
                params![today_key, now_ts, input.id],
            )
        } else {
            conn.execute(
                "UPDATE reminders
                 SET completed_day_key = NULL,
                     completed_at = NULL,
                     updated_at = ?1
                 WHERE id = ?2",
                params![now_ts, input.id],
            )
        }
        .map_err(|e| format!("failed to toggle recurring reminder completion: {e}"))?;
        return Ok(changed > 0);
    }

    let changed = conn
        .execute(
            "UPDATE reminders
             SET is_completed = ?1,
                 completed_at = CASE WHEN ?1 = 1 THEN ?2 ELSE NULL END,
                 snooze_until = CASE WHEN ?1 = 1 THEN NULL ELSE snooze_until END,
                 updated_at = ?2
             WHERE id = ?3",
            params![if input.done { 1 } else { 0 }, now_ts, input.id],
        )
        .map_err(|e| format!("failed to toggle reminder completion: {e}"))?;
    Ok(changed > 0)
}

pub(crate) fn set_reminder_order_entries(
    conn: &Connection,
    input: SetReminderOrderInput,
) -> Result<bool, String> {
    if input.ordered_ids.is_empty() {
        return Ok(true);
    }

    let now_ts = Local::now().timestamp();
    for (index, id) in input.ordered_ids.iter().enumerate() {
        conn.execute(
            "UPDATE reminders SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
            params![index as i64, now_ts, id],
        )
        .map_err(|e| format!("failed to update reminder sort order: {e}"))?;
    }
    Ok(true)
}

pub(crate) fn snooze_reminder_entry(
    conn: &Connection,
    id: i64,
    snooze_seconds: Option<i64>,
) -> Result<bool, String> {
    let now_ts = Local::now().timestamp();
    let seconds = snooze_seconds.unwrap_or(600).clamp(60, 86_400);
    let snooze_until = now_ts + seconds;
    let changed = conn
        .execute(
            "UPDATE reminders
             SET snooze_until = ?1,
                 updated_at = ?2
             WHERE id = ?3",
            params![snooze_until, now_ts, id],
        )
        .map_err(|e| format!("failed to snooze reminder: {e}"))?;
    Ok(changed > 0)
}

#[cfg(test)]
mod tests {
    use super::{next_weekly_due_timestamp, normalize_weekly_days, weekly_days_to_db, NO_DUE_TIMESTAMP};

    #[test]
    fn weekly_days_are_deduped_sorted_and_clamped() {
        assert_eq!(normalize_weekly_days(Some(vec![6, 2, 8, 2, -1, 0])), Some(vec![0, 2, 6]));
        assert_eq!(normalize_weekly_days(Some(vec![8, -1])), None);
    }

    #[test]
    fn weekly_days_are_serialized_for_db() {
        assert_eq!(weekly_days_to_db(&[0, 2, 6]), "0,2,6");
    }

    #[test]
    fn weekly_due_skips_completed_today() {
        let today_start_ts = 1_000_000;
        let due = next_weekly_due_timestamp(today_start_ts, 2, &[2], Some(60), true);
        assert_eq!(due, today_start_ts + 7 * 86_400 + 60 * 60);
    }

    #[test]
    fn weekly_due_without_minutes_is_never_due() {
        assert_eq!(next_weekly_due_timestamp(1_000_000, 2, &[2], None, false), NO_DUE_TIMESTAMP);
    }
}
