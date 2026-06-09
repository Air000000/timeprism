import type {
  AppRule,
  ForegroundCaptureDiagnostic,
  IdlePrompt,
  PendingRuleProcess,
  RecentLog,
  TopApp,
} from "../api";
import type { HistorySubViewKey } from "../composables/useAppNavigation";

export type TranslateFn = (zh: string, en: string) => string;
export type AllTimeFilter = "ALL" | "LEARN" | "REST";
export type RuleMappedType = "LEARN" | "REST" | "IGNORE";
export type IdleDecision = "LEARN" | "REST" | "IDLE" | "SKIP";
export type RuleSortKey = "alpha_asc" | "alpha_desc" | "time_desc" | "time_asc";

export type HistorySubViewOption = {
  key: HistorySubViewKey;
  label: string;
};

export type RecentTimelineGroup = {
  day: string;
  items: RecentLog[];
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
