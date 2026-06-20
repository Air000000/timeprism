pub(crate) fn recent_log_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(12).clamp(1, 100)
}

pub(crate) fn top_apps_today_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(5).clamp(1, 20)
}

pub(crate) fn top_apps_all_time_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(10).clamp(1, 30)
}

pub(crate) fn learn_heatmap_span_days(days: Option<i64>) -> i64 {
    days.unwrap_or(35).clamp(7, 2000)
}

pub(crate) fn learn_heatmap_goal_seconds(goal_seconds: Option<i64>) -> i64 {
    goal_seconds.unwrap_or(7200).clamp(0, 86_400)
}

pub(crate) fn heatmap_goal_seconds(goal_seconds: i64) -> i64 {
    goal_seconds.clamp(0, 86_400)
}

pub(crate) fn learn_heatmap_level(learn_seconds: i64, goal_seconds: i64) -> &'static str {
    if learn_seconds <= 0 {
        "GRAY"
    } else if learn_seconds < goal_seconds {
        "YELLOW"
    } else {
        "GREEN"
    }
}

pub(crate) fn usage_stack_span_days(days: Option<i64>) -> i64 {
    days.unwrap_or(14).clamp(3, 366)
}

pub(crate) fn normalize_usage_root_filter(root_filter: Option<String>) -> String {
    root_filter
        .unwrap_or_else(|| "ALL".to_string())
        .trim()
        .to_uppercase()
}

pub(crate) fn is_usage_stack_root_filter(filter: &str) -> bool {
    matches!(filter, "ALL" | "LEARN" | "REST")
}

pub(crate) fn usage_stack_includes_mapped_type(filter: &str, mapped_type: &str) -> bool {
    match filter {
        "LEARN" => mapped_type == "LEARN",
        "REST" => mapped_type == "REST",
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_use_existing_defaults_and_bounds() {
        assert_eq!(recent_log_limit(None), 12);
        assert_eq!(recent_log_limit(Some(0)), 1);
        assert_eq!(recent_log_limit(Some(101)), 100);

        assert_eq!(top_apps_today_limit(None), 5);
        assert_eq!(top_apps_today_limit(Some(0)), 1);
        assert_eq!(top_apps_today_limit(Some(25)), 20);

        assert_eq!(top_apps_all_time_limit(None), 10);
        assert_eq!(top_apps_all_time_limit(Some(0)), 1);
        assert_eq!(top_apps_all_time_limit(Some(35)), 30);
    }

    #[test]
    fn heatmap_and_stack_ranges_use_existing_bounds() {
        assert_eq!(learn_heatmap_span_days(None), 35);
        assert_eq!(learn_heatmap_span_days(Some(3)), 7);
        assert_eq!(learn_heatmap_span_days(Some(2500)), 2000);

        assert_eq!(learn_heatmap_goal_seconds(None), 7200);
        assert_eq!(learn_heatmap_goal_seconds(Some(-1)), 0);
        assert_eq!(learn_heatmap_goal_seconds(Some(90_000)), 86_400);

        assert_eq!(heatmap_goal_seconds(-1), 0);
        assert_eq!(heatmap_goal_seconds(90_000), 86_400);

        assert_eq!(usage_stack_span_days(None), 14);
        assert_eq!(usage_stack_span_days(Some(1)), 3);
        assert_eq!(usage_stack_span_days(Some(400)), 366);
    }

    #[test]
    fn heatmap_level_matches_existing_thresholds() {
        assert_eq!(learn_heatmap_level(0, 7200), "GRAY");
        assert_eq!(learn_heatmap_level(-1, 7200), "GRAY");
        assert_eq!(learn_heatmap_level(7199, 7200), "YELLOW");
        assert_eq!(learn_heatmap_level(7200, 7200), "GREEN");
        assert_eq!(learn_heatmap_level(1, 0), "GREEN");
    }

    #[test]
    fn root_filter_normalizes_whitespace_and_case() {
        assert_eq!(normalize_usage_root_filter(None), "ALL");
        assert_eq!(
            normalize_usage_root_filter(Some(" learn ".to_string())),
            "LEARN",
        );
        assert_eq!(
            normalize_usage_root_filter(Some("rest".to_string())),
            "REST",
        );
    }

    #[test]
    fn usage_stack_filter_validation_matches_existing_allowed_values() {
        assert!(is_usage_stack_root_filter("ALL"));
        assert!(is_usage_stack_root_filter("LEARN"));
        assert!(is_usage_stack_root_filter("REST"));
        assert!(!is_usage_stack_root_filter("IGNORE"));
        assert!(!is_usage_stack_root_filter(""));
    }

    #[test]
    fn usage_stack_include_filter_matches_existing_mapping_rules() {
        assert!(usage_stack_includes_mapped_type("ALL", "LEARN"));
        assert!(usage_stack_includes_mapped_type("ALL", "REST"));
        assert!(usage_stack_includes_mapped_type("ALL", "IGNORE"));

        assert!(usage_stack_includes_mapped_type("LEARN", "LEARN"));
        assert!(!usage_stack_includes_mapped_type("LEARN", "REST"));
        assert!(!usage_stack_includes_mapped_type("LEARN", "IGNORE"));

        assert!(usage_stack_includes_mapped_type("REST", "REST"));
        assert!(!usage_stack_includes_mapped_type("REST", "LEARN"));
        assert!(!usage_stack_includes_mapped_type("REST", "IGNORE"));
    }
}
