import { invokeCommand } from "./api/client";
import type {
  AppRule,
  Category,
  DeviationCheck,
  ForegroundCaptureDiagnostic,
  IdleMemoryState,
  IdlePrompt,
  LearnHeatmapCell,
  PendingRuleProcess,
  PrivacySettings,
  RecentLog,
  Reminder,
  TodaySummary,
  TopApp,
  UsageStackDay,
} from "./api/types";

export type {
  AppRule,
  Category,
  DeviationCheck,
  ForegroundCaptureDiagnostic,
  IdleMemoryState,
  IdlePrompt,
  LearnHeatmapCell,
  PendingRuleProcess,
  PrivacySettings,
  RecentLog,
  Reminder,
  TodaySummary,
  TopApp,
  UsageStackDay,
  UsageStackSegment,
} from "./api/types";

export async function listCategories(): Promise<Category[]> {
  return invokeCommand("list_categories");
}

export async function createCategory(input: {
  parent_id: number;
  name: string;
  color_hex?: string;
}): Promise<number> {
  return invokeCommand("create_category", { input });
}

export async function startSession(categoryId: number): Promise<number> {
  return invokeCommand("start_session", { categoryId });
}

export async function stopActiveSession(): Promise<boolean> {
  return invokeCommand("stop_active_session");
}

export async function appendAppUsageLog(input: {
  process_name: string;
  window_title: string;
  start_timestamp: number;
  duration_ms: number;
}): Promise<boolean> {
  return invokeCommand("append_app_usage_log", {
    processName: input.process_name,
    windowTitle: input.window_title,
    startTimestamp: input.start_timestamp,
    durationMs: input.duration_ms,
  });
}

export async function captureForegroundOnce(durationMs = 5000): Promise<boolean> {
  return invokeCommand("capture_foreground_once", { durationMs });
}

export async function getTodaySummary(): Promise<TodaySummary> {
  return invokeCommand("get_today_summary");
}

export async function listTopAppsToday(limit = 5): Promise<TopApp[]> {
  return invokeCommand("list_top_apps_today", { limit });
}

export async function listTopAppsAllTime(
  limit = 10,
  rootFilter: "ALL" | "LEARN" | "REST" = "ALL",
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
  rootFilter: "ALL" | "LEARN" | "REST" = "ALL",
): Promise<UsageStackDay[]> {
  return invokeCommand("get_usage_stack", { days, rootFilter });
}

export async function listRecentLogs(limit = 12): Promise<RecentLog[]> {
  return invokeCommand("list_recent_logs", { limit });
}

export async function saveAppRule(input: {
  process_name: string;
  mapped_type: "LEARN" | "REST" | "IGNORE";
  privacy_level?: "NORMAL" | "BLUR_TITLE" | "WHITELIST_ONLY";
}): Promise<void> {
  return invokeCommand("save_app_rule", { input });
}

export async function checkFocusDeviation(input: {
  process_name: string;
  debounce_seconds?: number;
}): Promise<DeviationCheck> {
  return invokeCommand("check_focus_deviation", {
    processName: input.process_name,
    debounceSeconds: input.debounce_seconds,
  });
}

export async function snoozeFocusGuard(cooldown_seconds = 900): Promise<void> {
  return invokeCommand("snooze_focus_guard", { cooldownSeconds: cooldown_seconds });
}

export async function getPrivacySettings(): Promise<PrivacySettings> {
  return invokeCommand("get_privacy_settings");
}

export async function updatePrivacySettings(input: {
  curtain_enabled: boolean;
  browser_title_mode: "FULL" | "BLUR" | "NONE";
  whitelist_only_enabled: boolean;
}): Promise<void> {
  return invokeCommand("update_privacy_settings", {
    input: {
      curtain_enabled: input.curtain_enabled,
      browser_title_mode: input.browser_title_mode,
      whitelist_only_enabled: input.whitelist_only_enabled,
    },
  });
}

export async function getAutoStartEnabled(): Promise<boolean> {
  return invokeCommand("get_auto_start_enabled");
}

export async function setAutoStartEnabled(enabled: boolean): Promise<boolean> {
  return invokeCommand("set_auto_start_enabled", { enabled });
}

export async function listWhitelist(): Promise<string[]> {
  return invokeCommand("list_whitelist");
}

export async function setWhitelistItem(input: {
  process_name: string;
  enabled: boolean;
}): Promise<void> {
  return invokeCommand("set_whitelist_item", {
    input: {
      process_name: input.process_name,
      enabled: input.enabled,
    },
  });
}

export async function listForegroundCaptureDiagnostics(
  limit = 10,
  uniqueByProcess = true,
): Promise<ForegroundCaptureDiagnostic[]> {
  return invokeCommand("list_foreground_capture_diagnostics", { limit, uniqueByProcess });
}

export async function listAppRules(limit = 200): Promise<AppRule[]> {
  return invokeCommand("list_app_rules", { limit });
}

export async function listPendingRuleProcesses(limit = 10): Promise<PendingRuleProcess[]> {
  return invokeCommand("list_pending_rule_processes", { limit });
}

export async function listPendingIdlePrompts(limit = 5): Promise<IdlePrompt[]> {
  return invokeCommand("list_pending_idle_prompts", { limit });
}

export async function resolveIdlePrompt(input: {
  prompt_id: number;
  decision: "LEARN" | "REST" | "IDLE" | "SKIP";
  remember_this_session?: boolean;
}): Promise<boolean> {
  return invokeCommand("resolve_idle_prompt", { input });
}

export async function getIdleMemoryState(): Promise<IdleMemoryState> {
  return invokeCommand("get_idle_memory_state");
}

export async function clearIdleMemoryState(): Promise<void> {
  return invokeCommand("clear_idle_memory_state");
}

export async function listReminders(
  limit = 50,
  includeCompleted = false,
): Promise<Reminder[]> {
  return invokeCommand("list_reminders", { limit, includeCompleted });
}

export async function listDueReminders(limit = 6): Promise<Reminder[]> {
  return invokeCommand("list_due_reminders", { limit });
}

export async function saveReminder(input: {
  id?: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  remind_at?: number;
  daily_time_minutes?: number;
  weekly_days?: number[];
}): Promise<number> {
  return invokeCommand("save_reminder", { input });
}

export async function deleteReminder(id: number): Promise<boolean> {
  return invokeCommand("delete_reminder", { id });
}

export async function setReminderDone(input: {
  id: number;
  done: boolean;
}): Promise<boolean> {
  return invokeCommand("set_reminder_done", { input });
}

export async function snoozeReminder(id: number, snoozeSeconds = 600): Promise<boolean> {
  return invokeCommand("snooze_reminder", { id, snoozeSeconds });
}

export async function setReminderOrder(input: {
  ordered_ids: number[];
}): Promise<boolean> {
  return invokeCommand("set_reminder_order", { input });
}
