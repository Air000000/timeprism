import { invokeCommand } from "../client";
import type {
  LearnHeatmapCell,
  RecentLog,
  TodaySummary,
  TopApp,
  UsageRootFilter,
  UsageStackDay,
} from "../types";

export async function getTodaySummary(): Promise<TodaySummary> {
  return invokeCommand("get_today_summary");
}

export async function listTopAppsToday(limit = 5): Promise<TopApp[]> {
  return invokeCommand("list_top_apps_today", { limit });
}

export async function listTopAppsAllTime(
  limit = 10,
  rootFilter: UsageRootFilter = "ALL",
  includeIgnore = true,
): Promise<TopApp[]> {
  return invokeCommand("list_top_apps_all_time", { limit, rootFilter, includeIgnore });
}

export async function getLearnHeatmap(
  days = 35,
  goalSeconds = 7200,
): Promise<LearnHeatmapCell[]> {
  return invokeCommand("get_learn_heatmap", { days, goalSeconds });
}

export async function getHeatmapGoalSecondsSetting(): Promise<number> {
  return invokeCommand("get_heatmap_goal_seconds_setting");
}

export async function setHeatmapGoalSecondsSetting(goalSeconds: number): Promise<number> {
  return invokeCommand("set_heatmap_goal_seconds_setting", { goalSeconds });
}

export async function getUsageStack(
  days = 14,
  rootFilter: UsageRootFilter = "ALL",
): Promise<UsageStackDay[]> {
  return invokeCommand("get_usage_stack", { days, rootFilter });
}

export async function listRecentLogs(limit = 12): Promise<RecentLog[]> {
  return invokeCommand("list_recent_logs", { limit });
}
