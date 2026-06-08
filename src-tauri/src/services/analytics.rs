use chrono::Local;
use rusqlite::{params, Connection};

use crate::business_day_window_from_local;
use crate::domain::analytics::{RecentLogEntry, TodaySummary, TopApp};

pub(crate) fn list_recent_log_entries(
    conn: &Connection,
    limit: Option<i64>,
) -> Result<Vec<RecentLogEntry>, String> {
    let cap = limit.unwrap_or(12).clamp(1, 100);
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

    let cap = limit.unwrap_or(5).clamp(1, 20);

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
    let cap = limit.unwrap_or(10).clamp(1, 30);
    let keep_ignore = include_ignore.unwrap_or(true);
    let filter = root_filter
        .unwrap_or_else(|| "ALL".to_string())
        .trim()
        .to_uppercase();

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
