import { computed, ref, type Ref } from "vue";
import type {
  IdlePrompt,
  PendingRuleProcess,
  RecentLog,
  Reminder,
  TodaySummary,
} from "../api";
import { buildHomeGoalProgress } from "../lib/homeOverviewProgress";
import {
  homeCurrentStatusLabel,
  homeCurrentStatusTone,
  homePendingSummaryText,
  homeRecentSummaryText,
  type HomeOverviewStatusCounts,
  type HomeStatusTone,
} from "../lib/homeOverviewStatus";
import { countDueReminders } from "../lib/reminderSchedule";

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
  const goalProgress = computed(() =>
    buildHomeGoalProgress(todayLearnSeconds.value, learnGoalSliderMinutes.value));
  const goalProgressPct = computed(() => goalProgress.value.percentText);
  const goalProgressFillNum = computed(() => goalProgress.value.fillPercent);
  const goalOverflowTier = computed(() => goalProgress.value.overflowTier);

  const recentSummary = computed(() =>
    homeRecentSummaryText(recentLogs.value[0], cleanProcessName, formatClock, tx));

  const dueReminderCount = computed(() => {
    const now = Math.floor(Date.now() / 1000);
    return countDueReminders(reminders.value, now);
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
