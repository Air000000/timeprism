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
