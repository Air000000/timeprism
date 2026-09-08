use chrono::{Duration, Local, TimeZone};

const BUSINESS_DAY_START_HOUR: i64 = 4;

pub(crate) fn business_day_start_from_local(now: chrono::DateTime<Local>) -> Result<i64, String> {
    let shifted = now - Duration::hours(BUSINESS_DAY_START_HOUR);
    let start_naive = shifted
        .date_naive()
        .and_hms_opt(BUSINESS_DAY_START_HOUR as u32, 0, 0)
        .ok_or_else(|| "failed to build business day start".to_string())?;
    let start_local = Local
        .from_local_datetime(&start_naive)
        .single()
        .ok_or_else(|| "failed to convert business day start to local timestamp".to_string())?;
    Ok(start_local.timestamp())
}

pub(crate) fn business_day_window_from_local(
    now: chrono::DateTime<Local>,
) -> Result<(i64, i64), String> {
    let start = business_day_start_from_local(now)?;
    Ok((start, start + 86_400))
}

pub(crate) fn business_day_start_for_timestamp(ts: i64) -> Result<i64, String> {
    let local_dt = Local
        .timestamp_opt(ts, 0)
        .single()
        .ok_or_else(|| "failed to convert timestamp to local datetime".to_string())?;
    business_day_start_from_local(local_dt)
}

pub(crate) fn business_day_key_from_start(start_ts: i64) -> Result<String, String> {
    let local_dt = Local
        .timestamp_opt(start_ts, 0)
        .single()
        .ok_or_else(|| "failed to convert business day start to local datetime".to_string())?;
    Ok(local_dt.format("%Y-%m-%d").to_string())
}

pub(crate) fn overlap_seconds(
    seg_start: i64,
    seg_end: i64,
    window_start: i64,
    window_end: i64,
) -> i64 {
    let start = seg_start.max(window_start);
    let end = seg_end.min(window_end);
    (end - start).max(0)
}

pub(crate) fn merge_intervals_total(mut intervals: Vec<(i64, i64)>) -> i64 {
    if intervals.is_empty() {
        return 0;
    }

    intervals.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut total = 0_i64;
    let mut current = intervals[0];
    for (start, end) in intervals.into_iter().skip(1) {
        if start <= current.1 {
            current.1 = current.1.max(end);
        } else {
            total += (current.1 - current.0).max(0);
            current = (start, end);
        }
    }

    total + (current.1 - current.0).max(0)
}

#[cfg(test)]
mod tests {
    use super::{merge_intervals_total, overlap_seconds};

    #[test]
    fn overlap_clamps_to_window() {
        assert_eq!(overlap_seconds(10, 20, 15, 30), 5);
        assert_eq!(overlap_seconds(10, 20, 21, 30), 0);
    }

    #[test]
    fn merge_intervals_handles_overlap_and_gaps() {
        assert_eq!(merge_intervals_total(vec![(0, 10), (5, 15), (20, 25)]), 20);
    }
}
