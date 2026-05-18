import { invoke } from "@tauri-apps/api/core";

export type Category = {
  id: number;
  parent_id: number | null;
  name: string;
  root_type: "LEARN" | "REST";
  color_hex: string;
};

export type TodaySummary = {
  learn_seconds: number;
  rest_seconds: number;
  active_session_id: number | null;
};

export type TopApp = {
  process_name: string;
  seconds: number;
};

export type LearnHeatmapCell = {
  day: string;
  learn_seconds: number;
  level: "GRAY" | "YELLOW" | "GREEN";
};

export type UsageStackSegment = {
  name: string;
  seconds: number;
};

export type UsageStackDay = {
  day: string;
  total_seconds: number;
  learn_seconds: number;
  rest_seconds: number;
  segments: UsageStackSegment[];
};

export type RecentLog = {
  id: number;
  process_name: string;
  window_title: string;
  start_timestamp: number;
  duration_ms: number;
};

export type DeviationCheck = {
  triggered: boolean;
  process_name: string;
  reason: string;
  active_root_type: "LEARN" | "REST" | null;
  mapped_type: "LEARN" | "REST" | "IGNORE" | null;
  suggested_root_category_id: number | null;
};

export type PrivacySettings = {
  curtain_enabled: boolean;
  browser_title_mode: "FULL" | "BLUR" | "NONE";
  whitelist_only_enabled: boolean;
};

export type ForegroundCaptureDiagnostic = {
  id: number;
  captured_at_ms: number;
  observed_process_name: string;
  observed_window_title: string;
  stored: boolean;
  block_reason: string | null;
  rule_saved: boolean;
  rule_mapped_type: "LEARN" | "REST" | "IGNORE";
};

export type AppRule = {
  process_name: string;
  mapped_type: "LEARN" | "REST" | "IGNORE";
  privacy_level: "NORMAL" | "BLUR_TITLE" | "WHITELIST_ONLY";
  updated_at: number;
};

export type PendingRuleProcess = {
  process_name: string;
  last_seen_timestamp: number;
  last_window_title: string;
  total_seconds: number;
};

export type IdlePrompt = {
  id: number;
  start_timestamp: number;
  end_timestamp: number;
  duration_ms: number;
  deferred_until_timestamp?: number | null;
};

export type IdleMemoryState = {
  remembered_decision: "LEARN" | "REST" | "IDLE" | null;
};

export type Reminder = {
  id: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  sort_order: number;
  remind_at: number | null;
  daily_time_minutes: number | null;
  weekly_days: number[] | null;
  snooze_until: number | null;
  next_due_timestamp: number;
  done: boolean;
  completed_day_key: string | null;
  completed_at: number | null;
  created_at: number;
  updated_at: number;
};

export async function listCategories(): Promise<Category[]> {
  return invoke("list_categories");
}

export async function createCategory(input: {
  parent_id: number;
  name: string;
  color_hex?: string;
}): Promise<number> {
  return invoke("create_category", { input });
}

export async function startSession(categoryId: number): Promise<number> {
  return invoke("start_session", { categoryId });
}

export async function stopActiveSession(): Promise<boolean> {
  return invoke("stop_active_session");
}

export async function appendAppUsageLog(input: {
  process_name: string;
  window_title: string;
  start_timestamp: number;
  duration_ms: number;
}): Promise<boolean> {
  return invoke("append_app_usage_log", {
    processName: input.process_name,
    windowTitle: input.window_title,
    startTimestamp: input.start_timestamp,
    durationMs: input.duration_ms,
  });
}

export async function captureForegroundOnce(durationMs = 5000): Promise<boolean> {
  return invoke("capture_foreground_once", { durationMs });
}

export async function getTodaySummary(): Promise<TodaySummary> {
  return invoke("get_today_summary");
}

export async function listTopAppsToday(limit = 5): Promise<TopApp[]> {
  return invoke("list_top_apps_today", { limit });
}

export async function listTopAppsAllTime(
  limit = 10,
  rootFilter: "ALL" | "LEARN" | "REST" = "ALL",
  includeIgnore = true,
): Promise<TopApp[]> {
  return invoke("list_top_apps_all_time", { limit, rootFilter, includeIgnore });
}

export async function getLearnHeatmap(
  days = 35,
  goalSeconds = 7200,
): Promise<LearnHeatmapCell[]> {
  return invoke("get_learn_heatmap", { days, goalSeconds });
}

export async function getHeatmapGoalSecondsSetting(): Promise<number> {
  return invoke("get_heatmap_goal_seconds_setting");
}

export async function setHeatmapGoalSecondsSetting(goalSeconds: number): Promise<number> {
  return invoke("set_heatmap_goal_seconds_setting", { goalSeconds });
}

export async function getUsageStack(
  days = 14,
  rootFilter: "ALL" | "LEARN" | "REST" = "ALL",
): Promise<UsageStackDay[]> {
  return invoke("get_usage_stack", { days, rootFilter });
}

export async function listRecentLogs(limit = 12): Promise<RecentLog[]> {
  return invoke("list_recent_logs", { limit });
}

export async function saveAppRule(input: {
  process_name: string;
  mapped_type: "LEARN" | "REST" | "IGNORE";
  privacy_level?: "NORMAL" | "BLUR_TITLE" | "WHITELIST_ONLY";
}): Promise<void> {
  return invoke("save_app_rule", { input });
}

export async function checkFocusDeviation(input: {
  process_name: string;
  debounce_seconds?: number;
}): Promise<DeviationCheck> {
  return invoke("check_focus_deviation", {
    processName: input.process_name,
    debounceSeconds: input.debounce_seconds,
  });
}

export async function snoozeFocusGuard(cooldown_seconds = 900): Promise<void> {
  return invoke("snooze_focus_guard", { cooldownSeconds: cooldown_seconds });
}

export async function getPrivacySettings(): Promise<PrivacySettings> {
  return invoke("get_privacy_settings");
}

export async function updatePrivacySettings(input: {
  curtain_enabled: boolean;
  browser_title_mode: "FULL" | "BLUR" | "NONE";
  whitelist_only_enabled: boolean;
}): Promise<void> {
  return invoke("update_privacy_settings", {
    input: {
      curtain_enabled: input.curtain_enabled,
      browser_title_mode: input.browser_title_mode,
      whitelist_only_enabled: input.whitelist_only_enabled,
    },
  });
}

export async function getAutoStartEnabled(): Promise<boolean> {
  return invoke("get_auto_start_enabled");
}

export async function setAutoStartEnabled(enabled: boolean): Promise<boolean> {
  return invoke("set_auto_start_enabled", { enabled });
}

export async function listWhitelist(): Promise<string[]> {
  return invoke("list_whitelist");
}

export async function setWhitelistItem(input: {
  process_name: string;
  enabled: boolean;
}): Promise<void> {
  return invoke("set_whitelist_item", {
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
  return invoke("list_foreground_capture_diagnostics", { limit, uniqueByProcess });
}

export async function listAppRules(limit = 200): Promise<AppRule[]> {
  return invoke("list_app_rules", { limit });
}

export async function listPendingRuleProcesses(limit = 10): Promise<PendingRuleProcess[]> {
  return invoke("list_pending_rule_processes", { limit });
}

export async function listPendingIdlePrompts(limit = 5): Promise<IdlePrompt[]> {
  return invoke("list_pending_idle_prompts", { limit });
}

export async function resolveIdlePrompt(input: {
  prompt_id: number;
  decision: "LEARN" | "REST" | "IDLE" | "SKIP";
  remember_this_session?: boolean;
}): Promise<boolean> {
  return invoke("resolve_idle_prompt", { input });
}

export async function getIdleMemoryState(): Promise<IdleMemoryState> {
  return invoke("get_idle_memory_state");
}

export async function clearIdleMemoryState(): Promise<void> {
  return invoke("clear_idle_memory_state");
}

export async function listReminders(
  limit = 50,
  includeCompleted = false,
): Promise<Reminder[]> {
  return invoke("list_reminders", { limit, includeCompleted });
}

export async function listDueReminders(limit = 6): Promise<Reminder[]> {
  return invoke("list_due_reminders", { limit });
}

export async function saveReminder(input: {
  id?: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  remind_at?: number;
  daily_time_minutes?: number;
  weekly_days?: number[];
}): Promise<number> {
  return invoke("save_reminder", { input });
}

export async function deleteReminder(id: number): Promise<boolean> {
  return invoke("delete_reminder", { id });
}

export async function setReminderDone(input: {
  id: number;
  done: boolean;
}): Promise<boolean> {
  return invoke("set_reminder_done", { input });
}

export async function snoozeReminder(id: number, snoozeSeconds = 600): Promise<boolean> {
  return invoke("snooze_reminder", { id, snoozeSeconds });
}

export async function setReminderOrder(input: {
  ordered_ids: number[];
}): Promise<boolean> {
  return invoke("set_reminder_order", { input });
}
