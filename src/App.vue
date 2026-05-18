<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import GuardView from "./components/GuardView.vue";
import HomeView from "./components/HomeView.vue";
import InsightsView from "./components/InsightsView.vue";
import {
  captureForegroundOnce,
  getAutoStartEnabled,
  getHeatmapGoalSecondsSetting,
  getIdleMemoryState,
  getTodaySummary,
  listAppRules,
  listPendingIdlePrompts,
  listPendingRuleProcesses,
  listReminders,
  getPrivacySettings,
  getLearnHeatmap,
  getUsageStack,
  saveReminder,
  setReminderOrder,
  deleteReminder,
  setAutoStartEnabled,
  setReminderDone,
  snoozeReminder,
  listForegroundCaptureDiagnostics,
  listTopAppsAllTime,
  listRecentLogs,
  listTopAppsToday,
  listWhitelist,
  resolveIdlePrompt,
  saveAppRule,
  setHeatmapGoalSecondsSetting,
  setWhitelistItem,
  type AppRule,
  type ForegroundCaptureDiagnostic,
  type IdleMemoryState,
  type LearnHeatmapCell,
  type IdlePrompt,
  type PendingRuleProcess,
  type PrivacySettings,
  type RecentLog,
  type Reminder,
  type TopApp,
  type TodaySummary,
  type UsageStackDay,
  updatePrivacySettings,
} from "./api";

type LocaleCode = "zh-CN" | "en-US";
const LOCALE_STORAGE_KEY = "timeprism-locale";
const locale = ref<LocaleCode>("zh-CN");

function tx(zh: string, en: string): string {
  return locale.value === "zh-CN" ? zh : en;
}

function applyLocale(next: LocaleCode) {
  locale.value = next;
  if (typeof document !== "undefined") {
    document.documentElement.lang = next;
  }
  try {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, next);
  } catch {
    // Ignore persistence errors in restricted WebView contexts.
  }
}

function initLocale() {
  try {
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored === "zh-CN" || stored === "en-US") {
      applyLocale(stored);
      return;
    }
  } catch {
    // Ignore read errors and continue with system preference.
  }

  const browserLang = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-cn";
  applyLocale(browserLang.startsWith("zh") ? "zh-CN" : "en-US");
}

const topApps = ref<TopApp[]>([]);
const allTimeTopApps = ref<TopApp[]>([]);
const allTimeFilter = ref<"ALL" | "LEARN" | "REST">("ALL");
const allTimeIncludeIgnore = ref(true);
const recentLogs = ref<RecentLog[]>([]);
const learnHeatmap = ref<LearnHeatmapCell[]>([]);
const usageStack = ref<UsageStackDay[]>([]);
const usageStackLearn = ref<UsageStackDay[]>([]);
const usageStackRest = ref<UsageStackDay[]>([]);
const homeUsageStack = ref<UsageStackDay[]>([]);
const usageRootFilter = ref<"ALL" | "LEARN" | "REST">("ALL");
const usageShowIgnore = ref(true);
const selectedUsageDay = ref("");
const selectedUsageProcess = ref("");
const learnGoalSliderMinutes = ref(120);
const viewMonthDate = ref(new Date(new Date().getFullYear(), new Date().getMonth(), 1));
const loadingHome = ref(false);
const loadingInsights = ref(false);
const loadingGuard = ref(false);
const loadingSettings = ref(false);
const settingsLoadedAt = ref(0);
const error = ref("");
const autoCaptureEnabled = ref(true);
const autoCaptureFeedback = ref("自动采样已开启");
const guardFeedback = ref("尚未执行检测");
const guardFeedbackType = ref<"info" | "ok" | "warn" | "error">("info");
const foregroundDiagnostics = ref<ForegroundCaptureDiagnostic[]>([]);
const appRules = ref<AppRule[]>([]);
const pendingRuleProcesses = ref<PendingRuleProcess[]>([]);
const idlePrompts = ref<IdlePrompt[]>([]);
const idleActionLoading = ref(false);
const idleRememberChoice = ref(false);
const idleMemoryState = ref<IdleMemoryState>({ remembered_decision: null });
const ruleSearch = ref("");
const ruleSort = ref<"alpha_asc" | "alpha_desc" | "time_desc" | "time_asc">("alpha_asc");
const privacy = ref<PrivacySettings>({
  curtain_enabled: false,
  browser_title_mode: "BLUR",
  whitelist_only_enabled: false,
});
const autoStartEnabled = ref(false);
const reminders = ref<Reminder[]>([]);
const whitelist = ref<string[]>([]);
const whitelistInput = ref("code.exe");
const privacyFeedback = ref("未保存隐私设置");
const privacyFeedbackType = ref<"info" | "ok" | "warn" | "error">("info");
const reminderActionLoading = ref(false);
const privacyViewMounted = ref(false);
type MainViewKey = "home" | "insights" | "guard" | "privacy";
const currentMainView = ref<MainViewKey>("home");
const mainViews = computed<Array<{ key: MainViewKey; label: string }>>(() => [
  { key: "home", label: tx("首页", "Home") },
  { key: "insights", label: tx("数据看板", "Insights") },
  { key: "guard", label: tx("专注守护", "Focus Guard") },
  { key: "privacy", label: tx("设置", "Settings") },
]);
type InsightsPrimaryViewKey = "history";
type HistorySubViewKey = "topApps" | "allTime" | "recent";
const insightsPrimaryView = ref<InsightsPrimaryViewKey>("history");
const historySubView = ref<HistorySubViewKey>("topApps");
const historySubViews = computed<Array<{ key: HistorySubViewKey; label: string }>>(() => [
  { key: "recent", label: tx("最近记录", "Recent Logs") },
  { key: "topApps", label: tx("今日时长", "Top Apps") },
  { key: "allTime", label: tx("历史总时长", "All-time Usage") },
]);

const guardStep3Done = ref(false);

const todaySummary = ref<TodaySummary>({
  learn_seconds: 0,
  rest_seconds: 0,
  active_session_id: null,
});
const todayLearnSeconds = computed(() => todaySummary.value.learn_seconds ?? 0);
const todayRestSeconds = computed(() => todaySummary.value.rest_seconds ?? 0);
const goalProgressRatio = computed(() => {
  const goal = Math.max(0, learnGoalSliderMinutes.value * 60);
  if (goal <= 0) {
    return 0;
  }
  return Math.max(0, todayLearnSeconds.value / goal);
});
const goalProgressPct = computed(() => `${(goalProgressRatio.value * 100).toFixed(0)}%`);
const goalProgressNum = computed(() => Math.round(goalProgressRatio.value * 100));
const goalProgressFillNum = computed(() => Math.max(0, Math.min(100, goalProgressNum.value)));
const goalOverflowTier = computed<"none" | "active">(() => (
  goalProgressNum.value > 100 ? "active" : "none"
));

const recentSummary = computed(() => {
  const latest = recentLogs.value[0];
  if (!latest) {
    return tx("暂无最近记录", "No recent records");
  }
  return `${cleanProcessName(latest.process_name)} · ${formatClock(latest.start_timestamp)} · ${Math.max(
    0,
    Math.floor(latest.duration_ms / 1000),
  )}s`;
});

const guardStep1Complete = computed(() => pendingRuleProcesses.value.length === 0);
const guardStep2Unlocked = computed(() => guardStep1Complete.value);
const guardStep2Complete = computed(() => idlePrompts.value.length === 0);
const guardStep3Unlocked = computed(() => guardStep1Complete.value && guardStep2Complete.value);
const guardStep4Unlocked = computed(() => guardStep3Unlocked.value && guardStep3Done.value);
const dueReminderCount = computed(() => {
  const now = Math.floor(Date.now() / 1000);
  return reminders.value.filter((item) => !item.done && item.next_due_timestamp <= now).length;
});

const currentStatusLabel = computed(() => {
  if (idlePrompts.value.length > 0) {
    return tx("离开时段待确认", "Idle segments pending");
  }
  if (dueReminderCount.value > 0) {
    return tx(`到点提醒 ${dueReminderCount.value} 条`, `${dueReminderCount.value} reminders due`);
  }
  if (pendingRuleProcesses.value.length > 0) {
    return tx("待处理软件规则", "App rules pending");
  }
  if (autoCaptureEnabled.value) {
    return tx("自动采样运行中", "Auto capture running");
  }
  return tx("自动采样已暂停", "Auto capture paused");
});

const currentStatusTone = computed<"ok" | "warn" | "alert" | "idle">(() => {
  if (idlePrompts.value.length > 0) {
    return "alert";
  }
  if (dueReminderCount.value > 0 || pendingRuleProcesses.value.length > 0) {
    return "warn";
  }
  if (autoCaptureEnabled.value) {
    return "ok";
  }
  return "idle";
});

const guardCurrentStepText = computed(() => {
  if (!guardStep1Complete.value) {
    return tx("步骤 1/4：处理待分类软件", "Step 1/4: Process pending apps");
  }
  if (!guardStep2Complete.value) {
    return tx("步骤 2/4：确认离开时段", "Step 2/4: Resolve idle segments");
  }
  if (!guardStep3Done.value) {
    return tx("步骤 3/4：复核已有规则", "Step 3/4: Review existing rules");
  }
  return tx("步骤 4/4：查看采样诊断", "Step 4/4: Review diagnostics");
});

const guardCurrentStepIndex = computed(() => {
  if (!guardStep1Complete.value) {
    return 1;
  }
  if (!guardStep2Complete.value) {
    return 2;
  }
  if (!guardStep3Done.value) {
    return 3;
  }
  return 4;
});

const guardStepLabels = computed(() => [
  tx("待处理软件", "Pending Apps"),
  tx("离开确认", "Idle Review"),
  tx("规则复核", "Rules Review"),
  tx("采样诊断", "Diagnostics"),
]);
type ThemeMode = "light" | "dark";
const themeMode = ref<ThemeMode>("light");

function switchInsightsSubView(next: InsightsPrimaryViewKey) {
  currentMainView.value = "insights";
  insightsPrimaryView.value = next;
  void refreshInsightsData();
}

function switchHistorySubView(next: HistorySubViewKey) {
  currentMainView.value = "insights";
  insightsPrimaryView.value = "history";
  historySubView.value = next;
  void refreshInsightsData();
}

function applyTheme(nextTheme: ThemeMode) {
  themeMode.value = nextTheme;
  if (typeof document !== "undefined") {
    document.body.setAttribute("data-theme", nextTheme);
  }
  try {
    window.localStorage.setItem("timeprism-theme", nextTheme);
  } catch {
    // Ignore persistence errors in restricted WebView contexts.
  }
}

function toggleThemeMode() {
  applyTheme(themeMode.value === "dark" ? "light" : "dark");
}

function initThemeMode() {
  try {
    const stored = window.localStorage.getItem("timeprism-theme");
    if (stored === "light" || stored === "dark") {
      applyTheme(stored);
      return;
    }
  } catch {
    // Ignore read errors and continue with system preference.
  }

  const prefersDark = typeof window.matchMedia === "function"
    && window.matchMedia("(prefers-color-scheme: dark)").matches;
  applyTheme(prefersDark ? "dark" : "light");
}

const currentMonthKey = computed(() => {
  const year = viewMonthDate.value.getFullYear();
  const month = (viewMonthDate.value.getMonth() + 1).toString().padStart(2, "0");
  return `${year}-${month}`;
});
const monthTitleText = computed(() => {
  const monthNames = locale.value === "zh-CN"
    ? ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]
    : ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  return monthNames[viewMonthDate.value.getMonth()] ?? `${viewMonthDate.value.getMonth() + 1}`;
});
const weekHeaders = computed(() => (
  locale.value === "zh-CN"
    ? ["日", "一", "二", "三", "四", "五", "六"]
    : ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
));
const currentMonthHeatmap = computed(() =>
  learnHeatmap.value.filter((cell) => cell.day.startsWith(currentMonthKey.value)),
);
const fullCurrentMonthHeatmap = computed<LearnHeatmapCell[]>(() => {
  const [yearText, monthText] = currentMonthKey.value.split("-");
  const year = Number.parseInt(yearText, 10);
  const month = Number.parseInt(monthText, 10);
  if (!Number.isFinite(year) || !Number.isFinite(month)) {
    return [];
  }

  const monthDays = new Date(year, month, 0).getDate();
  const byDay = new Map(currentMonthHeatmap.value.map((cell) => [cell.day, cell]));

  const cells: LearnHeatmapCell[] = [];
  for (let day = 1; day <= monthDays; day += 1) {
    const dayKey = `${yearText}-${monthText}-${day.toString().padStart(2, "0")}`;
    const existing = byDay.get(dayKey);
    if (existing) {
      cells.push(existing);
    } else {
      cells.push({
        day: dayKey,
        learn_seconds: 0,
        level: "GRAY",
      });
    }
  }

  return cells;
});

const calendarHeatmapCells = computed<Array<LearnHeatmapCell | null>>(() => {
  const [yearText, monthText] = currentMonthKey.value.split("-");
  const year = Number.parseInt(yearText, 10);
  const month = Number.parseInt(monthText, 10);
  if (!Number.isFinite(year) || !Number.isFinite(month)) {
    return [];
  }

  const firstWeekday = new Date(year, month - 1, 1).getDay();
  const padding = Array.from({ length: firstWeekday }, () => null);
  const cells = [...padding, ...fullCurrentMonthHeatmap.value];
  const trailing = Math.max(0, 42 - cells.length);
  return [...cells, ...Array.from({ length: trailing }, () => null)];
});

function isFutureDay(dayKey: string): boolean {
  return dayKey > currentLocalDayKey();
}

function isTodayDay(dayKey: string): boolean {
  return dayKey === currentLocalDayKey();
}

function heatmapCellClass(cell: LearnHeatmapCell): string[] {
  const classes = ["heat-cell", cell.level.toLowerCase()];
  if (cell.level === "GREEN") {
    const ratio = cell.learn_seconds / currentMonthGreenMaxSeconds.value;
    if (ratio >= 0.88) {
      classes.push("green-4");
    } else if (ratio >= 0.72) {
      classes.push("green-3");
    } else if (ratio >= 0.56) {
      classes.push("green-2");
    } else {
      classes.push("green-1");
    }
  }
  classes.push(isFutureDay(cell.day) ? "future-date" : "past-date");
  if (isTodayDay(cell.day)) {
    classes.push("today-cell");
  }
  return classes;
}

function heatmapDayText(dayKey: string): string {
  return dayKey.slice(-2).replace(/^0/, "");
}

function shiftHeatmapMonth(delta: number) {
  const year = viewMonthDate.value.getFullYear();
  const month = viewMonthDate.value.getMonth();
  viewMonthDate.value = new Date(year, month + delta, 1);
}

function getHeatmapFetchDays(): number {
  const now = new Date();
  const viewYear = viewMonthDate.value.getFullYear();
  const viewMonth = viewMonthDate.value.getMonth();
  const viewStart = new Date(viewYear, viewMonth, 1, 0, 0, 0, 0);
  const diffMs = now.getTime() - viewStart.getTime();
  const diffDays = Math.max(0, Math.ceil(diffMs / 86_400_000));
  const days = diffDays + 62;
  return Math.min(1800, Math.max(120, days));
}

function formatSeconds(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const h = Math.floor(safe / 3600)
    .toString()
    .padStart(2, "0");
  const m = Math.floor((safe % 3600) / 60)
    .toString()
    .padStart(2, "0");
  const s = Math.floor(safe % 60)
    .toString()
    .padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function formatClock(unixSeconds: number): string {
  const date = new Date(unixSeconds * 1000);
  return date.toLocaleTimeString(locale.value, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function formatReminderDateTime(unixSeconds: number): string {
  const date = new Date(unixSeconds * 1000);
  return date.toLocaleString(locale.value, {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function timeMinutesLabel(minutes: number): string {
  const h = Math.floor(minutes / 60)
    .toString()
    .padStart(2, "0");
  const m = Math.floor(minutes % 60)
    .toString()
    .padStart(2, "0");
  return `${h}:${m}`;
}

function reminderWeekdayShortLabel(day: number): string {
  const zh = ["日", "一", "二", "三", "四", "五", "六"];
  const en = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  return locale.value === "zh-CN" ? (zh[day] ?? `${day}`) : (en[day] ?? `${day}`);
}

function reminderWeeklyDaysText(days: number[] | null | undefined): string {
  const safe = (days ?? [])
    .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
    .sort((a, b) => a - b);
  if (safe.length === 0) {
    return tx("未选择", "No days");
  }
  return safe.map((day) => reminderWeekdayShortLabel(day)).join(" ");
}

function reminderDueText(item: Reminder): string {
  if (item.snooze_until && item.snooze_until > Math.floor(Date.now() / 1000)) {
    return `${tx("稍后至", "Snoozed until")} ${formatReminderDateTime(item.snooze_until)}`;
  }

  if (item.repeat_rule === "DAILY") {
    if (item.daily_time_minutes === null) {
      return tx("每日 · 不提醒", "Daily · No reminder");
    }
    const clock = timeMinutesLabel(item.daily_time_minutes ?? 0);
    const doneText = item.done ? tx("（今日已完成）", "(done today)") : "";
    return `${tx("每日", "Daily")} ${clock} ${doneText}`.trim();
  }

  if (item.repeat_rule === "WEEKLY") {
    const clock = item.daily_time_minutes === null
      ? tx("不提醒", "No reminder")
      : timeMinutesLabel(item.daily_time_minutes ?? 0);
    const doneText = item.done ? tx("（本轮已完成）", "(done this turn)") : "";
    return `${tx("每周", "Weekly")} ${reminderWeeklyDaysText(item.weekly_days)} ${clock} ${doneText}`.trim();
  }

  if (item.remind_at === null) {
    return tx("一次性 · 不提醒", "One-time · No reminder");
  }

  return formatReminderDateTime(item.next_due_timestamp);
}

function parseDateTimeLocalToUnix(value: string): number | null {
  if (!value) {
    return null;
  }
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return null;
  }
  return Math.floor(date.getTime() / 1000);
}

function toDateTimeLocalValue(unixSeconds: number): string {
  const date = new Date(unixSeconds * 1000);
  const year = date.getFullYear();
  const month = `${date.getMonth() + 1}`.padStart(2, "0");
  const day = `${date.getDate()}`.padStart(2, "0");
  const hour = `${date.getHours()}`.padStart(2, "0");
  const minute = `${date.getMinutes()}`.padStart(2, "0");
  return `${year}-${month}-${day}T${hour}:${minute}`;
}

function parseClockToMinutes(value: string): number | null {
  const match = value.trim().match(/^(\d{1,2}):(\d{2})$/);
  if (!match) {
    return null;
  }
  const hour = Number.parseInt(match[1], 10);
  const minute = Number.parseInt(match[2], 10);
  if (!Number.isFinite(hour) || !Number.isFinite(minute) || hour < 0 || hour > 23 || minute < 0 || minute > 59) {
    return null;
  }
  return hour * 60 + minute;
}

function allTimeBarWidth(seconds: number): string {
  const max = allTimeTopApps.value[0]?.seconds ?? 0;
  if (max <= 0 || seconds <= 0) {
    return "0%";
  }
  return `${Math.max(8, Math.min(100, (seconds / max) * 100)).toFixed(2)}%`;
}

function topAppsBarWidth(seconds: number): string {
  const max = topApps.value[0]?.seconds ?? 0;
  if (max <= 0 || seconds <= 0) {
    return "0%";
  }
  return `${Math.max(8, Math.min(100, (seconds / max) * 100)).toFixed(2)}%`;
}

function browserModeText(mode: PrivacySettings["browser_title_mode"]): string {
  if (mode === "FULL") {
    return tx("完整标题", "Full title");
  }
  if (mode === "BLUR") {
    return tx("模糊标题", "Blurred title");
  }
  return tx("不采集标题", "No title capture");
}

function mappedTypeText(mappedType: "LEARN" | "REST" | "IGNORE") {
  if (mappedType === "LEARN") {
    return tx("学习", "Learn");
  }
  if (mappedType === "REST") {
    return tx("休息", "Break");
  }
  return tx("未分类", "Unclassified");
}

function syncGoalFromSlider() {
  let total = Number.isFinite(learnGoalSliderMinutes.value)
    ? Math.round(learnGoalSliderMinutes.value / 15) * 15
    : 120;
  total = Math.min(1440, Math.max(0, total));
  learnGoalSliderMinutes.value = total;
  schedulePersistHeatmapGoal();
}

function getHeatmapGoalSeconds(): number {
  syncGoalFromSlider();
  return learnGoalSliderMinutes.value * 60;
}

function usageSegmentWidth(totalSeconds: number, segSeconds: number): string {
  if (totalSeconds <= 0 || segSeconds <= 0) {
    return "0%";
  }
  const pct = (segSeconds / totalSeconds) * 100;
  return `${Math.max(1, Math.min(100, pct)).toFixed(2)}%`;
}

function usageSegmentColor(name: string): string {
  let hash = 0;
  for (let i = 0; i < name.length; i += 1) {
    hash = (hash * 31 + name.charCodeAt(i)) | 0;
  }
  const hue = Math.abs(hash) % 360;
  return `hsl(${hue} 65% 46%)`;
}

function cleanProcessName(name: string): string {
  const normalized = name
    .replace(/^__idle_learn__\.exe$/i, tx("离开时段（已归类为学习）", "Away Segment (Learn)"))
    .replace(/^__idle_rest__\.exe$/i, tx("离开时段（已归类为休息）", "Away Segment (Break)"))
    .replace(/^__idle__\.exe$/i, tx("离开时段", "Away Segment"))
    .replace(/^idle_learn$/i, tx("离开时段（已归类为学习）", "Away Segment (Learn)"))
    .replace(/^idle_rest$/i, tx("离开时段（已归类为休息）", "Away Segment (Break)"))
    .replace(/^system\.idle$/i, tx("离开时段", "Away Segment"))
    .replace(/^idle\.segment$/i, tx("离开时段", "Away Segment"))
    .replace(/\.exe(?=\s*(\(|$))/gi, "")
    .replace(/\s*\([^)]*\)\s*$/g, "")
    .replace(/\s{2,}/g, " ")
    .trim();

  return normalized || tx("未知进程", "Unknown App");
}

const MIN_VISIBLE_SEGMENT_SECONDS = 180;

function visibleSegments(day: UsageStackDay) {
  return day.segments.filter((seg) => seg.seconds >= MIN_VISIBLE_SEGMENT_SECONDS);
}

function visibleTotalSeconds(day: UsageStackDay): number {
  return visibleSegments(day).reduce((sum, seg) => sum + seg.seconds, 0);
}

function logDayKey(unixSeconds: number): string {
  const date = new Date((unixSeconds - 4 * 3600) * 1000);
  const y = date.getFullYear();
  const m = (date.getMonth() + 1).toString().padStart(2, "0");
  const d = date.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function shouldShowSegmentLabel(totalSeconds: number, segSeconds: number): boolean {
  if (totalSeconds <= 0 || segSeconds <= 0) {
    return false;
  }
  return segSeconds / totalSeconds >= 0.12;
}

type RenderSegment = {
  name: string;
  seconds: number;
  color: string;
};

type HomeRhythmBar = {
  day: string;
  label: string;
  totalSeconds: number;
  computerTotalSeconds: number;
  learnSeconds: number;
  restSeconds: number;
  totalHeightPx: number;
  learnHeightPx: number;
  restHeightPx: number;
  totalHeight: string;
  learnHeight: string;
  restHeight: string;
  isToday: boolean;
};

type TooltipState = {
  visible: boolean;
  x: number;
  y: number;
  color: string;
  lines: TooltipLine[];
};

type TooltipLine = {
  text: string;
  header?: boolean;
  strong?: boolean;
};

const stackTooltip = ref<TooltipState>({
  visible: false,
  x: 0,
  y: 0,
  color: "#0f172a",
  lines: [],
});
const stackTooltipSize = ref({
  width: 260,
  height: 120,
});

function estimateTooltipHeight(lineCount: number): number {
  const rowHeight = 18;
  const padding = 20;
  return Math.max(78, Math.min(260, padding + Math.max(1, lineCount) * rowHeight));
}

function measureStackTooltipSize() {
  const node = document.querySelector<HTMLElement>(".stack-tooltip");
  if (!node) {
    return;
  }
  stackTooltipSize.value = {
    width: Math.max(180, Math.ceil(node.offsetWidth)),
    height: Math.max(78, Math.ceil(node.offsetHeight)),
  };
}

function ratioLabel(part: number, total: number): string {
  if (part <= 0 || total <= 0) {
    return "";
  }
  const raw = (part / total) * 100;
  if (raw < 1) {
    return "<1%";
  }
  return `${Math.round(raw)}%`;
}

function lightenColor(color: string): string {
  const hex = color.match(/^#([0-9a-f]{6})$/i);
  if (hex) {
    const raw = hex[1];
    const r = Number.parseInt(raw.slice(0, 2), 16);
    const g = Number.parseInt(raw.slice(2, 4), 16);
    const b = Number.parseInt(raw.slice(4, 6), 16);
    const boost = (v: number) => Math.min(255, Math.round(v + (255 - v) * 0.18));
    return `rgb(${boost(r)} ${boost(g)} ${boost(b)})`;
  }

  const hsl = color.match(/^hsl\((\d+)\s+(\d+)%\s+(\d+)%\)$/i);
  if (hsl) {
    const h = Number.parseInt(hsl[1], 10);
    const s = Number.parseInt(hsl[2], 10);
    const l = Number.parseInt(hsl[3], 10);
    return `hsl(${h} ${s}% ${Math.min(78, l + 12)}%)`;
  }

  return color;
}

function findUsageDay(stack: UsageStackDay[], day: string): UsageStackDay | null {
  return stack.find((item) => item.day === day) ?? null;
}

function buildOverviewBreakdownLines(day: UsageStackDay, root: "LEARN" | "REST"): TooltipLine[] {
  const detail = root === "LEARN" ? findUsageDay(usageStackLearn.value, day.day) : findUsageDay(usageStackRest.value, day.day);
  const rootSeconds = root === "LEARN" ? day.learn_seconds : day.rest_seconds;
  const rootPercent = ratioLabel(rootSeconds, day.total_seconds) || "<1%";
  const rootName = mappedTypeText(root);

  const lines: TooltipLine[] = [
    { text: tx(`今日电脑使用时长中，${rootPercent} 的时间在 ${rootName}`, `${rootPercent} of today's computer time was spent on ${rootName}`), header: true },
    { text: tx("在这段时间中：", "Within this period:"), header: true },
  ];

  if (!detail || detail.segments.length === 0 || rootSeconds <= 0) {
    lines.push({ text: tx("暂无更细的应用统计", "No finer-grained app stats yet") });
    return lines;
  }

  const top2 = detail.segments.slice(0, 2);
  const topSeconds = top2.reduce((sum, seg) => sum + seg.seconds, 0);
  const otherSeconds = Math.max(0, rootSeconds - topSeconds);

  for (const seg of top2) {
    const pct = ratioLabel(seg.seconds, rootSeconds);
    if (!pct) {
      continue;
    }
    lines.push({ text: tx(`${pct} 的时间在使用 ${cleanProcessName(seg.name)}`, `${pct} spent in ${cleanProcessName(seg.name)}`), strong: true });
  }
  if (otherSeconds > 0) {
    const otherPct = ratioLabel(otherSeconds, rootSeconds);
    if (otherPct) {
      lines.push({ text: tx(`${otherPct} 的时间在使用其他软件`, `${otherPct} spent in other apps`), strong: true });
    }
  }

  return lines;
}

function dayRenderSegments(day: UsageStackDay): RenderSegment[] {
  if (usageRootFilter.value === "ALL") {
    const result: RenderSegment[] = [];
    if (day.learn_seconds > 0) {
      result.push({ name: "LEARN", seconds: day.learn_seconds, color: "#16a34a" });
    }
    if (day.rest_seconds > 0) {
      result.push({ name: "REST", seconds: day.rest_seconds, color: "#8ec5ff" });
    }
    const ignoreSeconds = Math.max(0, day.total_seconds - day.learn_seconds - day.rest_seconds);
    if (usageShowIgnore.value && ignoreSeconds > 0) {
      result.push({ name: "IGNORE", seconds: ignoreSeconds, color: "#64748b" });
    }
    return result;
  }

  return visibleSegments(day).map((seg) => ({
    name: seg.name,
    seconds: seg.seconds,
    color: usageSegmentColor(seg.name),
  }));
}

function dayRenderTotal(day: UsageStackDay): number {
  return dayRenderSegments(day).reduce((sum, seg) => sum + seg.seconds, 0);
}

const heatmapByDay = computed(() => new Map(learnHeatmap.value.map((cell) => [cell.day, cell])));
const currentMonthGreenMaxSeconds = computed(() =>
  Math.max(
    1,
    ...fullCurrentMonthHeatmap.value
      .filter((cell) => cell.level === "GREEN")
      .map((cell) => cell.learn_seconds),
  )
);

const HOME_RHYTHM_MIN_HEIGHT = 18;
const HOME_RHYTHM_MAX_HEIGHT = 176;

function homeRhythmHeightForRatio(ratio: number): number {
  if (ratio <= 0) {
    return HOME_RHYTHM_MIN_HEIGHT;
  }
  const scaled = HOME_RHYTHM_MIN_HEIGHT + ratio * (HOME_RHYTHM_MAX_HEIGHT - HOME_RHYTHM_MIN_HEIGHT);
  return Math.round(Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.min(HOME_RHYTHM_MAX_HEIGHT, scaled)));
}

function reminderGroupRank(item: Reminder): number {
  return item.done ? 1 : 0;
}

function reminderRepeatRank(rule: Reminder["repeat_rule"]): number {
  if (rule === "NONE") {
    return 0;
  }
  if (rule === "DAILY") {
    return 1;
  }
  return 2;
}

function compareReminders(a: Reminder, b: Reminder): number {
  return reminderGroupRank(a) - reminderGroupRank(b)
    || a.sort_order - b.sort_order
    || reminderRepeatRank(a.repeat_rule) - reminderRepeatRank(b.repeat_rule)
    || a.next_due_timestamp - b.next_due_timestamp
    || b.updated_at - a.updated_at
    || a.id - b.id;
}

function sortedReminders(items: Reminder[]): Reminder[] {
  return [...items].sort(compareReminders);
}

const homeMonthRhythmBars = computed<HomeRhythmBar[]>(() => {
  const today = new Date();
  const byDay = new Map(homeUsageStack.value.map((item) => [item.day, item]));
  const dayKeys = Array.from({ length: 7 }, (_, index) => {
    const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() - (6 - index));
    return localDayKeyFromDate(date);
  });
  const bars = dayKeys.map((dayKey) => {
    const day = byDay.get(dayKey);
    const learnSeconds = day?.learn_seconds ?? 0;
    const restSeconds = day?.rest_seconds ?? 0;
    const computerTotalSeconds = day?.total_seconds ?? 0;
    const totalSeconds = learnSeconds + restSeconds;
    return {
      day: dayKey,
      label: dayKey.slice(-2).replace(/^0/, ""),
      totalSeconds,
      computerTotalSeconds,
      learnSeconds,
      restSeconds,
      isToday: dayKey === currentLocalDayKey(),
    };
  });
  const maxSeconds = Math.max(1, ...bars.map((item) => item.totalSeconds));
  return bars.map((item) => {
    const ratio = item.totalSeconds <= 0 ? 0 : item.totalSeconds / maxSeconds;
    const totalHeightPx = homeRhythmHeightForRatio(ratio);
    const pieceTotal = Math.max(1, item.learnSeconds + item.restSeconds);
    const learnHeightPx = item.learnSeconds > 0
      ? Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.round(totalHeightPx * (item.learnSeconds / pieceTotal)))
      : 0;
    const restHeightPx = item.restSeconds > 0
      ? Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.round(totalHeightPx * (item.restSeconds / pieceTotal)))
      : 0;
    return {
      day: item.day,
      label: item.label,
      totalSeconds: item.totalSeconds,
      computerTotalSeconds: item.computerTotalSeconds,
      learnSeconds: item.learnSeconds,
      restSeconds: item.restSeconds,
      totalHeightPx,
      learnHeightPx,
      restHeightPx,
      totalHeight: `${totalHeightPx}px`,
      learnHeight: `${learnHeightPx}px`,
      restHeight: `${restHeightPx}px`,
      isToday: item.isToday,
    };
  });
});

const homeMonthGoalProgress = computed(() => {
  const today = new Date();
  const year = today.getFullYear();
  const month = today.getMonth();
  const elapsedDays = today.getDate();
  let goalDays = 0;
  for (let day = 1; day <= elapsedDays; day += 1) {
    const dayKey = localDayKeyFromDate(new Date(year, month, day));
    if ((heatmapByDay.value.get(dayKey)?.level ?? "GRAY") === "GREEN") {
      goalDays += 1;
    }
  }
  return {
    goalDays,
    elapsedDays,
  };
});

const homeMonthActiveStreakDays = computed(() => {
  const today = new Date();
  let streak = 0;
  for (let day = today.getDate(); day >= 1; day -= 1) {
    const dayKey = localDayKeyFromDate(new Date(today.getFullYear(), today.getMonth(), day));
    const cell = heatmapByDay.value.get(dayKey);
    if ((cell?.learn_seconds ?? 0) <= 0) {
      break;
    }
    streak += 1;
  }
  return streak;
});

const homePendingSummary = computed(() => {
  const parts: string[] = [];
  if (pendingRuleProcesses.value.length > 0) {
    parts.push(tx(`待分类 ${pendingRuleProcesses.value.length}`, `${pendingRuleProcesses.value.length} pending apps`));
  }
  if (idlePrompts.value.length > 0) {
    parts.push(tx(`待确认 ${idlePrompts.value.length}`, `${idlePrompts.value.length} idle reviews`));
  }
  if (dueReminderCount.value > 0) {
    parts.push(tx(`提醒 ${dueReminderCount.value}`, `${dueReminderCount.value} reminders`));
  }
  if (parts.length === 0) {
    return tx("当前没有新的待处理项，首页会保持安静。", "No pending items right now, so home stays quiet.");
  }
  return parts.join(" · ");
});

function tooltipLinesForSegment(day: UsageStackDay, seg: RenderSegment): TooltipLine[] {
  if (usageRootFilter.value === "ALL") {
    if (seg.name === "LEARN" || seg.name === "REST") {
      return buildOverviewBreakdownLines(day, seg.name);
    }
    return [{ text: tx(`未分类 | ${formatSeconds(seg.seconds)}`, `Unclassified | ${formatSeconds(seg.seconds)}`) }];
  }

  const percent = ratioLabel(seg.seconds, dayRenderTotal(day)) || "<1%";
  return [
    { text: `${cleanProcessName(seg.name)}`, strong: true },
    { text: tx(`占当日可见时长：${percent}`, `Visible share today: ${percent}`) },
    { text: tx(`时长：${formatSeconds(seg.seconds)}`, `Duration: ${formatSeconds(seg.seconds)}`) },
  ];
}

function clampTooltipPosition(
  clientX: number,
  clientY: number,
  size: { width: number; height: number } = stackTooltipSize.value,
): { x: number; y: number } {
  const pad = 8;
  const gap = 8;
  const width = Math.max(160, size.width);
  const height = Math.max(78, size.height);
  let x = clientX + gap;
  let y = clientY - height - gap;
  if (x + width > window.innerWidth - pad) {
    x = clientX - width - gap;
  }
  if (y < pad) {
    y = clientY + gap;
  }
  const maxX = Math.max(pad, window.innerWidth - width - pad);
  const maxY = Math.max(pad, window.innerHeight - height - pad);
  return {
    x: Math.max(pad, Math.min(x, maxX)),
    y: Math.max(pad, Math.min(y, maxY)),
  };
}

async function handleSegmentMouseEnter(event: MouseEvent, day: UsageStackDay, seg: RenderSegment) {
  const lines = tooltipLinesForSegment(day, seg);
  stackTooltipSize.value = {
    width: stackTooltipSize.value.width,
    height: estimateTooltipHeight(lines.length),
  };
  const pos = clampTooltipPosition(event.clientX, event.clientY, stackTooltipSize.value);
  stackTooltip.value = {
    visible: true,
    x: pos.x,
    y: pos.y,
    color: lightenColor(seg.color),
    lines,
  };
  await nextTick();
  measureStackTooltipSize();
  const recalculatedPos = clampTooltipPosition(event.clientX, event.clientY);
  stackTooltip.value.x = recalculatedPos.x;
  stackTooltip.value.y = recalculatedPos.y;
}

function handleSegmentMouseMove(event: MouseEvent) {
  if (!stackTooltip.value.visible) {
    return;
  }
  const pos = clampTooltipPosition(event.clientX, event.clientY);
  stackTooltip.value.x = pos.x;
  stackTooltip.value.y = pos.y;
}

function handleSegmentMouseLeave() {
  stackTooltip.value.visible = false;
}

async function handleSegmentClick(day: UsageStackDay, seg: RenderSegment) {
  if (usageRootFilter.value !== "ALL") {
    return;
  }

  if (seg.name !== "LEARN" && seg.name !== "REST") {
    return;
  }

  selectedUsageDay.value = day.day;
  await switchUsageFilter(seg.name === "LEARN" ? "LEARN" : "REST");
}

function topSegmentSummary(day: UsageStackDay): string {
  if (usageRootFilter.value === "ALL") {
    const base = `${mappedTypeText("LEARN")} ${formatSeconds(day.learn_seconds)} | ${mappedTypeText("REST")} ${formatSeconds(day.rest_seconds)}`;
    const ignoreSeconds = Math.max(0, day.total_seconds - day.learn_seconds - day.rest_seconds);
    if (usageShowIgnore.value && ignoreSeconds > 0) {
      return `${base} | ${mappedTypeText("IGNORE")} ${formatSeconds(ignoreSeconds)}`;
    }
    return base;
  }

  const top3 = visibleSegments(day).slice(0, 3);
  if (top3.length === 0) {
    return tx("<5分钟段已隐藏", "<5 min segments hidden>");
  }
  return top3
    .map((seg) => {
      const shownTotal = visibleTotalSeconds(day);
      const ratio = shownTotal > 0 ? Math.round((seg.seconds / shownTotal) * 100) : 0;
      return `${cleanProcessName(seg.name)} ${ratio}%`;
    })
    .join(" | ");
}

const selectedUsageDayData = computed(() => {
  if (!selectedUsageDay.value) {
    return usageStack.value[0] ?? null;
  }
  return usageStack.value.find((day) => day.day === selectedUsageDay.value) ?? usageStack.value[0] ?? null;
});

const usageFilterLabel = computed(() => {
  if (usageRootFilter.value === "LEARN") {
    return tx("仅看学习", "Learn only");
  }
  if (usageRootFilter.value === "REST") {
    return tx("仅看休息", "Break only");
  }
  return tx("总览", "Overview");
});

function selectUsageDay(day: string) {
  selectedUsageDay.value = day;
  selectedUsageProcess.value = "";
}

const processDrillCandidates = computed(() => {
  if (usageRootFilter.value === "ALL") {
    return [];
  }
  return selectedUsageDayData.value?.segments ?? [];
});

const linkedRecentLogs = computed(() => {
  const day = selectedUsageDayData.value?.day;
  if (!day) {
    return [];
  }

  let logs = recentLogs.value.filter((item) => logDayKey(item.start_timestamp) === day);

  if (selectedUsageProcess.value) {
    const target = selectedUsageProcess.value.toLowerCase();
    logs = logs.filter((item) => item.process_name.toLowerCase() === target);
  }

  return logs;
});

const filteredSortedRules = computed(() => {
  const q = ruleSearch.value.trim().toLowerCase();
  let list = appRules.value;
  if (q) {
    list = list.filter((rule) => rule.process_name.toLowerCase().includes(q));
  }

  const sorted = [...list];
  if (ruleSort.value === "alpha_asc") {
    sorted.sort((a, b) => a.process_name.localeCompare(b.process_name));
  } else if (ruleSort.value === "alpha_desc") {
    sorted.sort((a, b) => b.process_name.localeCompare(a.process_name));
  } else if (ruleSort.value === "time_desc") {
    sorted.sort((a, b) => b.updated_at - a.updated_at);
  } else {
    sorted.sort((a, b) => a.updated_at - b.updated_at);
  }
  return sorted;
});

const currentIdlePrompt = computed(() => idlePrompts.value[0] ?? null);

const reminderListForPanel = computed(() => sortedReminders(reminders.value));

function localDayKeyFromDate(date: Date): string {
  return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, "0")}-${date.getDate().toString().padStart(2, "0")}`;
}

function currentLocalDayKey(): string {
  return localDayKeyFromDate(new Date());
}

function currentLocalWeekday(): number {
  return new Date().getDay();
}

function dayKeyFromUnixSeconds(unixSeconds: number | null): string | null {
  if (unixSeconds === null || !Number.isFinite(unixSeconds)) {
    return null;
  }
  return localDayKeyFromDate(new Date(unixSeconds * 1000));
}

function reminderIsVisibleToday(item: Reminder): boolean {
  const todayKey = currentLocalDayKey();
  const completedToday = item.completed_day_key === todayKey;

  if (item.repeat_rule === "DAILY") {
    return true;
  }

  if (item.repeat_rule === "WEEKLY") {
    const todayWeekday = currentLocalWeekday();
    return completedToday || (item.weekly_days ?? []).includes(todayWeekday);
  }

  if (item.remind_at === null) {
    return !item.done;
  }

  const oneShotDayKey = dayKeyFromUnixSeconds(item.remind_at) ?? dayKeyFromUnixSeconds(item.created_at);
  return completedToday || oneShotDayKey === todayKey;
}

const homeScheduleItems = computed(() =>
  sortedReminders(reminders.value)
    .filter((item) => reminderIsVisibleToday(item))
);

function formatIdlePromptSpan(item: IdlePrompt): string {
  const start = new Date(item.start_timestamp * 1000);
  const end = new Date(item.end_timestamp * 1000);
  const durationSeconds = Math.max(0, Math.floor(item.duration_ms / 1000));
  return `${start.toLocaleTimeString(locale.value, { hour: "2-digit", minute: "2-digit" })} - ${end.toLocaleTimeString(locale.value, { hour: "2-digit", minute: "2-digit" })} (${formatSeconds(durationSeconds)})`;
}

function selectUsageProcess(processName: string) {
  selectedUsageProcess.value = processName;
}

function openGuardWorkflow() {
  currentMainView.value = "guard";
  void refreshGuardData();
}

function setMainView(next: MainViewKey) {
  currentMainView.value = next;
  if (next === "insights" && insightsPrimaryView.value === "history") {
    historySubView.value = "topApps";
  }
  if (next === "insights") {
    void refreshInsightsData();
  } else if (next === "guard") {
    void refreshGuardData();
  } else if (next === "privacy") {
    privacyViewMounted.value = true;
    window.setTimeout(() => {
      void refreshSettingsData();
    }, 0);
  }
}

function markGuardStep3Done() {
  if (!guardStep3Unlocked.value) {
    return;
  }
  guardStep3Done.value = true;
  guardFeedbackType.value = "ok";
  guardFeedback.value = tx("规则复核已完成，可进入采样诊断。", "Rules review completed. Diagnostics unlocked.");
}

async function switchUsageFilter(filter: "ALL" | "LEARN" | "REST") {
  selectedUsageProcess.value = "";
  usageRootFilter.value = filter;
  await refreshInsightsData();
}

async function backToUsageOverview() {
  await switchUsageFilter("ALL");
}

function setErrorMessage(e: unknown) {
  error.value = `${e}`;
}

async function refreshHomeData() {
  if (loadingHome.value) {
    return;
  }

  loadingHome.value = true;
  try {
    const [summary, logs, heatmap, pendingRules, pendingIdle, reminderRows, rhythmStack] = await Promise.all([
      getTodaySummary(),
      listRecentLogs(12),
      getLearnHeatmap(getHeatmapFetchDays(), getHeatmapGoalSeconds()),
      listPendingRuleProcesses(10),
      listPendingIdlePrompts(3),
      listReminders(120, true),
      getUsageStack(7, "ALL"),
    ]);
    todaySummary.value = summary;
    recentLogs.value = logs;
    learnHeatmap.value = heatmap;
    pendingRuleProcesses.value = pendingRules;
    idlePrompts.value = pendingIdle;
    reminders.value = reminderRows;
    homeUsageStack.value = rhythmStack;
  } catch (e) {
    setErrorMessage(e);
  } finally {
    loadingHome.value = false;
  }
}

async function refreshInsightsData() {
  if (loadingInsights.value) {
    return;
  }

  loadingInsights.value = true;
  try {
    const [apps, allTimeApps, logs, heatmap, stack, stackLearn, stackRest] = await Promise.all([
      listTopAppsToday(6),
      listTopAppsAllTime(10, allTimeFilter.value, allTimeIncludeIgnore.value),
      listRecentLogs(12),
      getLearnHeatmap(getHeatmapFetchDays(), getHeatmapGoalSeconds()),
      getUsageStack(14, usageRootFilter.value),
      getUsageStack(14, "LEARN"),
      getUsageStack(14, "REST"),
    ]);
    topApps.value = apps;
    allTimeTopApps.value = allTimeApps;
    recentLogs.value = logs;
    learnHeatmap.value = heatmap;
    usageStack.value = stack;
    usageStackLearn.value = stackLearn;
    usageStackRest.value = stackRest;
    if (!selectedUsageDay.value || !stack.find((item) => item.day === selectedUsageDay.value)) {
      selectedUsageDay.value = stack[0]?.day ?? "";
      selectedUsageProcess.value = "";
    }
  } catch (e) {
    setErrorMessage(e);
  } finally {
    loadingInsights.value = false;
  }
}

async function refreshGuardData() {
  if (loadingGuard.value) {
    return;
  }

  loadingGuard.value = true;
  try {
    const [diagnostics, rules, pendingRules, pendingIdle, idleMemory] = await Promise.all([
      listForegroundCaptureDiagnostics(10, true),
      listAppRules(300),
      listPendingRuleProcesses(10),
      listPendingIdlePrompts(3),
      getIdleMemoryState(),
    ]);
    foregroundDiagnostics.value = diagnostics;
    appRules.value = rules;
    pendingRuleProcesses.value = pendingRules;
    idlePrompts.value = pendingIdle;
    idleMemoryState.value = idleMemory;
  } catch (e) {
    setErrorMessage(e);
  } finally {
    loadingGuard.value = false;
  }
}

async function refreshSettingsData() {
  if (loadingSettings.value) {
    return;
  }

  const now = Date.now();
  if (settingsLoadedAt.value > 0 && now - settingsLoadedAt.value < 60_000) {
    return;
  }

  loadingSettings.value = true;
  try {
    await Promise.all([refreshPrivacy(), refreshAutoStartSetting()]);
    settingsLoadedAt.value = Date.now();
  } finally {
    loadingSettings.value = false;
  }
}

async function refreshData() {
  await refreshHomeData();
  if (currentMainView.value === "insights") {
    await refreshInsightsData();
  } else if (currentMainView.value === "guard") {
    await refreshGuardData();
  } else if (currentMainView.value === "privacy") {
    await refreshSettingsData();
  }
}

async function handleResolveIdle(
  decision: "LEARN" | "REST" | "IDLE" | "SKIP",
  promptId?: number,
) {
  const prompt = promptId
    ? idlePrompts.value.find((item) => item.id === promptId) ?? null
    : currentIdlePrompt.value;
  if (!prompt || idleActionLoading.value) {
    return;
  }

  idleActionLoading.value = true;
  try {
    const rememberChoice = idleRememberChoice.value && decision !== "SKIP";
    await resolveIdlePrompt({
      prompt_id: prompt.id,
      decision,
      remember_this_session: rememberChoice,
    });

    if (decision === "LEARN") {
      guardFeedbackType.value = "ok";
      guardFeedback.value = rememberChoice
        ? tx("已将该空闲时段归类为学习，并记忆本次选择。", "This idle segment is marked as Learn and remembered.")
        : tx("已将该空闲时段归类为学习。", "This idle segment is marked as Learn.");
    } else if (decision === "REST") {
      guardFeedbackType.value = "ok";
      guardFeedback.value = rememberChoice
        ? tx("已将该空闲时段归类为休息，并记忆本次选择。", "This idle segment is marked as Break and remembered.")
        : tx("已将该空闲时段归类为休息。", "This idle segment is marked as Break.");
    } else if (decision === "IDLE") {
      guardFeedbackType.value = "info";
      guardFeedback.value = rememberChoice
        ? tx("已将该空闲时段归类为离开，并记忆本次选择。", "This idle segment is marked as Away and remembered.")
        : tx("已将该空闲时段归类为离开（不计入学习/休息）。", "This idle segment is marked as Away (excluded from Learn/Break).");
    } else {
      guardFeedbackType.value = "warn";
      guardFeedback.value = tx("该空闲时段已暂缓，后续将继续采样。", "This idle segment is postponed. Sampling will continue.");
    }

    idleRememberChoice.value = false;
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    guardFeedbackType.value = "error";
    guardFeedback.value = tx(`空闲时段分类失败：${e}`, `Failed to classify idle segment: ${e}`);
  } finally {
    idleActionLoading.value = false;
  }
}

function captureBlockReasonText(reason: string | null): string {
  if (!reason) {
    return "-";
  }
  if (reason === "baseline_only") {
    return tx("首次采样仅建立基线", "First sample only sets baseline");
  }
  if (reason === "curtain_enabled") {
    return tx("被窗帘模式拦截", "Blocked by Curtain mode");
  }
  if (reason === "incognito_window") {
    return tx("无痕/隐私窗口拦截", "Blocked by incognito/private window");
  }
  if (reason === "whitelist_blocked") {
    return tx("白名单策略降级", "Whitelisted-only policy fallback");
  }
  if (reason === "no_foreground_window") {
    return tx("未获取到前台窗口", "No foreground window detected");
  }
  if (reason === "elapsed_too_short") {
    return tx("采样间隔过短", "Sampling interval too short");
  }
  return reason;
}

function captureRuleText(item: ForegroundCaptureDiagnostic): string {
  if (!item.rule_saved) {
    return tx("未分类（规则未保存）", "Unclassified (rule not saved)");
  }
  return mappedTypeText(item.rule_mapped_type);
}

function canSaveRuleFromDiagnostic(item: ForegroundCaptureDiagnostic): boolean {
  return item.observed_process_name !== "unknown.exe";
}

async function handleSaveRuleFromDiagnostic(
  item: ForegroundCaptureDiagnostic,
  mappedType: "LEARN" | "REST" | "IGNORE",
) {
  if (!canSaveRuleFromDiagnostic(item)) {
    return;
  }
  try {
    await saveAppRule({
      process_name: item.observed_process_name,
      mapped_type: mappedType,
      privacy_level: "NORMAL",
    });
    guardFeedbackType.value = "ok";
    guardFeedback.value = tx(
      `已保存规则：${item.observed_process_name} -> ${mappedTypeText(mappedType)}`,
      `Rule saved: ${item.observed_process_name} -> ${mappedTypeText(mappedType)}`,
    );
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    guardFeedbackType.value = "error";
    guardFeedback.value = tx(`保存规则失败：${e}`, `Failed to save rule: ${e}`);
  }
}

async function refreshPrivacy() {
  try {
    const [settings, list] = await Promise.all([getPrivacySettings(), listWhitelist()]);
    privacy.value = settings;
    whitelist.value = list;
  } catch (e) {
    setErrorMessage(e);
  }
}

async function refreshAutoStartSetting() {
  try {
    autoStartEnabled.value = await getAutoStartEnabled();
  } catch (e) {
    setErrorMessage(e);
  }
}

async function handleUpdateExistingRule(rule: AppRule) {
  try {
    await saveAppRule({
      process_name: rule.process_name,
      mapped_type: rule.mapped_type,
      privacy_level: rule.privacy_level,
    });
    guardFeedbackType.value = "ok";
    guardFeedback.value = tx(
      `已更新规则：${rule.process_name} -> ${mappedTypeText(rule.mapped_type)}`,
      `Rule updated: ${rule.process_name} -> ${mappedTypeText(rule.mapped_type)}`,
    );
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    guardFeedbackType.value = "error";
    guardFeedback.value = tx(`更新规则失败：${e}`, `Failed to update rule: ${e}`);
  }
}

async function handleSavePendingRule(
  item: PendingRuleProcess,
  mappedType: "LEARN" | "REST" | "IGNORE",
) {
  try {
    await saveAppRule({
      process_name: item.process_name,
      mapped_type: mappedType,
      privacy_level: "NORMAL",
    });
    guardFeedbackType.value = "ok";
    guardFeedback.value = tx(
      `已保存规则：${item.process_name} -> ${mappedTypeText(mappedType)}`,
      `Rule saved: ${item.process_name} -> ${mappedTypeText(mappedType)}`,
    );
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    guardFeedbackType.value = "error";
    guardFeedback.value = tx(`保存规则失败：${e}`, `Failed to save rule: ${e}`);
  }
}

async function handleSavePrivacySettings() {
  try {
    await setAutoStartEnabled(autoStartEnabled.value);
    await updatePrivacySettings({
      curtain_enabled: privacy.value.curtain_enabled,
      browser_title_mode: privacy.value.browser_title_mode,
      whitelist_only_enabled: privacy.value.whitelist_only_enabled,
    });
    privacyFeedbackType.value = "ok";
    privacyFeedback.value = tx(
      `隐私设置已保存（浏览器模式：${browserModeText(privacy.value.browser_title_mode)}）。`,
      `Privacy settings saved (browser mode: ${browserModeText(privacy.value.browser_title_mode)}).`,
    );
    await refreshPrivacy();
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    privacyFeedbackType.value = "error";
    privacyFeedback.value = tx(`保存失败：${e}`, `Save failed: ${e}`);
  }
}

async function handleAddWhitelist() {
  const processName = whitelistInput.value.trim().toLowerCase();
  if (!processName) {
    return;
  }

  try {
    await setWhitelistItem({
      process_name: processName,
      enabled: true,
    });
    whitelistInput.value = "";
    privacyFeedbackType.value = "ok";
    privacyFeedback.value = tx(`已加入白名单：${processName}`, `Added to whitelist: ${processName}`);
    await refreshPrivacy();
  } catch (e) {
    setErrorMessage(e);
    privacyFeedbackType.value = "error";
    privacyFeedback.value = tx(`添加失败：${e}`, `Add failed: ${e}`);
  }
}

async function handleRemoveWhitelist(processName: string) {
  try {
    await setWhitelistItem({
      process_name: processName,
      enabled: false,
    });
    privacyFeedbackType.value = "warn";
    privacyFeedback.value = tx(`已移除白名单：${processName}`, `Removed from whitelist: ${processName}`);
    await refreshPrivacy();
  } catch (e) {
    setErrorMessage(e);
    privacyFeedbackType.value = "error";
    privacyFeedback.value = tx(`移除失败：${e}`, `Remove failed: ${e}`);
  }
}

async function handleUpsertReminder(input: {
  id?: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  remind_at_text?: string;
  daily_time_text?: string;
  weekly_days?: number[];
  reminder_enabled: boolean;
}) {
  const content = input.content.trim();
  if (!content) {
    throw new Error(tx("提醒内容不能为空", "Reminder content cannot be empty"));
  }

  reminderActionLoading.value = true;
  try {
    if (input.repeat_rule === "DAILY" || input.repeat_rule === "WEEKLY") {
      let dailyMinutes: number | undefined;
      if (input.reminder_enabled) {
        const parsed = parseClockToMinutes(input.daily_time_text ?? "");
        if (parsed === null) {
          throw new Error(tx("每日时间格式错误，请使用 HH:MM", "Invalid daily time, use HH:MM"));
        }
        dailyMinutes = parsed;
      }

      let weeklyDays: number[] | undefined;
      if (input.repeat_rule === "WEEKLY") {
        weeklyDays = (input.weekly_days ?? [])
          .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
          .sort((a, b) => a - b);
        if (weeklyDays.length === 0) {
          throw new Error(tx("请选择每周重复的日期", "Please choose at least one weekday"));
        }
      }

      await saveReminder({
        id: input.id,
        content,
        repeat_rule: input.repeat_rule,
        daily_time_minutes: dailyMinutes,
        weekly_days: weeklyDays,
      });
    } else {
      let remindAt: number | undefined;
      if (input.reminder_enabled) {
        const parsed = parseDateTimeLocalToUnix(input.remind_at_text ?? "");
        if (parsed === null) {
          throw new Error(tx("请选择有效提醒时间", "Please choose a valid reminder time"));
        }
        remindAt = parsed;
      }

      await saveReminder({
        id: input.id,
        content,
        repeat_rule: "NONE",
        remind_at: remindAt,
      });
    }
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    throw e;
  } finally {
    reminderActionLoading.value = false;
  }
}

async function handleDeleteReminder(id: number) {
  if (reminderActionLoading.value) {
    return;
  }
  reminderActionLoading.value = true;
  try {
    await deleteReminder(id);
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    throw e;
  } finally {
    reminderActionLoading.value = false;
  }
}

async function handleReminderDone(id: number, done: boolean) {
  if (reminderActionLoading.value) {
    return;
  }
  reminderActionLoading.value = true;
  try {
    await setReminderDone({ id, done });
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    throw e;
  } finally {
    reminderActionLoading.value = false;
  }
}

async function handleReminderReorder(orderedIds: number[]) {
  if (reminderActionLoading.value || orderedIds.length === 0) {
    return;
  }

  const orderedSet = new Set(orderedIds);
  if (orderedSet.size !== orderedIds.length) {
    throw new Error(tx("排序数据无效", "Invalid reminder ordering"));
  }

  const previous = reminders.value.map((item) => ({ ...item }));
  const sortedCurrent = sortedReminders(reminders.value);
  const untouched = sortedCurrent.filter((item) => !orderedSet.has(item.id));
  const orderedItems = orderedIds
    .map((id) => reminders.value.find((item) => item.id === id))
    .filter((item): item is Reminder => Boolean(item));
  const nextItems = [...orderedItems, ...untouched].map((item, index) => ({
    ...item,
    sort_order: index,
  }));

  reminders.value = nextItems;
  reminderActionLoading.value = true;
  try {
    await setReminderOrder({ ordered_ids: nextItems.map((item) => item.id) });
    await refreshData();
  } catch (e) {
    reminders.value = previous;
    setErrorMessage(e);
    throw e;
  } finally {
    reminderActionLoading.value = false;
  }
}

async function handleReminderSnooze(id: number, seconds = 600) {
  if (reminderActionLoading.value) {
    return;
  }
  reminderActionLoading.value = true;
  try {
    await snoozeReminder(id, seconds);
    await refreshData();
  } catch (e) {
    setErrorMessage(e);
    throw e;
  } finally {
    reminderActionLoading.value = false;
  }
}

let pollTimer: number | null = null;
let captureTimer: number | null = null;
let navigateSectionUnlisten: UnlistenFn | null = null;
let sectionFlashTimer: number | null = null;
let heatmapGoalSaveTimer: number | null = null;
let settingsWarmTimer: number | null = null;

function schedulePersistHeatmapGoal() {
  if (heatmapGoalSaveTimer !== null) {
    window.clearTimeout(heatmapGoalSaveTimer);
    heatmapGoalSaveTimer = null;
  }

  heatmapGoalSaveTimer = window.setTimeout(async () => {
    try {
      await setHeatmapGoalSecondsSetting(getHeatmapGoalSeconds());
    } catch (e) {
      setErrorMessage(e);
    } finally {
      heatmapGoalSaveTimer = null;
    }
  }, 260);
}

function onGoalSliderInput(event: Event) {
  const input = event.target as HTMLInputElement;
  learnGoalSliderMinutes.value = Number.parseInt(input.value, 10) || 0;
  syncGoalFromSlider();
}

function onUsageIgnoreToggle(event: Event) {
  const input = event.target as HTMLInputElement;
  usageShowIgnore.value = input.checked;
}

function setAllTimeFilter(next: "ALL" | "LEARN" | "REST") {
  allTimeFilter.value = next;
  void refreshData();
}

function onAllTimeIgnoreToggle(event: Event) {
  const input = event.target as HTMLInputElement;
  allTimeIncludeIgnore.value = input.checked;
  void refreshData();
}

function onAutoCaptureToggle(event: Event) {
  const input = event.target as HTMLInputElement;
  autoCaptureEnabled.value = input.checked;
}

function onRuleSearchInput(event: Event) {
  const input = event.target as HTMLInputElement;
  ruleSearch.value = input.value;
}

function onRuleSortChange(event: Event) {
  const input = event.target as HTMLSelectElement;
  if (input.value === "alpha_asc" || input.value === "alpha_desc" || input.value === "time_desc" || input.value === "time_asc") {
    ruleSort.value = input.value;
  }
}

function onRuleMappedTypeChange(rule: AppRule, event: Event) {
  const input = event.target as HTMLSelectElement;
  if (input.value === "LEARN" || input.value === "REST" || input.value === "IGNORE") {
    rule.mapped_type = input.value;
  }
}

const recentTimelineGroups = computed(() => {
  const groups = new Map<string, RecentLog[]>();
  for (const item of recentLogs.value) {
    const day = logDayKey(item.start_timestamp);
    const list = groups.get(day) ?? [];
    list.push(item);
    groups.set(day, list);
  }
  return Array.from(groups.entries()).map(([day, items]) => ({ day, items }));
});

function recentDurationWidth(durationMs: number): string {
  const max = recentLogs.value[0]?.duration_ms ?? 0;
  if (max <= 0 || durationMs <= 0) {
    return "0%";
  }
  return `${Math.max(8, Math.min(100, (durationMs / max) * 100)).toFixed(2)}%`;
}

function flashInsightsSection(target: HTMLElement | null) {
  if (!target) {
    return;
  }
  document.getElementById("insights-stack-anchor")?.classList.remove("section-flash");
  target.classList.add("section-flash");

  if (sectionFlashTimer !== null) {
    window.clearTimeout(sectionFlashTimer);
    sectionFlashTimer = null;
  }

  sectionFlashTimer = window.setTimeout(() => {
    target.classList.remove("section-flash");
    sectionFlashTimer = null;
  }, 1000);
}

async function scrollToInsightsSection(section: string) {
  currentMainView.value = "insights";
  void section;
  switchInsightsSubView("history");
  await nextTick();
  const target = document.getElementById("insights-stack-anchor");
  flashInsightsSection(target);
}

const homeCtx = computed(() => ({
  tx,
  formatSeconds,
  todayLearnSeconds: todayLearnSeconds.value,
  todayRestSeconds: todayRestSeconds.value,
  currentStatusLabel: currentStatusLabel.value,
  currentStatusTone: currentStatusTone.value,
  goalProgressPct: goalProgressPct.value,
  goalProgressFillNum: goalProgressFillNum.value,
  goalOverflowTier: goalOverflowTier.value,
  recentSummary: recentSummary.value,
  homeScheduleItems: homeScheduleItems.value,
  reminderListForPanel: reminderListForPanel.value,
  reminderDueText,
  toDateTimeLocalValue,
  timeMinutesLabel,
  handleUpsertReminder,
  handleDeleteReminder,
  handleReminderDone,
  handleReminderReorder,
  handleReminderSnooze,
  reminderActionLoading: reminderActionLoading.value,
  shiftHeatmapMonth,
  monthTitleText: monthTitleText.value,
  weekHeaders: weekHeaders.value,
  calendarHeatmapCells: calendarHeatmapCells.value,
  heatmapCellClass,
  heatmapDayText,
  openGuardWorkflow,
  pendingRuleCount: pendingRuleProcesses.value.length,
  idlePromptCount: idlePrompts.value.length,
  dueReminderCount: dueReminderCount.value,
  homeMonthRhythmBars: homeMonthRhythmBars.value,
  homeMonthGoalProgress: homeMonthGoalProgress.value,
  homeMonthActiveStreakDays: homeMonthActiveStreakDays.value,
  homePendingSummary: homePendingSummary.value,
}));

const insightsCtx = computed(() => ({
  tx,
  insightsPrimaryView: insightsPrimaryView.value,
  switchInsightsSubView,
  learnGoalSliderMinutes: learnGoalSliderMinutes.value,
  onGoalSliderInput,
  refreshData,
  shiftHeatmapMonth,
  monthTitleText: monthTitleText.value,
  weekHeaders: weekHeaders.value,
  calendarHeatmapCells: calendarHeatmapCells.value,
  heatmapCellClass,
  heatmapDayText,
  formatSeconds,
  usageRootFilter: usageRootFilter.value,
  backToUsageOverview,
  usageFilterLabel: usageFilterLabel.value,
  selectedUsageDayData: selectedUsageDayData.value,
  switchUsageFilter,
  usageShowIgnore: usageShowIgnore.value,
  onUsageIgnoreToggle,
  usageStack: usageStack.value,
  selectUsageDay,
  dayRenderSegments,
  dayRenderTotal,
  usageSegmentWidth,
  handleSegmentClick,
  handleSegmentMouseEnter,
  handleSegmentMouseMove,
  handleSegmentMouseLeave,
  shouldShowSegmentLabel,
  topSegmentSummary,
  cleanProcessName,
  usageSegmentColor,
  processDrillCandidates: processDrillCandidates.value,
  selectedUsageProcess: selectedUsageProcess.value,
  selectUsageProcess,
  linkedRecentLogs: linkedRecentLogs.value,
  formatClock,
  stackTooltip: stackTooltip.value,
  historySubViews: historySubViews.value,
  historySubView: historySubView.value,
  switchHistorySubView,
  topApps: topApps.value,
  topAppsBarWidth,
  allTimeTopApps: allTimeTopApps.value,
  setAllTimeFilter,
  allTimeIncludeIgnore: allTimeIncludeIgnore.value,
  onAllTimeIgnoreToggle,
  allTimeBarWidth,
  recentTimelineGroups: recentTimelineGroups.value,
  recentDurationWidth,
}));

const guardCtx = computed(() => ({
  tx,
  guardCurrentStepText: guardCurrentStepText.value,
  guardStepLabels: guardStepLabels.value,
  guardCurrentStepIndex: guardCurrentStepIndex.value,
  autoCaptureEnabled: autoCaptureEnabled.value,
  onAutoCaptureToggle,
  pendingRuleProcesses: pendingRuleProcesses.value,
  cleanProcessName,
  formatClock,
  formatSeconds,
  handleSavePendingRule,
  guardStep2Unlocked: guardStep2Unlocked.value,
  idlePrompts: idlePrompts.value,
  formatIdlePromptSpan,
  idleActionLoading: idleActionLoading.value,
  handleResolveIdle,
  guardStep3Unlocked: guardStep3Unlocked.value,
  ruleSearch: ruleSearch.value,
  onRuleSearchInput,
  ruleSort: ruleSort.value,
  onRuleSortChange,
  filteredSortedRules: filteredSortedRules.value,
  onRuleMappedTypeChange,
  handleUpdateExistingRule,
  markGuardStep3Done,
  guardStep3Done: guardStep3Done.value,
  guardStep4Unlocked: guardStep4Unlocked.value,
  foregroundDiagnostics: foregroundDiagnostics.value,
  captureBlockReasonText,
  captureRuleText,
  canSaveRuleFromDiagnostic,
  handleSaveRuleFromDiagnostic,
}));

onMounted(async () => {
  initLocale();
  try {
    initThemeMode();
  } catch {
    applyTheme("light");
  }

  try {
    const savedGoalSeconds = await getHeatmapGoalSecondsSetting();
    const minutes = Math.round(savedGoalSeconds / 60 / 15) * 15;
    learnGoalSliderMinutes.value = Math.min(1440, Math.max(0, minutes));
  } catch (e) {
    setErrorMessage(e);
  }

  autoCaptureFeedback.value = tx("自动采样已开启", "Auto capture enabled");
  guardFeedback.value = tx("尚未执行检测", "No check executed yet");
  privacyFeedback.value = tx("未保存隐私设置", "Privacy settings not saved");
  void refreshHomeData();
  settingsWarmTimer = window.setTimeout(() => {
    privacyViewMounted.value = true;
    void refreshSettingsData();
  }, 1200);
  pollTimer = window.setInterval(() => {
    void refreshHomeData();
    if (currentMainView.value === "guard") {
      void refreshGuardData();
    }
  }, 5000);

  captureTimer = window.setInterval(() => {
    if (!autoCaptureEnabled.value) {
      return;
    }
    void captureForegroundOnce(5000)
      .then((stored) => {
        autoCaptureFeedback.value = stored
          ? tx("自动采样运行中（最近一条已入库）。", "Auto capture running (latest sample stored).")
          : tx("自动采样运行中（最近一条未入库：隐私拦截或基线样本）。", "Auto capture running (latest sample not stored: privacy block or baseline sample).");
      })
      .catch((e) => {
        autoCaptureFeedback.value = tx(`自动采样失败：${e}`, `Auto capture failed: ${e}`);
      });
  }, 5000);

  navigateSectionUnlisten = await listen<string>("navigate-insights-section", (event) => {
    void scrollToInsightsSection(event.payload || "heatmap");
  });
});

watch(locale, (next, prev) => {
  if (next === prev) {
    return;
  }
  applyLocale(next);
});

watch(guardStep3Unlocked, (unlocked) => {
  if (!unlocked) {
    guardStep3Done.value = false;
  }
});

onUnmounted(() => {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }
  if (captureTimer !== null) {
    window.clearInterval(captureTimer);
    captureTimer = null;
  }
  if (settingsWarmTimer !== null) {
    window.clearTimeout(settingsWarmTimer);
    settingsWarmTimer = null;
  }
  if (navigateSectionUnlisten) {
    navigateSectionUnlisten();
    navigateSectionUnlisten = null;
  }
  if (sectionFlashTimer !== null) {
    window.clearTimeout(sectionFlashTimer);
    sectionFlashTimer = null;
  }
  if (heatmapGoalSaveTimer !== null) {
    window.clearTimeout(heatmapGoalSaveTimer);
    heatmapGoalSaveTimer = null;
  }
});
</script>

<template>
  <main class="layout">
    <section class="card top-shell fixed-top-nav">
      <div class="top-brand" :aria-label="tx('品牌区', 'Brand section')">
        <div class="brand-logo" aria-hidden="true">TP</div>
        <div class="brand-copy">
          <strong>Time Prism</strong>
          <span>{{ tx("Focus OS", "Focus OS") }}</span>
        </div>
      </div>

      <div class="top-nav" role="tablist" :aria-label="tx('主页面导航', 'Main Navigation')">
        <button
          v-for="view in mainViews"
          :key="view.key"
          type="button"
          class="top-nav-btn"
          :class="{ active: currentMainView === view.key }"
          @click="setMainView(view.key)"
        >
          {{ view.label }}
        </button>
      </div>
    </section>

    <div class="content-scroll">
      <section v-if="currentIdlePrompt" class="card idle-inline" :aria-label="tx('离开时段待确认', 'Idle Segment Confirmation')">
        <h2>{{ tx("离开时段待确认", "Idle Segment Confirmation") }}</h2>
        <p class="hint">{{ tx("系统空闲超过5分钟后触发，已回补完整时段。", "Triggered after 5+ minutes idle; full segment has been backfilled.") }}</p>
        <p class="idle-span">{{ formatIdlePromptSpan(currentIdlePrompt) }}</p>
        <div class="toggle-row" style="margin-top: 8px;">
          <input id="idle-remember-choice" type="checkbox" v-model="idleRememberChoice" />
          <label for="idle-remember-choice">{{ tx("记住本次选择（后续离开时段自动应用）", "Remember this choice for later idle segments") }}</label>
        </div>
        <div class="diag-actions" style="margin-top: 8px;">
          <button type="button" :disabled="idleActionLoading" @click="handleResolveIdle('LEARN')">{{ tx("标记为学习", "Mark as Learn") }}</button>
          <button type="button" :disabled="idleActionLoading" @click="handleResolveIdle('REST')">{{ tx("标记为休息", "Mark as Break") }}</button>
          <button type="button" :disabled="idleActionLoading" @click="handleResolveIdle('IDLE')">{{ tx("标记为离开", "Mark as Away") }}</button>
          <button type="button" :disabled="idleActionLoading" @click="handleResolveIdle('SKIP')">{{ tx("稍后提醒", "Remind me later") }}</button>
        </div>
      </section>

      <HomeView v-if="currentMainView === 'home'" :ctx="homeCtx" />
      <InsightsView v-if="currentMainView === 'insights'" :ctx="insightsCtx" />
      <GuardView v-if="currentMainView === 'guard'" :ctx="guardCtx" />

      <section v-if="privacyViewMounted" v-show="currentMainView === 'privacy'" class="card privacy-card">
        <h2>{{ tx("设置", "Settings") }}</h2>
        <div class="privacy-scroll-shell">
          <div class="privacy-grid">
          <article class="guard-panel">
            <h3>{{ tx("界面与显示", "Interface & Display") }}</h3>
            <div class="row" style="margin-top: 4px;">
              <label for="language-select">{{ tx("界面语言", "Language") }}</label>
              <select id="language-select" v-model="locale">
                <option value="zh-CN">中文</option>
                <option value="en-US">English</option>
              </select>
            </div>
            <div class="row theme-toggle-row" style="margin-top: 2px;">
              <label>{{ tx("深色模式", "Dark mode") }}</label>
              <button type="button" class="compact-btn" @click="toggleThemeMode">
                {{ themeMode === 'dark' ? tx('切换浅色', 'Switch to Light') : tx('切换深色', 'Switch to Dark') }}
              </button>
            </div>
            <div class="toggle-row">
              <input id="auto-start" type="checkbox" v-model="autoStartEnabled" />
              <label for="auto-start">{{ tx("开机自启", "Launch at startup") }}</label>
            </div>
          </article>

          <article class="guard-panel">
            <h3>{{ tx("采样策略", "Sampling Policy") }}</h3>
            <div class="toggle-row">
              <input id="curtain" type="checkbox" v-model="privacy.curtain_enabled" />
              <label for="curtain">{{ tx("拉窗帘模式（停止记录所有应用日志）", "Curtain mode (stop recording all app logs)") }}</label>
            </div>

            <div class="row" style="margin-top: 8px;">
              <label for="browser-title-mode">{{ tx("浏览器标题策略", "Browser title policy") }}</label>
              <select id="browser-title-mode" v-model="privacy.browser_title_mode">
                <option value="FULL">{{ tx("FULL - 记录完整标题", "FULL - Keep full title") }}</option>
                <option value="BLUR">{{ tx("BLUR - 模糊显示为 Web Browser", "BLUR - Mask as Web Browser") }}</option>
                <option value="NONE">{{ tx("NONE - 不采集标题", "NONE - No title capture") }}</option>
              </select>
              <p class="hint">{{ tx("建议默认 BLUR；如需最高隐私请选择 NONE。", "BLUR is recommended; choose NONE for maximum privacy.") }}</p>
            </div>

            <div class="toggle-row">
              <input id="whitelist-only" type="checkbox" v-model="privacy.whitelist_only_enabled" />
              <label for="whitelist-only">{{ tx("仅记录白名单进程", "Record only whitelisted apps") }}</label>
            </div>

            <div class="actions" style="margin-top: 8px;">
              <button @click="handleSavePrivacySettings">{{ tx("保存设置", "Save settings") }}</button>
            </div>
            <p class="guard-feedback" :class="privacyFeedbackType">{{ privacyFeedback }}</p>
          </article>

          <article class="guard-panel">
            <h3>{{ tx("白名单管理", "Whitelist") }}</h3>
            <div class="row" style="margin-top: 4px;">
              <label for="whitelist-input">{{ tx("添加白名单进程", "Add whitelist process") }}</label>
              <input id="whitelist-input" v-model="whitelistInput" placeholder="e.g. pycharm64.exe" />
            </div>
            <div class="actions">
              <button @click="handleAddWhitelist">{{ tx("添加白名单", "Add") }}</button>
            </div>

            <ul class="whitelist-list whitelist-scroll">
              <li v-for="proc in whitelist" :key="proc">
                <span>{{ proc }}</span>
                <button @click="handleRemoveWhitelist(proc)">{{ tx("移除", "Remove") }}</button>
              </li>
              <li v-if="whitelist.length === 0" class="muted">{{ tx("白名单为空", "Whitelist is empty") }}</li>
            </ul>
          </article>
          </div>
        </div>
      </section>

      <p v-if="error" class="error">{{ error }}</p>
    </div>
  </main>
</template>

<style>
body[data-theme="dark"] {
  --bg-a: #1a1b24;
  --bg-b: #12131b;
  --bg-c: #0d0e15;
  --text-main: #dfe5f0;
  --text-soft: #a7b1c5;
  --text-faint: #7e889e;
  --card-bg: rgba(27, 29, 42, 0.82);
  --card-edge: rgba(151, 161, 198, 0.2);
  --panel-bg: rgba(21, 23, 34, 0.74);
  --panel-edge: rgba(145, 156, 194, 0.24);
  --button-bg: #2c3656;
  --button-edge: #4a5c8d;
  --button-text: #e8edf9;
  --button-hover: #36446d;
  --accent-soft: #90a2d1;
  --accent-main: #8ab8d6;
  --ok: #8dda9f;
  --warn: #ffd485;
  --danger: #ff9898;
  --line-dash: rgba(171, 180, 201, 0.28);
  --track-bg: rgba(10, 12, 20, 0.76);
  --shadow-soft: 0 14px 30px rgba(3, 6, 20, 0.4);
  --inner-top: inset 0 1px 0 rgba(255, 255, 255, 0.08);
}

body,
body[data-theme="light"] {
  --bg-a: #f4f1ea;
  --bg-b: #efe8dc;
  --bg-c: #e6dbcb;
  --text-main: #3c342a;
  --text-soft: #6b5f52;
  --text-faint: #92857a;
  --card-bg: rgba(255, 252, 246, 0.84);
  --card-edge: rgba(176, 155, 130, 0.46);
  --panel-bg: rgba(254, 249, 240, 0.78);
  --panel-edge: rgba(187, 165, 140, 0.42);
  --button-bg: #cdc1af;
  --button-edge: #a89274;
  --button-text: #2f261d;
  --button-hover: #d9cebc;
  --accent-soft: #7d6f5d;
  --accent-main: #5478aa;
  --ok: #3f8f58;
  --warn: #ad7135;
  --danger: #b44f4f;
  --line-dash: rgba(146, 133, 122, 0.34);
  --track-bg: rgba(191, 180, 164, 0.34);
  --shadow-soft: 0 18px 32px rgba(101, 82, 60, 0.16);
  --inner-top: inset 0 1px 0 rgba(255, 255, 255, 0.48);
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  min-height: 100vh;
  overflow-y: hidden;
  overflow-x: hidden;
  color: var(--text-main);
  background:
    radial-gradient(1200px 560px at -10% -20%, rgba(255, 255, 255, 0.22), transparent 68%),
    radial-gradient(900px 540px at 120% 0%, rgba(142, 170, 218, 0.18), transparent 70%),
    linear-gradient(140deg, var(--bg-a) 0%, var(--bg-b) 46%, var(--bg-c) 100%);
  font-family: "Microsoft YaHei UI", "Segoe UI", "PingFang SC", sans-serif;
  transition: background 320ms ease, color 260ms ease;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
}

.layout {
  width: 860px;
  max-width: 860px;
  margin: 0 auto;
  height: 100%;
  max-height: 100vh;
  min-height: 0;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow: hidden;
}

.fixed-top-nav {
  position: relative;
  flex: 0 0 auto;
  z-index: 40;
}

.top-shell {
  display: grid;
  grid-template-columns: minmax(180px, 1fr) minmax(0, 2.2fr);
  gap: 10px;
  align-items: center;
  min-height: 84px;
  padding: 14px 18px;
  border-radius: 24px;
  background:
    linear-gradient(180deg, #fffdfc 0%, #f4eee5 100%);
  border-color: #dfd2c2;
}

.top-brand {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
}

.brand-logo {
  width: 44px;
  height: 44px;
  border-radius: 14px;
  display: grid;
  place-items: center;
  font-family: "Inter", "Segoe UI", sans-serif;
  font-size: 16px;
  font-weight: 800;
  letter-spacing: 0.5px;
  color: #f2fbff;
  background: linear-gradient(145deg, #2ab6a8 0%, #3b74d2 100%);
  box-shadow: 0 8px 18px rgba(41, 126, 173, 0.24);
}

.brand-copy {
  min-width: 0;
  display: grid;
  gap: 1px;
}

.brand-copy strong {
  line-height: 1;
  font-family: Arial, "Microsoft YaHei UI", sans-serif;
  font-size: 22px;
  font-weight: 800;
  color: #2f2a24;
}

.brand-copy span {
  font-family: "MiSans-Regular", "Microsoft YaHei UI", "Segoe UI", sans-serif;
  font-size: 12px;
  color: #7a7066;
}

.top-signal {
  min-width: 0;
  display: grid;
  gap: 4px;
  justify-self: start;
  border: 1px solid var(--panel-edge);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg) 88%, transparent);
  padding: 6px 8px;
}

.signal-line {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  font-size: 11px;
}

.signal-duration-line {
  white-space: nowrap;
}

.signal-duration-key {
  font-size: 11px;
  color: var(--text-soft);
  font-weight: 700;
}

.signal-duration-key.break-key {
  margin-left: 10px;
}

.signal-duration-val {
  font-size: 11px;
  font-variant-numeric: tabular-nums;
}

.signal-status-line .signal-strong {
  font-size: 12px;
}

.signal-label {
  flex: 0 0 auto;
  color: var(--text-soft);
  font-size: 11px;
}

.signal-strong,
.signal-text {
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 999px;
  background: #6a7280;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.35);
}

.status-dot.ok {
  background: #39b96a;
}

.status-dot.warn {
  background: #c98a3d;
}

.status-dot.alert {
  background: #d35b5b;
}

.top-nav {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 6px;
  width: 560px;
  padding: 4px;
  border-radius: 999px;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.8), rgba(242, 236, 227, 0.62)),
    color-mix(in srgb, var(--panel-bg) 92%, transparent);
  border: 1px solid color-mix(in srgb, var(--panel-edge) 48%, transparent);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.82),
    0 10px 24px rgba(115, 92, 63, 0.08);
}

.top-nav-btn {
  min-height: 48px;
  padding: 14px 18px;
  font-family: "MiSans-Regular", "Microsoft YaHei UI", "Segoe UI", sans-serif;
  font-size: 14px;
  font-weight: 700;
  color: #6c6b68;
  border-radius: 20px;
  border: 1px solid transparent;
  background: transparent;
  box-shadow: none;
  transition:
    background 180ms ease,
    border-color 180ms ease,
    box-shadow 180ms ease,
    color 180ms ease,
    transform 180ms ease;
}

.top-nav-btn.active {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.98), rgba(244, 240, 234, 0.96));
  color: #1f1a14;
  border-color: transparent;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.95),
    0 8px 18px rgba(29, 24, 20, 0.1),
    0 0 0 1px rgba(255, 255, 255, 0.55);
  transform: none;
}

.content-scroll {
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-right: 0;
}

.content-scroll > .insight-pane,
.content-scroll > .guard-card,
.content-scroll > .privacy-card {
  flex: 1 1 auto;
  min-height: 0;
  max-height: 100%;
}

.card {
  border: 1px solid var(--card-edge);
  border-radius: 28px;
  background: var(--card-bg);
  backdrop-filter: blur(8px);
  box-shadow: var(--inner-top), var(--shadow-soft);
  padding: 14px;
}

.theme-toggle-row {
  grid-template-columns: auto auto;
  align-items: center;
  justify-content: start;
  gap: 10px;
}

.theme-toggle-row .compact-btn {
  width: auto;
  min-width: 108px;
  padding: 8px 12px;
}

.overview-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}

.overview-grid {
  margin-top: 4px;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.overview-card-hero {
  padding: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.overview-grid-hero {
  grid-template-columns: 270px 269px minmax(0, 273px);
  gap: 10px;
  min-height: 300px;
  align-items: stretch;
  justify-content: space-between;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #6c5a47;
  letter-spacing: 0;
}

.overview-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 28px;
}

.overview-card-head-plain {
  justify-content: flex-start;
}

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  min-height: 26px;
  padding: 8px 12px;
  border-radius: 999px;
  border: 1px solid #dccebe;
  background: #fffdf9;
  font-family: "MiSans-Regular", "Microsoft YaHei UI", "Segoe UI", sans-serif;
  font-size: 12px;
  font-weight: 600;
  color: #54a66d;
}

.status-chip-ok {
  color: #2f8d5c;
}

.status-chip-warn {
  color: #a7712b;
}

.status-chip-alert {
  color: #b14d4d;
}

.status-chip-idle {
  color: #6a7280;
}

.overview-summary-card,
.overview-schedule-hero,
.overview-goal-hero {
  display: grid;
  align-content: start;
  gap: 8px;
  height: 290px;
  min-height: 290px;
  grid-template-rows: auto 1fr auto;
  border-radius: 22px;
  padding: 16px;
  background: #fbf7ef;
  border: 1px solid #d9c9b5;
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.78),
    0 10px 28px rgba(201, 181, 150, 0.13);
}

.overview-summary-card {
  width: 270px;
}

.overview-goal-hero {
  width: 269px;
}

.overview-schedule-hero {
  width: 273px;
}

.overview-summary-panel {
  display: grid;
  gap: 8px;
  padding: 0;
  border-radius: 0;
  background: transparent;
  border: none;
}

.overview-kicker {
  display: block;
  font-size: 12px;
  font-weight: 400;
  letter-spacing: 0;
  color: #8a7864;
}

.overview-status-main {
  margin: 0;
  font-size: 22px;
  line-height: 1.12;
  letter-spacing: -0.03em;
  color: #352e27;
  word-break: break-word;
}

.status-recent-panel {
  margin-top: 2px;
  padding-top: 0;
  border-top: none;
}

.overview-signal-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.overview-signal-card {
  display: grid;
  gap: 8px;
  min-height: 62px;
  padding: 8px 10px;
  border-radius: 18px;
  border: 1px solid #e8dcce;
  background: #f9f3e8;
}

.overview-signal-card strong {
  font-size: 16px;
  line-height: 1;
  font-weight: 800;
  color: #3c342a;
  font-variant-numeric: tabular-nums;
}

.overview-card-footnote {
  margin: auto 0 0;
  padding-top: 0;
  border-top: none;
  font-size: 12px;
  line-height: 1.45;
  color: #7e6f61;
}

.status-recent {
  margin-top: 4px;
}

.status-recent-title {
  display: block;
  font-size: 12px;
  color: #8a7864;
}

.status-recent-value {
  margin: 2px 0 0;
  font-size: 13px;
  line-height: 1.35;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: #6a5a4c;
}

.dual-metrics {
  margin-top: 6px;
  display: grid;
  grid-template-columns: 1fr;
  gap: 6px;
}

.metric-mini {
  display: block;
  font-size: 11px;
  color: var(--text-soft);
}

.home-focus-grid {
  display: grid;
  grid-template-columns: 350px 470px;
  gap: 10px;
  align-items: stretch;
  overflow: hidden;
}

.home-module-card {
  border: 1px solid #e1d4c5;
  background: #faf4ea;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  border-radius: 18px;
  box-shadow: none;
}

.home-module-header {
  min-height: 24px;
  display: flex;
  align-items: center;
}

.home-module-header h3 {
  margin: 0;
  font-size: 14px;
  line-height: 1.25;
  font-weight: 700;
  font-family: "Segoe UI", "Microsoft YaHei UI", sans-serif;
  color: #4a3b2e;
}

.calendar-nav-row {
  display: flex;
  justify-content: center;
  align-items: center;
}

.home-calendar-card {
  gap: 10px;
  height: 430px;
  min-height: 430px;
}

.home-rhythm-card {
  gap: 8px;
  height: 430px;
  min-height: 430px;
  overflow: hidden;
}

.home-calendar-card .home-module-header h3 {
  font-size: 17px;
  color: #3b3027;
}

.rhythm-glance-card {
  border: 1px solid #e4d7c7;
  border-radius: 14px;
  background: #f7efe3;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
  box-shadow: none;
  min-height: 348px;
  min-width: 0;
}

.rhythm-glance-card-bottom {
  margin-top: 2px;
  flex: 1 1 auto;
  min-height: 0;
  overflow: hidden;
}

.rhythm-glance-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.rhythm-section-title {
  font-size: 11px;
  font-weight: 600;
  color: #7d6d5f;
}

.rhythm-mini-tag {
  font-size: 10px;
  color: #8da0b5;
}

.rhythm-summary-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}

.rhythm-summary-chip {
  border-radius: 12px;
  border: 1px solid #e6d9c8;
  background: rgba(255, 251, 245, 0.82);
  padding: 8px 10px;
  display: grid;
  gap: 4px;
}

.rhythm-summary-label {
  font-size: 11px;
  color: #7b6958;
}

.rhythm-summary-chip strong {
  font-size: 15px;
  line-height: 1.1;
  color: #2f5c50;
}

.rhythm-summary-note {
  margin: 0;
  font-size: 12px;
  line-height: 1.35;
  color: #66584c;
  font-weight: 500;
}

.rhythm-bars {
  height: 100%;
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  gap: 8px;
  align-items: stretch;
  overflow: hidden;
  padding-top: 8px;
}

.rhythm-bar-col {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: 22px minmax(0, 1fr) 22px;
  gap: 7px;
  padding: 0;
  justify-items: center;
}

.rhythm-bar-value {
  display: block;
  text-align: center;
  font-size: 14px;
  line-height: 1.1;
  color: #6d5f51;
  font-variant-numeric: tabular-nums;
  min-height: 16px;
  font-weight: 600;
}

.rhythm-bar-value.today {
  color: #365f72;
  font-weight: 700;
}

.rhythm-bar-track {
  min-height: 0;
  display: flex;
  align-items: end;
  justify-content: center;
  width: 100%;
  padding: 6px 0 2px;
  border-bottom: 1px solid rgba(125, 109, 95, 0.16);
  overflow: visible;
}

.rhythm-bar-stack {
  width: 58%;
  min-width: 18px;
  max-width: 28px;
  display: flex;
  flex-direction: column-reverse;
  justify-content: flex-start;
  gap: 0;
  min-height: 0;
  border-radius: 999px;
  overflow: hidden;
  background: rgba(218, 208, 193, 0.16);
  box-shadow: inset 0 0 0 1px rgba(168, 146, 116, 0.1);
}

.rhythm-bar-stack.today {
  box-shadow:
    inset 0 0 0 1px rgba(84, 120, 170, 0.22),
    0 0 0 1px rgba(84, 120, 170, 0.08);
}

.rhythm-bar {
  width: 100%;
  min-height: 12px;
  border-radius: 0;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.16);
  transition: filter 140ms ease, opacity 140ms ease;
  cursor: pointer;
}

.rhythm-bar:hover {
  filter: brightness(0.98) saturate(1.08);
}

.rhythm-bar-learn {
  background: linear-gradient(180deg, #85bc93 0%, #71a884 100%);
}

.rhythm-bar-rest {
  background: linear-gradient(180deg, #c9d7ee 0%, #b3c4e3 100%);
}

.rhythm-bar-label {
  display: block;
  text-align: center;
  font-size: 13px;
  line-height: 1.1;
  color: #7a6a5b;
  font-weight: 500;
}

.rhythm-bar-label.today {
  color: #48697a;
  font-weight: 700;
}

.rhythm-hover-tooltip {
  position: fixed;
  z-index: 5000;
  min-width: 220px;
  max-width: 260px;
  border-radius: 18px;
  border: 1px solid rgba(122, 97, 74, 0.18);
  background:
    radial-gradient(circle at top left, rgba(255, 255, 255, 0.92), rgba(247, 240, 229, 0.96));
  box-shadow:
    0 20px 34px rgba(91, 68, 47, 0.14),
    inset 0 1px 0 rgba(255, 255, 255, 0.72);
  padding: 14px 16px;
  pointer-events: none;
  backdrop-filter: blur(10px);
}

.rhythm-hover-kicker {
  display: block;
  margin: 0 0 6px;
  font-size: 11px;
  line-height: 1.2;
  letter-spacing: 0.02em;
  color: #8a7763;
}

.rhythm-hover-tooltip strong {
  display: block;
  margin: 0 0 8px;
  font-size: 22px;
  line-height: 1.02;
  color: #254c43;
  letter-spacing: -0.03em;
}

.rhythm-hover-tooltip p {
  margin: 0;
  font-size: 12px;
  line-height: 1.4;
  color: #6f6154;
}

.rhythm-hover-tooltip p + p {
  margin-top: 4px;
}

.rhythm-hover-tooltip span {
  color: #245247;
  font-weight: 700;
}

.rhythm-hover-share {
  font-size: 13px;
}

.rhythm-hover-tone {
  font-size: 13px;
  line-height: 1.45;
  color: #46392e;
}

.rhythm-hover-context {
  font-size: 11px;
  color: #877768;
}

.rhythm-metric-list {
  display: grid;
  gap: 8px;
  flex: 0 0 auto;
}

.rhythm-metric-item {
  border: 1px solid #e4d7c7;
  border-radius: 14px;
  background: #fbf7ef;
  height: 48px;
  min-height: 48px;
  padding: 8px 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  box-shadow: none;
}

.rhythm-metric-copy {
  display: grid;
  gap: 2px;
}

.rhythm-metric-label {
  font-size: 11px;
  color: #8a7864;
}

.rhythm-metric-copy strong {
  font-size: 15px;
  line-height: 1.1;
  color: #2f5f4d;
}

.rhythm-metric-note {
  font-size: 11px;
  font-weight: 600;
  color: #7e6f61;
  white-space: nowrap;
}

.month-nav-wrap {
  display: grid;
  grid-template-columns: 25px auto 25px;
  align-items: center;
  justify-items: center;
  column-gap: 14px;
}

.month-nav-wrap strong {
  min-width: 34px;
  font-size: 17px;
  font-family: "Inter", "Segoe UI", sans-serif;
  font-weight: 700;
  line-height: 1;
  text-align: center;
  color: #4a3b2e;
}

.goal-dial-hint-inline {
  margin: 0 0 4px;
  font-size: 11px;
}

.goal-dial-hint {
  margin-top: 0;
  text-align: center;
  font-size: 11px;
}

.top-apps-compact h4 {
  margin: 2px 0 6px;
  font-size: 12px;
}

.top-apps-compact ul {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 6px;
}

.top-app-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  font-size: 12px;
}

.top-app-track {
  margin-top: 4px;
  width: 100%;
  height: 8px;
  border-radius: 999px;
  border: 1px solid var(--panel-edge);
  background: var(--track-bg);
  overflow: hidden;
}

.top-app-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #51b565, #8bc7aa);
}

.week-header.compact-week-header {
  width: 322px;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  justify-content: start;
  gap: 8px;
  margin-bottom: 0;
  font-size: 11px;
}

.week-header.compact-week-header span {
  text-align: center;
  color: #8a7864;
}

.heatmap-grid.compact-heatmap-grid {
  gap: 6px;
  width: 322px;
  grid-template-columns: repeat(7, minmax(0, 1fr));
  justify-content: start;
  margin-top: 0;
}

.heatmap-grid.compact-heatmap-grid .heat-cell {
  width: 100%;
  height: 41px;
  aspect-ratio: auto;
  border-radius: 10px;
  border: none;
}

.heatmap-grid.compact-heatmap-grid .heat-cell.today-cell {
  position: relative;
  background: #faf4ea;
  box-shadow: inset 0 0 0 2px #55a07a;
}

.heatmap-grid.compact-heatmap-grid .heat-cell.today-cell .day-label {
  font-weight: 700;
  color: #3f5e50;
}

.day-label.compact-day-label {
  font-size: 14px;
}

.home-goal-panel {
  border: 1px solid var(--panel-edge);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg) 72%, transparent);
  padding: 10px;
}

.overview-metric,
.overview-status,
.overview-goal,
.trend-card {
  border: 1px solid var(--panel-edge);
  border-radius: 12px;
  background: var(--panel-bg);
  padding: 8px;
  box-shadow: var(--inner-top);
}

.metric-label {
  display: block;
  font-size: 12px;
  color: var(--text-soft);
}

.metric-value {
  display: block;
  margin-top: 4px;
  font-size: 22px;
  line-height: 1.1;
}

.overview-goal-pie {
  margin-top: 0;
  width: 140px;
  height: 140px;
  border-radius: 50%;
  position: relative;
  display: grid;
  place-items: center;
  background: conic-gradient(from 90deg, #56b68f 0deg, #56b68f var(--goal-angle), #ddeae2 var(--goal-angle), #ddeae2 360deg);
  border: none;
  box-shadow: none;
}

.overview-goal-pie::before {
  content: "";
  position: absolute;
  inset: 21px;
  border-radius: 50%;
  background: #fbf7ef;
  box-shadow: none;
}

.overview-goal-pie::after {
  display: none;
}

.overview-goal-pie.overflow-active {
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.8),
    0 16px 28px rgba(53, 96, 77, 0.12),
    0 0 0 7px rgba(79, 201, 152, 0.1);
  animation: overflowPulseGentle 2.2s ease-in-out infinite;
}

.overview-goal-pie strong {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1;
  font-family: "MiSans-Regular", "Microsoft YaHei UI", "Segoe UI", sans-serif;
  font-size: 26px;
  letter-spacing: -0.03em;
  color: #314e41;
  text-shadow: none;
}

.overview-goal {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: stretch;
  text-align: center;
}

.overview-goal .metric-label {
  margin-bottom: 6px;
}

.overview-goal-main {
  display: grid;
  grid-template-columns: 52px minmax(0, 1fr);
  align-items: center;
  gap: 18px;
}

.overview-goal-main-hero {
  grid-template-columns: 62px minmax(0, 1fr);
  gap: 20px;
  align-items: center;
}

.overview-goal-main-compact {
  grid-template-columns: 1fr;
  gap: 12px;
}

.overview-goal-visual {
  display: grid;
  grid-template-columns: 1fr;
  align-items: center;
  justify-items: center;
  gap: 12px;
  min-width: 0;
}

.overview-goal-ring-wrap {
  width: 100%;
  display: grid;
  place-items: center;
  padding: 0;
}

.overview-goal-readout {
  display: grid;
  justify-items: center;
  gap: 0;
  width: 100%;
}

.overview-goal-readout-value {
  font-family: "Segoe UI", "Microsoft YaHei UI", sans-serif;
  font-size: 23px;
  line-height: 1;
  letter-spacing: -0.04em;
  font-weight: 700;
  color: #3c3028;
  font-variant-numeric: tabular-nums;
}

.overview-goal-readout-sub {
  font-size: 10px;
  color: #7e6f61;
  text-align: center;
  line-height: 1.2;
}

.overview-schedule {
  display: grid;
  gap: 10px;
  align-content: stretch;
  position: relative;
  min-height: 0;
}

.schedule-headline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.schedule-icon-btn {
  min-height: 0;
  width: 34px;
  height: 34px;
  padding: 0;
  border-radius: 12px;
  display: grid;
  place-items: center;
  font-size: 13px;
  background: #e4d7c6;
  border: 1px solid #b39c80;
  box-shadow: none;
}

.schedule-line-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 8px;
}

.schedule-list-wrap {
  min-height: 0;
  flex: 1 1 auto;
  max-height: 168px;
  overflow-y: auto;
  padding-right: 4px;
}

.schedule-line-list li {
  display: grid;
  grid-template-columns: 16px minmax(0, 1fr);
  align-items: start;
  gap: 10px;
  min-height: 0;
  padding: 0;
  border: none;
  border-bottom: 1px solid #e8dccc;
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.schedule-line-list li.done {
  opacity: 0.86;
}

.schedule-line-list li.dragging {
  opacity: 0.58;
}

.schedule-line-list li.drop-target {
  border-bottom-color: #6eab8b;
  box-shadow: inset 0 -2px 0 #6eab8b;
}

.schedule-line-list li:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.schedule-line-content {
  display: grid;
  gap: 3px;
  min-width: 0;
  align-content: start;
  padding-top: 2px;
}

.schedule-line-top {
  display: flex;
  align-items: start;
  justify-content: space-between;
  gap: 8px;
}

.schedule-line-tools {
  flex: 0 0 auto;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.schedule-line-tool {
  width: 22px;
  height: 22px;
  min-height: 22px;
  padding: 0;
  border-radius: 8px;
  display: grid;
  place-items: center;
  background: #f5ede1;
  border: 1px solid #dbcab4;
  color: #7e6f61;
  font-size: 12px;
  box-shadow: none;
}

.schedule-line-drag {
  cursor: grab;
}

.schedule-line-drag[draggable="true"] {
  touch-action: none;
}

.schedule-line-drag:active {
  cursor: grabbing;
}

.schedule-line-check {
  width: 16px;
  height: 16px;
  min-height: 16px;
  border-radius: 5px;
  border: 2px solid #b89f7f;
  background: #fff8ef;
  color: #8f775b;
  font-size: 12px;
  font-weight: 900;
  line-height: 1;
  display: inline-grid;
  place-items: center;
  padding: 0;
  align-self: start;
  justify-self: start;
  aspect-ratio: 1;
  box-shadow: none;
  margin-top: 4px;
}

.schedule-line-check.done {
  background: #e7f3ea;
  border-color: #73a885;
  color: #3f5e50;
  box-shadow: none;
}

.schedule-line-title {
  font-size: 14px;
  font-weight: 600;
  color: #3a3029;
  line-height: 1.32;
  white-space: normal;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
}

.schedule-line-title.done {
  color: #6e6c68;
  text-decoration: none;
}

.schedule-line-due {
  font-size: 12px;
  color: #7e6f61;
  line-height: 1.25;
  white-space: normal;
}

.schedule-empty {
  padding: 12px 0;
  border: 1px dashed color-mix(in srgb, var(--line-dash) 88%, transparent);
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.42);
}

.schedule-support {
  margin: auto 0 0;
  padding: 0;
  border-top: none;
  line-height: 1.45;
  font-size: 12px;
  color: #7e6f61;
}

.schedule-popover-mask {
  position: fixed;
  inset: 0;
  z-index: 4000;
  background: rgba(8, 11, 18, 0.38);
  display: grid;
  place-items: center;
  padding: 16px 10px;
  overflow-y: auto;
}

.schedule-popover {
  width: min(460px, calc(100vw - 16px));
  max-height: min(calc(100vh - 32px), 680px);
  overflow: auto;
  border-radius: 14px;
  border: 1px solid var(--panel-edge);
  background: var(--card-bg);
  padding: 8px 10px 10px;
  margin: auto;
}

.schedule-popover-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}

.schedule-popover-head h3 {
  margin: 0;
  font-size: 16px;
}

.schedule-close-btn {
  min-height: 0;
  width: 28px;
  height: 28px;
  padding: 0;
  border-radius: 999px;
  font-size: 15px;
}

.schedule-form-grid {
  margin-top: 2px;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px 8px;
}

.schedule-form-grid-compact {
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
}

.schedule-check {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.schedule-row-compact {
  gap: 5px;
}

.schedule-popover .row {
  gap: 6px;
  margin-bottom: 8px;
}

.schedule-popover label {
  font-size: 12px;
}

.schedule-popover input,
.schedule-popover select,
.schedule-popover button {
  min-height: 34px;
  padding: 8px 10px;
  border-radius: 10px;
}

.schedule-popover .actions {
  gap: 6px;
}

.schedule-actions {
  justify-content: flex-start;
}

.schedule-popover .actions button {
  min-height: 32px;
  padding: 7px 10px;
  font-size: 12px;
}

.schedule-popover .guard-feedback {
  margin: 8px 0 0;
  padding: 7px 9px;
  font-size: 12px;
}

.schedule-popover .reminder-scroll {
  max-height: 150px;
}

.schedule-popover .reminder-row-head strong {
  font-size: 12px;
}

.schedule-popover .reminder-actions {
  gap: 5px;
}

.schedule-popover .reminder-actions button {
  padding: 5px 8px;
  font-size: 11px;
}

.compact-reminder-list .reminder-row {
  padding: 6px 0;
}

.schedule-switch {
  display: inline-flex;
  align-items: center;
}

.schedule-switch input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.schedule-switch-ui {
  width: 34px;
  height: 20px;
  border-radius: 999px;
  border: 1px solid #9ba7af;
  background: #d0d6dc;
  position: relative;
  transition: background 180ms ease, border-color 180ms ease;
}

.schedule-switch-ui::before {
  content: "";
  position: absolute;
  top: 1px;
  left: 1px;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.24);
  transition: transform 180ms ease;
}

.schedule-switch input:checked + .schedule-switch-ui {
  background: #0078d4;
  border-color: #0078d4;
}

.schedule-switch input:checked + .schedule-switch-ui::before {
  transform: translateX(14px);
}

.schedule-weekdays-row {
  margin-top: 2px;
}

.weekday-chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.weekday-chip {
  min-height: 28px;
  padding: 4px 8px;
  border-radius: 999px;
  font-size: 11px;
  background: var(--panel-bg);
}

.weekday-chip.active {
  border-color: color-mix(in srgb, var(--accent-main) 70%, transparent);
  background: color-mix(in srgb, var(--accent-main) 20%, var(--panel-bg) 80%);
}

@keyframes overflowPulseGentle {
  0%,
  100% {
    filter: saturate(106%) brightness(100%);
  }
  50% {
    filter: saturate(118%) brightness(106%);
  }
}

.month-head {
  padding-right: 0;
}

.recent-inline {
  margin-top: 10px;
}

.action-entry {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  min-height: 83px;
  padding: 18px 22px;
  border-radius: 20px;
  background: #f4efe6;
  border-color: #d7c8b3;
  box-shadow: none;
}

.action-entry .actions {
  justify-content: flex-end;
}

.action-entry .actions button {
  min-width: 115px;
  min-height: 39px;
  padding: 10px 18px;
  border-radius: 16px;
}

.action-entry h3 {
  margin: 0 0 6px;
  font-family: "Inter", "Segoe UI", sans-serif;
  font-size: 13px;
  font-weight: 600;
  color: #5d4d3e;
}

.action-entry .hint {
  margin: 0;
  width: 420px;
  font-family: "Inter", "Segoe UI", sans-serif;
  font-size: 12px;
  line-height: 1.4;
  color: #7f6f60;
}

.text-btn {
  min-height: 0;
  padding: 4px 8px;
  border-radius: 999px;
  background: transparent;
}

.btn-primary {
  border-color: #6f8198;
  background: #d4d9e0;
  color: #4b596b;
}

.btn-secondary {
  background: var(--panel-bg);
  color: var(--text-main);
}

.guard-stepper {
  margin: 8px 0 10px;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 8px;
}

.guard-step {
  border: 1px solid var(--panel-edge);
  border-radius: 11px;
  background: var(--panel-bg);
  padding: 7px 8px;
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 6px;
  align-items: center;
}

.guard-step .step-index {
  width: 18px;
  height: 18px;
  border-radius: 999px;
  display: grid;
  place-items: center;
  font-size: 11px;
  border: 1px solid var(--panel-edge);
}

.guard-step .step-label {
  font-size: 12px;
  color: var(--text-soft);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.guard-step.done {
  border-color: color-mix(in srgb, var(--ok) 64%, var(--panel-edge) 36%);
}

.guard-step.done .step-index {
  background: color-mix(in srgb, var(--ok) 30%, transparent);
  border-color: color-mix(in srgb, var(--ok) 66%, transparent);
}

.guard-step.active {
  border-color: color-mix(in srgb, var(--accent-main) 66%, transparent);
  background: color-mix(in srgb, var(--accent-main) 16%, var(--panel-bg) 84%);
}

.guard-step.active .step-index {
  border-color: color-mix(in srgb, var(--accent-main) 72%, transparent);
  background: color-mix(in srgb, var(--accent-main) 24%, transparent);
}

.guard-step.locked {
  opacity: 0.72;
}

.guard-panel.locked {
  opacity: 0.62;
}

.lock-note {
  margin: 6px 0 0;
  font-size: 12px;
  color: var(--warn);
}

.pet {
  display: flex;
  align-items: center;
  gap: 14px;
}

.pet-avatar {
  width: 56px;
  height: 56px;
  border-radius: 15px;
  display: grid;
  place-items: center;
  font-weight: 800;
  letter-spacing: 0.5px;
  color: #2b2620;
  background: linear-gradient(140deg, #f5e7c8 0%, #dcb98b 100%);
  box-shadow: inset 0 1px 2px rgba(255, 255, 255, 0.48), 0 8px 18px rgba(69, 47, 22, 0.22);
}

.pet-avatar.doubt {
  color: #2f1b1b;
  background: linear-gradient(145deg, #f3c5ac 0%, #d68d71 100%);
}

.pet-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.theme-toggle {
  border-radius: 999px;
  padding: 6px 12px;
  min-height: 0;
  font-size: 12px;
}

.insight-subnav {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  margin-bottom: 12px;
}

.history-subnav {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
  margin: 8px 0 12px;
}

.insight-subnav-btn {
  min-height: 34px;
  padding: 6px 8px;
  font-size: 12px;
  background: var(--panel-bg);
}

.insight-subnav-btn.active {
  background: linear-gradient(140deg, rgba(120, 160, 210, 0.34), rgba(86, 120, 176, 0.34));
  border-color: color-mix(in srgb, var(--accent-main) 66%, transparent);
  color: var(--text-main);
}

.insight-pane {
  animation: insightPaneIn 160ms ease;
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.insight-pane-body {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}

.insight-section {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding-right: 4px;
}

h1,
h2,
h3,
h4 {
  margin: 0 0 10px;
  color: var(--text-main);
}

.hint,
.label,
.muted,
.stack-day,
.usage-breadcrumb {
  color: var(--text-soft);
}

.hint {
  margin: 4px 0 0;
  font-size: 13px;
}

.row {
  display: grid;
  gap: 8px;
  margin-bottom: 10px;
}

.toggle-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 8px 0;
}

.toggle-row input[type="checkbox"] {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-main);
}

.privacy-card .toggle-row input[type="checkbox"] {
  width: 30px;
  height: 18px;
  appearance: none;
  -webkit-appearance: none;
  border-radius: 999px;
  border: 1px solid #9ba7af;
  background: #d0d6dc;
  position: relative;
  cursor: pointer;
  padding: 0;
  min-height: 0;
  transition: background 180ms ease, border-color 180ms ease;
}

.privacy-card .toggle-row input[type="checkbox"]::before {
  content: "";
  position: absolute;
  top: 1px;
  left: 1px;
  width: 14px;
  height: 14px;
  border-radius: 999px;
  background: #ffffff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.24);
  transition: transform 180ms ease;
}

.privacy-card .toggle-row input[type="checkbox"]:checked {
  background: #0078d4;
  border-color: #0078d4;
}

.privacy-card .toggle-row input[type="checkbox"]:checked::before {
  transform: translateX(12px);
}

label {
  font-size: 13px;
}

select,
input,
button {
  border: 1px solid var(--button-edge);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg) 80%, transparent);
  color: var(--text-main);
  padding: 10px 12px;
  min-height: 38px;
  transition: background 180ms ease, border-color 180ms ease, transform 180ms ease;
}

button {
  cursor: pointer;
  color: var(--button-text);
  background: linear-gradient(160deg, var(--button-bg), color-mix(in srgb, var(--button-bg) 82%, #ffffff 18%));
}

button:hover:not(:disabled) {
  background: linear-gradient(160deg, var(--button-hover), var(--button-bg));
  transform: translateY(-1px);
}

button:disabled {
  opacity: 0.54;
  cursor: not-allowed;
}

.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.state {
  margin: 10px 0 0;
  color: var(--ok);
}

.metrics ul,
.alltime-list,
.recent-list,
.pending-list,
.rule-list,
.drill-list,
.whitelist-list {
  list-style: none;
  margin: 8px 0 0;
  padding: 0;
}

.metrics li,
.recent-list li,
.pending-list li,
.rule-list li,
.whitelist-list li {
  border-bottom: 1px dashed var(--line-dash);
  padding: 8px 0;
}

.alltime-list {
  display: grid;
  gap: 8px;
  min-height: 0;
}

.alltime-list li,
.drill-panel,
.guard-panel {
  border: 1px solid var(--panel-edge);
  border-radius: 12px;
  background: var(--panel-bg);
}

.alltime-list li,
.drill-panel,
.guard-panel,
.idle-modal {
  box-shadow: var(--inner-top);
}

.alltime-list li,
.guard-panel,
.drill-panel {
  padding: 10px;
}

.alltime-head,
.recent-main,
.recent-sub,
.rule-main,
.rule-actions,
.stack-meta {
  display: flex;
  justify-content: space-between;
  gap: 10px;
}

.recent-sub {
  margin-top: 4px;
}

.alltime-track,
.stack-bar,
.drill-track {
  margin-top: 6px;
  width: 100%;
  border-radius: 999px;
  background: var(--track-bg);
  border: 1px solid var(--panel-edge);
  overflow: hidden;
}

.alltime-track,
.drill-track {
  height: 10px;
}

.alltime-fill {
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, #96b7ef, #8bc7aa);
}

.recent-scroll,
.panel-scroll,
.rule-scroll,
.whitelist-scroll {
  overflow-y: auto;
  padding-right: 4px;
}

.insight-pane .alltime-list,
.insight-pane .recent-scroll {
  flex: 1 1 auto;
  min-height: 0;
  max-height: none;
  overflow-y: auto;
}

.recent-scroll {
  max-height: 280px;
}

.recent-timeline {
  padding-right: 2px;
}

.timeline-group + .timeline-group {
  margin-top: 10px;
}

.timeline-day {
  margin: 0 0 8px;
  font-size: 12px;
  color: var(--text-soft);
}

.timeline-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: grid;
  gap: 10px;
}

.timeline-item {
  display: grid;
  grid-template-columns: 74px 14px 1fr;
  gap: 8px;
  align-items: start;
}

.timeline-time {
  font-size: 12px;
  color: var(--text-soft);
  text-align: right;
  padding-top: 3px;
}

.timeline-dot {
  width: 10px;
  height: 10px;
  border-radius: 999px;
  margin-top: 6px;
  background: var(--accent-main);
  box-shadow: 0 0 0 3px color-mix(in srgb, var(--accent-main) 22%, transparent);
}

.timeline-card {
  border: 1px solid var(--panel-edge);
  border-radius: 12px;
  background: color-mix(in srgb, var(--panel-bg) 82%, transparent);
  padding: 8px 10px;
}

.panel-scroll {
  max-height: 260px;
}

.rule-scroll,
.whitelist-scroll {
  max-height: 320px;
}

.reminder-scroll {
  max-height: 360px;
}

.reminder-draft-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}

.reminder-row {
  display: grid;
  gap: 8px;
}

.reminder-row-head {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  align-items: center;
}

.reminder-row-head strong {
  font-size: 13px;
}

.reminder-meta {
  display: flex;
  gap: 8px;
  font-size: 12px;
  color: var(--text-soft);
}

.reminder-meta .done-tag {
  color: var(--ok);
}

.reminder-actions {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.reminder-actions button {
  min-height: 0;
  padding: 6px 10px;
  font-size: 12px;
}

.guard-grid,
.privacy-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  margin-top: 10px;
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  padding-right: 4px;
  align-content: start;
}

.guard-card,
.privacy-card {
  display: flex;
  flex-direction: column;
  min-height: 0;
  height: 100%;
  overflow: hidden;
}

.guard-card-body,
.privacy-scroll-shell {
  flex: 1 1 auto;
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  padding-right: 0;
}

.guard-panel-wide {
  grid-column: 1 / -1;
}

.rule-toolbar {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 8px;
}

.rule-toolbar select {
  min-width: 170px;
}

.rule-actions {
  margin-top: 6px;
  align-items: center;
}

.rule-actions select {
  min-width: 120px;
}

.diag-rule-tag {
  display: inline-flex;
  align-items: center;
  border: 1px solid color-mix(in srgb, var(--accent-main) 65%, transparent);
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 12px;
  color: var(--accent-main);
  background: color-mix(in srgb, var(--accent-main) 24%, transparent);
}

.diag-rule-tag.unsaved {
  color: var(--warn);
  border-color: color-mix(in srgb, var(--warn) 68%, transparent);
  background: color-mix(in srgb, var(--warn) 24%, transparent);
}

.diag-actions {
  margin-top: 6px;
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.diag-actions button {
  padding: 6px 10px;
  border-radius: 10px;
  min-height: 0;
  font-size: 12px;
}

.goal-slider {
  appearance: none;
  width: 100%;
  height: 10px;
  border-radius: 999px;
  border: 1px solid var(--panel-edge);
  background: linear-gradient(to right, #7cae98 var(--goal-pct), var(--track-bg) var(--goal-pct));
  padding: 0;
}

.goal-slider::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fffdf8;
  border: 2px solid #6d8dbd;
  cursor: pointer;
}

.goal-slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: #fffdf8;
  border: 2px solid #6d8dbd;
  cursor: pointer;
}

.month-head {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 6px;
}

.month-head strong {
  min-width: 64px;
  font-size: 18px;
  text-align: center;
}

.month-nav {
  width: 22px;
  height: 22px;
  min-height: 0;
  border-radius: 999px;
  padding: 0;
  display: grid;
  place-items: center;
  font-size: 13px;
}

.week-header {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 6px;
  margin-bottom: 6px;
  color: var(--text-faint);
  font-size: 12px;
}

.week-header span {
  text-align: center;
}

.goal-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.goal-item {
  display: grid;
  gap: 6px;
}

.heatmap-grid {
  margin-top: 10px;
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 6px;
}

.heat-cell {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 10px;
  border: 1px solid transparent;
  display: grid;
  place-items: center;
}

.day-label {
  font-size: 14px;
  line-height: 1;
  color: #5f5043;
  font-weight: 600;
}

.heat-cell.past-date {
  opacity: 1;
}

.heat-cell.future-date {
  opacity: 1;
  border-style: solid;
}

.heat-cell.gray {
  background: #efe8dc;
  border-color: #d5c8ba;
}

.heat-cell.yellow {
  background: #e2b063;
  border-color: transparent;
}

.heat-cell.green {
  background: #7eb187;
  border-color: transparent;
}

.heat-cell.green.green-1 {
  background: #7eb187;
}

.heat-cell.green.green-2 {
  background: #78ad82;
}

.heat-cell.green.green-3 {
  background: #72a87b;
}

.heat-cell.green.green-4 {
  background: #6eaa7a;
}

.heat-cell.future-date.gray {
  background: #f7f2e8;
  border-color: transparent;
}

.heat-cell.future-date .day-label {
  color: #c4b7a7;
  font-weight: 600;
}

.heat-cell.pad-cell {
  visibility: visible;
  background: #faf4ea;
  border-color: transparent;
}

.heat-cell.pad-cell .day-label {
  opacity: 0;
}

.legend {
  margin-top: 8px;
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 12px;
  color: var(--text-soft);
}

.legend span {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.legend .heat-cell {
  width: 12px;
  height: 12px;
  aspect-ratio: auto;
}

.stack-list {
  display: grid;
  gap: 8px;
  margin-top: 8px;
}

.usage-breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 6px 0 10px;
  font-size: 12px;
}

.breadcrumb-link {
  border: none;
  background: transparent;
  color: var(--accent-main);
  padding: 0;
  min-height: 0;
  text-decoration: underline;
}

.stack-row {
  display: grid;
  grid-template-columns: 52px 1fr;
  gap: 8px;
  align-items: center;
  border-radius: 11px;
  border: 1px solid transparent;
  padding: 4px 6px;
  cursor: pointer;
}

.stack-row:hover {
  border-color: var(--panel-edge);
  background: color-mix(in srgb, var(--panel-bg) 84%, transparent);
}

.stack-row.active {
  border-color: color-mix(in srgb, var(--accent-main) 65%, transparent);
  background: color-mix(in srgb, var(--accent-main) 20%, var(--panel-bg) 80%);
}

.stack-main {
  display: grid;
  gap: 4px;
}

.stack-bar {
  min-height: 24px;
  border-radius: 8px;
  display: flex;
}

.stack-seg {
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.stack-label {
  font-size: 11px;
  line-height: 1;
  color: #f8fafc;
  font-weight: 700;
  white-space: nowrap;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.35);
}

.stack-empty {
  width: 100%;
  display: grid;
  place-items: center;
  font-size: 11px;
}

.stack-meta {
  font-size: 12px;
  color: var(--text-soft);
}

.stack-tooltip {
  position: fixed;
  z-index: 2147483000;
  max-width: 260px;
  border: 1px solid;
  border-radius: 12px;
  padding: 10px 12px;
  color: #fff;
  box-shadow: 0 12px 24px rgba(2, 6, 23, 0.38);
  pointer-events: none;
}

.stack-tooltip p {
  margin: 0;
  font-size: 12px;
  line-height: 1.45;
  white-space: normal;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.22);
}

.stack-tooltip p + p {
  margin-top: 4px;
}

.stack-tooltip p.header,
.stack-tooltip p.strong {
  font-weight: 700;
}

.drill-panel {
  margin-top: 10px;
}

.drill-panel h4 {
  margin: 0 0 8px;
  font-size: 13px;
}

.drill-list {
  display: grid;
  gap: 8px;
}

.drill-list li {
  display: grid;
  grid-template-columns: 120px 1fr 70px;
  gap: 8px;
  align-items: center;
}

.drill-name,
.drill-time {
  font-size: 12px;
  color: var(--text-main);
}

.drill-fill {
  height: 100%;
}

.process-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.chip {
  border-radius: 999px;
  min-height: 0;
  padding: 6px 10px;
  color: var(--text-main);
}

.chip.active {
  border-color: color-mix(in srgb, var(--accent-main) 70%, transparent);
  background: color-mix(in srgb, var(--accent-main) 24%, var(--panel-bg) 76%);
}

.whitelist-list li {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.idle-span {
  margin: 8px 0 0;
  color: var(--warn);
  font-weight: 600;
}

.idle-inline {
  border-color: color-mix(in srgb, var(--warn) 62%, transparent);
  background: color-mix(in srgb, var(--card-bg) 90%, transparent);
}

.label {
  font-size: 12px;
}

.error {
  margin: 0;
  color: var(--danger);
}

.guard-feedback {
  margin: 10px 0 0;
  border: 1px solid var(--panel-edge);
  border-radius: 10px;
  padding: 8px 10px;
}

.guard-feedback.info {
  color: color-mix(in srgb, var(--accent-main) 72%, var(--text-main) 28%);
  background: color-mix(in srgb, var(--accent-main) 18%, transparent);
}

.guard-feedback.ok {
  color: var(--ok);
  background: color-mix(in srgb, var(--ok) 15%, transparent);
}

.guard-feedback.warn {
  color: var(--warn);
  background: color-mix(in srgb, var(--warn) 16%, transparent);
}

.guard-feedback.error {
  color: var(--danger);
  background: color-mix(in srgb, var(--danger) 14%, transparent);
}

.section-flash {
  animation: sectionFlashGlow 1000ms ease;
}

@keyframes sectionFlashGlow {
  0% {
    color: var(--text-main);
    text-shadow: none;
  }

  35% {
    color: var(--accent-main);
    text-shadow: 0 0 10px color-mix(in srgb, var(--accent-main) 58%, transparent);
  }

  100% {
    color: var(--text-main);
    text-shadow: none;
  }
}

@keyframes insightPaneIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (max-width: 760px) {
  .layout {
    padding: 10px 10px 10px;
  }

  .overview-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .overview-grid-hero,
  .home-focus-grid {
    grid-template-columns: 1fr;
  }

  .overview-goal-main {
    grid-template-columns: auto 1fr;
    gap: 10px;
  }

  .overview-goal-visual {
    grid-template-columns: 1fr;
    justify-items: center;
  }

  .rhythm-summary-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .overview-signal-grid {
    grid-template-columns: 1fr;
  }

  .overview-card-head,
  .overview-summary-topline {
    align-items: flex-start;
    flex-direction: column;
  }

  .home-focus-grid {
    gap: 10px;
  }

  .action-entry {
    flex-direction: column;
    align-items: flex-start;
  }

  .action-entry .actions {
    width: 100%;
    justify-content: flex-start;
  }

  .action-entry .actions button {
    min-width: 0;
    width: 100%;
  }

  .guard-stepper {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .pet {
    align-items: flex-start;
  }

  .pet-head {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .top-shell {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .top-nav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .top-signal {
    order: 3;
  }

  .top-nav {
    order: 2;
  }

  .insight-subnav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .history-subnav {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .timeline-item {
    grid-template-columns: 60px 10px 1fr;
  }

  .guard-grid,
  .privacy-grid,
  .reminder-draft-grid,
  .schedule-form-grid,
  .rule-toolbar,
  .drill-list li {
    grid-template-columns: 1fr;
  }

  .guard-panel-wide {
    grid-column: auto;
  }
}
</style>


