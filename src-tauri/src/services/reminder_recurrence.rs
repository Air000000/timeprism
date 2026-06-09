pub(crate) const NO_DUE_TIMESTAMP: i64 = i64::MAX / 4;

pub(crate) fn normalize_repeat_rule(value: &str) -> Result<String, String> {
    let normalized = value.trim().to_uppercase();
    if matches!(normalized.as_str(), "NONE" | "DAILY" | "WEEKLY") {
        Ok(normalized)
    } else {
        Err("repeat_rule must be NONE, DAILY, or WEEKLY".to_string())
    }
}

pub(crate) fn normalize_weekly_days(days: Option<Vec<i64>>) -> Option<Vec<i64>> {
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

pub(crate) fn weekly_days_to_db(days: &[i64]) -> String {
    days.iter()
        .map(|day| day.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn parse_weekly_days_db(value: Option<String>) -> Option<Vec<i64>> {
    let parsed = value?
        .split(',')
        .filter_map(|chunk| chunk.trim().parse::<i64>().ok())
        .filter(|day| (0..=6).contains(day))
        .collect::<Vec<_>>();
    normalize_weekly_days(Some(parsed))
}

pub(crate) fn next_weekly_due_timestamp(
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

#[cfg(test)]
mod tests {
    use super::{
        next_weekly_due_timestamp, normalize_weekly_days, weekly_days_to_db, NO_DUE_TIMESTAMP,
    };

    #[test]
    fn weekly_days_are_deduped_sorted_and_clamped() {
        assert_eq!(
            normalize_weekly_days(Some(vec![6, 2, 8, 2, -1, 0])),
            Some(vec![0, 2, 6])
        );
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
        assert_eq!(
            next_weekly_due_timestamp(1_000_000, 2, &[2], None, false),
            NO_DUE_TIMESTAMP
        );
    }
}
