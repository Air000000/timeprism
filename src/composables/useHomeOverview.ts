import { computed, ref, type Ref } from "vue";
import type {
  IdlePrompt,
  PendingRuleProcess,
  RecentLog,
  Reminder,
  TodaySummary,
} from "../api";
import {
  homeCurrentStatusLabel,
  homeCurrentStatusTone,
  homePendingSummaryText,
  type HomeOverviewStatusCounts,
  type HomeStatusTone,
} from "../lib/homeOverviewStatus";

type TranslateFn = (zh: string, en: string) => string;

type UseHomeOverviewOptions = {
  tx: TranslateFn;
  learnGoalSliderMinutes: Ref<number>;
  recentLogs: Ref<RecentLog[]>;
  reminders: Ref<Reminder[]>;
  pendingRuleProcesses: Ref<PendingRuleProcess[]>;
  idlePrompts: Ref<IdlePrompt[]>;
  autoCaptureEnabled: Ref<boolean>;
  cleanProcessName: (name: string) => string;
  formatClock: (unixSeconds: number) => string;
};

export function useHomeOverview({
  tx,
  learnGoalSliderMinutes,
  recentLogs,
  reminders,
  pendingRuleProcesses,
  idlePrompts,
  autoCaptureEnabled,
  cleanProcessName,
  formatClock,
}: UseHomeOverviewOptions) {
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

  const dueReminderCount = computed(() => {
    const now = Math.floor(Date.now() / 1000);
    return reminders.value.filter((item) => !item.done && item.next_due_timestamp <= now).length;
  });

  function currentStatusCounts(): HomeOverviewStatusCounts {
    return {
      idlePromptCount: idlePrompts.value.length,
      dueReminderCount: dueReminderCount.value,
      pendingRuleProcessCount: pendingRuleProcesses.value.length,
      autoCaptureEnabled: autoCaptureEnabled.value,
    };
  }

  const currentStatusLabel = computed(() => homeCurrentStatusLabel(currentStatusCounts(), tx));
  const currentStatusTone = computed<HomeStatusTone>(() => homeCurrentStatusTone(currentStatusCounts()));
  const homePendingSummary = computed(() => homePendingSummaryText(currentStatusCounts(), tx));

  return {
    todaySummary,
    todayLearnSeconds,
    todayRestSeconds,
    goalProgressPct,
    goalProgressFillNum,
    goalOverflowTier,
    recentSummary,
    dueReminderCount,
    currentStatusLabel,
    currentStatusTone,
    homePendingSummary,
  };
}
