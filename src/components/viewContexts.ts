import type {
  AppRule,
  AppRuleMappedType,
  ForegroundCaptureDiagnostic,
  IdlePrompt,
  LearnHeatmapCell,
  PendingRuleProcess,
  PrivacySettings,
  RecentLog,
  Reminder,
  TopApp,
  UsageRootFilter,
} from "../api";
import type { HistorySubViewKey } from "../composables/useAppNavigation";
import type { LocaleCode } from "../composables/useLocale";
import type { ReminderUpsertInput } from "../composables/useReminders";
import type { ThemeMode } from "../composables/useThemeMode";

export type TranslateFn = (zh: string, en: string) => string;
export type AllTimeFilter = UsageRootFilter;
export type RuleMappedType = AppRuleMappedType;
export type IdleDecision = "APP" | "LEARN" | "REST" | "IDLE" | "SKIP";
export type RuleSortKey = "alpha_asc" | "alpha_desc" | "time_desc" | "time_asc";
export type FeedbackTone = "info" | "ok" | "warn" | "error";

export type HistorySubViewOption = {
  key: HistorySubViewKey;
  label: string;
};

export type RecentTimelineGroup = {
  day: string;
  items: RecentLog[];
};

export type HomeRhythmBar = {
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

export type HomeViewContext = {
  tx: TranslateFn;
  formatSeconds: (totalSeconds: number) => string;
  todayLearnSeconds: number;
  currentStatusLabel: string;
  currentStatusTone: "ok" | "warn" | "alert" | "idle";
  goalProgressPct: string;
  goalProgressFillNum: number;
  goalOverflowTier: "none" | "active";
  recentSummary: string;
  homeScheduleItems: Reminder[];
  reminderListForPanel: Reminder[];
  reminderDueText: (item: Reminder) => string;
  toDateTimeLocalValue: (unixSeconds: number) => string;
  timeMinutesLabel: (minutes: number) => string;
  handleUpsertReminder: (input: ReminderUpsertInput) => Promise<void> | void;
  handleDeleteReminder: (id: number) => Promise<void> | void;
  handleReminderDone: (id: number, done: boolean) => Promise<void> | void;
  handleReminderReorder: (orderedIds: number[]) => Promise<void> | void;
  handleReminderSnooze: (id: number, seconds?: number) => Promise<void> | void;
  reminderActionLoading: boolean;
  shiftHeatmapMonth: (delta: number) => void;
  monthTitleText: string;
  weekHeaders: string[];
  calendarHeatmapCells: Array<LearnHeatmapCell | null>;
  heatmapCellClass: (cell: LearnHeatmapCell) => string[];
  heatmapDayText: (dayKey: string) => string;
  pendingRuleCount: number;
  idlePromptCount: number;
  dueReminderCount: number;
  homeMonthRhythmBars: HomeRhythmBar[];
};

export type SettingsViewContext = {
  tx: TranslateFn;
  locale: LocaleCode;
  onLocaleChange: (event: Event) => void;
  themeMode: ThemeMode;
  toggleThemeMode: () => void;
  autoStartEnabled: boolean;
  onAutoStartChange: (event: Event) => void;
  privacy: PrivacySettings;
  handleSavePrivacySettings: () => Promise<void> | void;
  privacyFeedbackType: FeedbackTone;
  privacyFeedback: string;
  whitelistInput: string;
  onWhitelistInput: (event: Event) => void;
  handleAddWhitelist: () => Promise<void> | void;
  whitelist: string[];
  handleRemoveWhitelist: (processName: string) => Promise<void> | void;
};

export type IdlePromptBannerContext = {
  tx: TranslateFn;
  currentIdlePrompt: IdlePrompt;
  formatIdlePromptSpan: (item: IdlePrompt) => string;
  cleanProcessName: (name: string) => string;
  idleRememberChoice: boolean;
  onIdleRememberChoiceChange: (event: Event) => void;
  idleActionLoading: boolean;
  handleResolveIdle: (decision: IdleDecision, promptId?: number) => Promise<void> | void;
};

export type InsightsViewContext = {
  tx: TranslateFn;
  historySubViews: HistorySubViewOption[];
  historySubView: HistorySubViewKey;
  switchHistorySubView: (next: HistorySubViewKey) => Promise<void> | void;
  topApps: TopApp[];
  cleanProcessName: (name: string) => string;
  formatSeconds: (totalSeconds: number) => string;
  topAppsBarWidth: (seconds: number) => string;
  setAllTimeFilter: (next: AllTimeFilter) => void;
  allTimeIncludeIgnore: boolean;
  onAllTimeIgnoreToggle: (event: Event) => void;
  allTimeTopApps: TopApp[];
  allTimeBarWidth: (seconds: number) => string;
  recentTimelineGroups: RecentTimelineGroup[];
  formatClock: (unixSeconds: number) => string;
  recentDurationWidth: (durationMs: number) => string;
};

export type GuardViewContext = {
  tx: TranslateFn;
  guardCurrentStepText: string;
  guardStepLabels: string[];
  guardCurrentStepIndex: number;
  autoCaptureEnabled: boolean;
  onAutoCaptureToggle: (event: Event) => void;
  pendingRuleProcesses: PendingRuleProcess[];
  cleanProcessName: (name: string) => string;
  formatClock: (unixSeconds: number) => string;
  formatSeconds: (totalSeconds: number) => string;
  handleSavePendingRule: (item: PendingRuleProcess, mappedType: RuleMappedType) => Promise<void> | void;
  guardStep2Unlocked: boolean;
  idlePrompts: IdlePrompt[];
  formatIdlePromptSpan: (item: IdlePrompt) => string;
  idleActionLoading: boolean;
  handleResolveIdle: (decision: IdleDecision, promptId?: number) => Promise<void> | void;
  guardStep3Unlocked: boolean;
  ruleSearch: string;
  onRuleSearchInput: (event: Event) => void;
  ruleSort: RuleSortKey;
  onRuleSortChange: (event: Event) => void;
  filteredSortedRules: AppRule[];
  onRuleMappedTypeChange: (rule: AppRule, event: Event) => void;
  handleUpdateExistingRule: (rule: AppRule) => Promise<void> | void;
  markGuardStep3Done: () => void;
  guardStep3Done: boolean;
  guardStep4Unlocked: boolean;
  foregroundDiagnostics: ForegroundCaptureDiagnostic[];
  captureBlockReasonText: (reason: string | null) => string;
  captureRuleText: (item: ForegroundCaptureDiagnostic) => string;
  canSaveRuleFromDiagnostic: (item: ForegroundCaptureDiagnostic) => boolean;
  handleSaveRuleFromDiagnostic: (
    item: ForegroundCaptureDiagnostic,
    mappedType: RuleMappedType,
  ) => Promise<void> | void;
};
