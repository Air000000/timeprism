use std::collections::{BTreeMap, HashMap};

use chrono::Local;
use rusqlite::{params, Connection};

use super::analytics_params::{
    heatmap_goal_seconds, is_usage_stack_root_filter, learn_heatmap_goal_seconds,
    learn_heatmap_span_days, normalize_usage_root_filter, recent_log_limit,
    top_apps_all_time_limit, top_apps_today_limit, usage_stack_span_days,
};
use super::time::{
    business_day_key_from_start, business_day_start_for_timestamp, business_day_window_from_local,
    merge_intervals_total, overlap_seconds,
};
use crate::db::migrations::ensure_heatmap_snapshot_table;
use crate::domain::analytics::{
    LearnHeatmapCell, RecentLogEntry, TodaySummary, TopApp, UsageStackDay, UsageStackSegment,
};

const HEATMAP_LOCK_GOAL_SECONDS: i64 = 7200;

fn parse_i64_config(conn: &Connection, key: &str, default_value: i64) -> i64 {
    conn.query_row(
        "SELECT value FROM app_config WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .and_then(|v| v.parse::<i64>().ok())
    .unwrap_or(default_value)
}

fn compute_learn_seconds_for_window(
    conn: &Connection,
    start_ts: i64,
    end_ts: i64,
) -> Result<i64, String> {
    let mut log_stmt = conn
        .prepare(
            "SELECT l.start_timestamp, l.duration_ms
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = 'LEARN'
               AND l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare learn-seconds log query: {e}"))?;

    let log_rows = log_stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?.max(0)))
        })
        .map_err(|e| format!("failed to query learn-seconds log rows: {e}"))?;

    let mut log_intervals = Vec::new();
    for row in log_rows {
        let (seg_start, duration_ms) =
            row.map_err(|e| format!("failed to parse learn-seconds log row: {e}"))?;
        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end > clip_start {
            log_intervals.push((clip_start, clip_end));
        }
    }

    let from_logs = merge_intervals_total(log_intervals).max(0);

    if from_logs > 0 {
        return Ok(from_logs);
    }

    let mut session_stmt = conn
        .prepare(
            "SELECT ts.start_time, COALESCE(ts.end_time, ?2)
             FROM task_sessions ts
             JOIN categories c ON c.id = ts.category_id
             WHERE c.root_type = 'LEARN'
               AND ts.start_time < ?2
               AND COALESCE(ts.end_time, ?2) > ?1",
        )
        .map_err(|e| format!("failed to prepare learn-seconds session query: {e}"))?;

    let session_rows = session_stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("failed to query learn-seconds session rows: {e}"))?;

    let mut session_intervals = Vec::new();
    for row in session_rows {
        let (seg_start, seg_end) =
            row.map_err(|e| format!("failed to parse learn-seconds session row: {e}"))?;
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end > clip_start {
            session_intervals.push((clip_start, clip_end));
        }
    }

    let from_sessions = merge_intervals_total(session_intervals).max(0);

    Ok(from_sessions)
}

fn seal_historical_heatmap_snapshot(
    conn: &Connection,
    day_start_ts: i64,
    goal_seconds: i64,
) -> Result<(), String> {
    let day_key = business_day_key_from_start(day_start_ts)?;
    let exists = conn
        .query_row(
            "SELECT 1 FROM daily_heatmap_snapshot WHERE day_key = ?1 LIMIT 1",
            [day_key.clone()],
            |_row| Ok(true),
        )
        .unwrap_or(false);
    if exists {
        return Ok(());
    }

    let day_end_ts = day_start_ts + 86_400;
    let learn_seconds = compute_learn_seconds_for_window(conn, day_start_ts, day_end_ts)?.max(0);
    let level = if learn_seconds <= 0 {
        "GRAY"
    } else if learn_seconds < goal_seconds {
        "YELLOW"
    } else {
        "GREEN"
    };

    conn.execute(
        "INSERT OR IGNORE INTO daily_heatmap_snapshot (day_key, learn_seconds, goal_seconds, level, sealed_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![day_key, learn_seconds, goal_seconds, level, Local::now().timestamp()],
    )
    .map_err(|e| format!("failed to insert heatmap snapshot: {e}"))?;

    Ok(())
}

pub(crate) fn list_recent_log_entries(
    conn: &Connection,
    limit: Option<i64>,
) -> Result<Vec<RecentLogEntry>, String> {
    let cap = recent_log_limit(limit);
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, process_name, window_title, start_timestamp, duration_ms
             FROM app_usage_logs
             WHERE start_timestamp >= ?1 AND start_timestamp < ?2
             ORDER BY id DESC
             LIMIT ?3",
        )
        .map_err(|e| format!("failed to prepare recent logs query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts, cap], |row| {
            Ok(RecentLogEntry {
                id: row.get(0)?,
                process_name: row.get(1)?,
                window_title: row.get(2)?,
                start_timestamp: row.get(3)?,
                duration_ms: row.get(4)?,
            })
        })
        .map_err(|e| format!("failed to query recent logs rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse recent log row: {e}"))?);
    }

    Ok(result)
}

pub(crate) fn today_summary(conn: &Connection) -> Result<TodaySummary, String> {
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;
    let mut learn_seconds = 0_i64;
    let mut rest_seconds = 0_i64;

    let mut stmt = conn
        .prepare(
            "SELECT COALESCE(r.mapped_type, 'IGNORE') AS mapped_type,
                    SUM(
                        MAX(
                            0,
                            MIN(l.start_timestamp + (l.duration_ms / 1000), ?2) - MAX(l.start_timestamp, ?1)
                        )
                    ) AS duration_seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1
             GROUP BY mapped_type",
        )
        .map_err(|e| format!("failed to prepare summary query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            let root_type: String = row.get(0)?;
            let duration_seconds: i64 = row.get(1)?;
            Ok((root_type, duration_seconds.max(0)))
        })
        .map_err(|e| format!("failed to query summary rows: {e}"))?;

    for row in rows {
        let (root_type, duration_seconds) =
            row.map_err(|e| format!("failed to parse summary row: {e}"))?;
        if root_type == "LEARN" {
            learn_seconds = duration_seconds;
        } else if root_type == "REST" {
            rest_seconds = duration_seconds;
        }
    }

    Ok(TodaySummary {
        learn_seconds,
        rest_seconds,
        active_session_id: None,
    })
}

pub(crate) fn list_top_apps_today_entries(
    conn: &Connection,
    limit: Option<i64>,
) -> Result<Vec<TopApp>, String> {
    let (start_ts, end_ts) = business_day_window_from_local(Local::now())?;

    let cap = top_apps_today_limit(limit);

    let mut stmt = conn
        .prepare(
            "SELECT l.process_name,
                    SUM(
                        MAX(
                            0,
                            MIN(l.start_timestamp + (l.duration_ms / 1000), ?2) - MAX(l.start_timestamp, ?1)
                        )
                    ) AS seconds
             FROM app_usage_logs l
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1
             GROUP BY process_name
             ORDER BY seconds DESC
             LIMIT ?3",
        )
        .map_err(|e| format!("failed to prepare top apps query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts, cap], |row| {
            Ok(TopApp {
                process_name: row.get(0)?,
                seconds: row.get::<_, i64>(1)?.max(0),
            })
        })
        .map_err(|e| format!("failed to query top apps rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse top app row: {e}"))?);
    }

    Ok(result)
}

pub(crate) fn list_top_apps_all_time_entries(
    conn: &Connection,
    limit: Option<i64>,
    root_filter: Option<String>,
    include_ignore: Option<bool>,
) -> Result<Vec<TopApp>, String> {
    let cap = top_apps_all_time_limit(limit);
    let keep_ignore = include_ignore.unwrap_or(true);
    let filter = normalize_usage_root_filter(root_filter);

    let (sql, bind_filter): (&str, Option<String>) = match filter.as_str() {
        "LEARN" => (
            "SELECT l.process_name,
                    SUM(MAX(0, l.duration_ms / 1000)) AS seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = ?1
             GROUP BY l.process_name
             ORDER BY seconds DESC
             LIMIT ?2",
            Some("LEARN".to_string()),
        ),
        "REST" => (
            "SELECT l.process_name,
                    SUM(MAX(0, l.duration_ms / 1000)) AS seconds
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE COALESCE(r.mapped_type, 'IGNORE') = ?1
             GROUP BY l.process_name
             ORDER BY seconds DESC
             LIMIT ?2",
            Some("REST".to_string()),
        ),
        _ => {
            if keep_ignore {
                (
                    "SELECT l.process_name,
                            SUM(MAX(0, l.duration_ms / 1000)) AS seconds
                     FROM app_usage_logs l
                     GROUP BY l.process_name
                     ORDER BY seconds DESC
                     LIMIT ?1",
                    None,
                )
            } else {
                (
                    "SELECT l.process_name,
                            SUM(MAX(0, l.duration_ms / 1000)) AS seconds
                     FROM app_usage_logs l
                     LEFT JOIN app_rules r ON r.process_name = l.process_name
                     WHERE COALESCE(r.mapped_type, 'IGNORE') != 'IGNORE'
                     GROUP BY l.process_name
                     ORDER BY seconds DESC
                     LIMIT ?1",
                    None,
                )
            }
        }
    };

    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| format!("failed to prepare all-time top apps query: {e}"))?;

    if let Some(mapped_type) = bind_filter {
        let rows = stmt
            .query_map(params![mapped_type, cap], |row| {
                Ok(TopApp {
                    process_name: row.get(0)?,
                    seconds: row.get::<_, i64>(1)?.max(0),
                })
            })
            .map_err(|e| format!("failed to query filtered all-time top apps rows: {e}"))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(
                row.map_err(|e| format!("failed to parse filtered all-time top app row: {e}"))?,
            );
        }
        return Ok(result);
    }

    let rows = stmt
        .query_map([cap], |row| {
            Ok(TopApp {
                process_name: row.get(0)?,
                seconds: row.get::<_, i64>(1)?.max(0),
            })
        })
        .map_err(|e| format!("failed to query all-time top apps rows: {e}"))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| format!("failed to parse all-time top app row: {e}"))?);
    }

    Ok(result)
}

pub(crate) fn learn_heatmap(
    conn: &Connection,
    days: Option<i64>,
    goal_seconds: Option<i64>,
) -> Result<Vec<LearnHeatmapCell>, String> {
    ensure_heatmap_snapshot_table(conn)?;
    let span_days = learn_heatmap_span_days(days);
    let goal = learn_heatmap_goal_seconds(goal_seconds);
    let lock_goal = parse_i64_config(conn, "heatmap_lock_goal_seconds", HEATMAP_LOCK_GOAL_SECONDS)
        .clamp(0, 86_400);

    let (today_start_ts, today_end_ts) = business_day_window_from_local(Local::now())?;
    let start_ts = today_start_ts - (span_days - 1) * 86_400;
    let end_ts = today_end_ts;

    for i in 0..span_days {
        let day_start_ts = start_ts + i * 86_400;
        if day_start_ts >= today_start_ts {
            continue;
        }
        seal_historical_heatmap_snapshot(conn, day_start_ts, lock_goal)?;
    }

    let start_day_key = business_day_key_from_start(start_ts)?;
    let end_day_key = business_day_key_from_start(end_ts - 1)?;
    let mut snap_stmt = conn
        .prepare(
            "SELECT day_key, learn_seconds, level
             FROM daily_heatmap_snapshot
             WHERE day_key >= ?1 AND day_key <= ?2",
        )
        .map_err(|e| format!("failed to prepare heatmap snapshot query: {e}"))?;

    let snapshot_rows = snap_stmt
        .query_map(params![start_day_key, end_day_key], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| format!("failed to query heatmap snapshots: {e}"))?;

    let mut snapshots: HashMap<String, (i64, String)> = HashMap::new();
    for row in snapshot_rows {
        let (day_key, seconds, level) =
            row.map_err(|e| format!("failed to parse heatmap snapshot row: {e}"))?;
        snapshots.insert(day_key, (seconds, level));
    }

    let mut stmt = conn
        .prepare(
            "SELECT l.start_timestamp,
                    l.duration_ms,
                    COALESCE(r.mapped_type, 'IGNORE') AS mapped_type
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare heatmap query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?.to_uppercase(),
            ))
        })
        .map_err(|e| format!("failed to query heatmap rows: {e}"))?;

    let mut by_day: HashMap<String, i64> = HashMap::new();
    for row in rows {
        let (seg_start, duration_ms, mapped_type) =
            row.map_err(|e| format!("failed to parse heatmap row: {e}"))?;
        if mapped_type != "LEARN" {
            continue;
        }

        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end <= clip_start {
            continue;
        }

        let mut cursor = clip_start;
        while cursor < clip_end {
            let bucket_start = business_day_start_for_timestamp(cursor)?;
            let bucket_end = bucket_start + 86_400;
            let piece_end = clip_end.min(bucket_end);
            let piece_seconds = overlap_seconds(cursor, piece_end, bucket_start, bucket_end);
            if piece_seconds > 0 {
                let day_key = business_day_key_from_start(bucket_start)?;
                *by_day.entry(day_key).or_insert(0) += piece_seconds;
            }
            cursor = piece_end;
        }
    }

    let mut result = Vec::with_capacity(span_days as usize);
    for i in 0..span_days {
        let day_start_ts = start_ts + i * 86_400;
        let day_key = business_day_key_from_start(day_start_ts)?;
        let (learn_seconds, level) = if day_start_ts < today_start_ts {
            if let Some((seconds, locked_level)) = snapshots.get(&day_key) {
                (*seconds, locked_level.clone())
            } else {
                (0, "GRAY".to_string())
            }
        } else {
            let seconds =
                compute_learn_seconds_for_window(conn, day_start_ts, day_start_ts + 86_400)?.max(0);
            let lv = if seconds <= 0 {
                "GRAY"
            } else if seconds < goal {
                "YELLOW"
            } else {
                "GREEN"
            }
            .to_string();
            (seconds, lv)
        };

        result.push(LearnHeatmapCell {
            day: day_key,
            learn_seconds,
            level,
        });
    }

    Ok(result)
}

pub(crate) fn heatmap_goal_seconds_setting(conn: &Connection) -> i64 {
    heatmap_goal_seconds(parse_i64_config(conn, "heatmap_goal_seconds", 7_200))
}

pub(crate) fn set_heatmap_goal_seconds_setting(
    conn: &Connection,
    goal_seconds: i64,
) -> Result<i64, String> {
    let normalized = heatmap_goal_seconds(goal_seconds);
    conn.execute(
        "INSERT INTO app_config (key, value) VALUES ('heatmap_goal_seconds', ?1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        [normalized.to_string()],
    )
    .map_err(|e| format!("failed to save heatmap goal seconds: {e}"))?;
    Ok(normalized)
}

pub(crate) fn usage_stack(
    conn: &Connection,
    days: Option<i64>,
    root_filter: Option<String>,
) -> Result<Vec<UsageStackDay>, String> {
    let span_days = usage_stack_span_days(days);
    let filter = normalize_usage_root_filter(root_filter);
    if !is_usage_stack_root_filter(&filter) {
        return Err("root_filter must be ALL, LEARN, or REST".to_string());
    }

    let (today_start_ts, today_end_ts) = business_day_window_from_local(Local::now())?;
    let start_ts = today_start_ts - (span_days - 1) * 86_400;
    let end_ts = today_end_ts;

    let mut stmt = conn
        .prepare(
            "SELECT l.start_timestamp,
                    l.duration_ms,
                    COALESCE(r.mapped_type, 'IGNORE') AS mapped_type,
                    l.process_name
             FROM app_usage_logs l
             LEFT JOIN app_rules r ON r.process_name = l.process_name
             WHERE l.start_timestamp < ?2
               AND (l.start_timestamp + (l.duration_ms / 1000)) > ?1",
        )
        .map_err(|e| format!("failed to prepare usage stack query: {e}"))?;

    let rows = stmt
        .query_map(params![start_ts, end_ts], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?.max(0),
                row.get::<_, String>(2)?.to_uppercase(),
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("failed to query usage stack rows: {e}"))?;

    #[derive(Default)]
    struct DayAcc {
        total_intervals: Vec<(i64, i64)>,
        learn_intervals: Vec<(i64, i64)>,
        rest_intervals: Vec<(i64, i64)>,
        segments: HashMap<String, i64>,
    }

    let mut by_day: BTreeMap<String, DayAcc> = BTreeMap::new();
    for row in rows {
        let (seg_start, duration_ms, mapped_type, process_name) =
            row.map_err(|e| format!("failed to parse usage stack row: {e}"))?;
        if duration_ms <= 0 {
            continue;
        }

        let include = match filter.as_str() {
            "LEARN" => mapped_type == "LEARN",
            "REST" => mapped_type == "REST",
            _ => true,
        };
        if !include {
            continue;
        }

        let seg_end = seg_start + (duration_ms / 1000);
        let clip_start = seg_start.max(start_ts);
        let clip_end = seg_end.min(end_ts);
        if clip_end <= clip_start {
            continue;
        }

        let mut cursor = clip_start;
        while cursor < clip_end {
            let bucket_start = business_day_start_for_timestamp(cursor)?;
            let bucket_end = bucket_start + 86_400;
            let piece_end = clip_end.min(bucket_end);
            let seconds = overlap_seconds(cursor, piece_end, bucket_start, bucket_end);
            if seconds <= 0 {
                cursor = piece_end;
                continue;
            }

            let day = business_day_key_from_start(bucket_start)?;

            let day_entry = by_day.entry(day).or_default();
            day_entry.total_intervals.push((cursor, piece_end));
            if mapped_type == "LEARN" {
                day_entry.learn_intervals.push((cursor, piece_end));
            } else if mapped_type == "REST" {
                day_entry.rest_intervals.push((cursor, piece_end));
            }

            let seg_name = if mapped_type == "IGNORE" {
                format!("{process_name} (IGNORE)")
            } else {
                process_name.clone()
            };
            *day_entry.segments.entry(seg_name).or_insert(0) += seconds;
            cursor = piece_end;
        }
    }

    let mut result = Vec::new();
    for (i, (day, acc)) in by_day.into_iter().enumerate() {
        if i >= span_days as usize {
            break;
        }

        let mut segments: Vec<UsageStackSegment> = acc
            .segments
            .into_iter()
            .map(|(name, seconds)| UsageStackSegment { name, seconds })
            .collect();
        segments.sort_by(|a, b| b.seconds.cmp(&a.seconds));

        result.push(UsageStackDay {
            day,
            total_seconds: merge_intervals_total(acc.total_intervals).max(0),
            learn_seconds: merge_intervals_total(acc.learn_intervals).max(0),
            rest_seconds: merge_intervals_total(acc.rest_intervals).max(0),
            segments,
        });
    }

    result.reverse();

    Ok(result)
}
