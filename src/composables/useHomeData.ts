import { ref, type Ref } from "vue";
import {
  getLearnHeatmap,
  getTodaySummary,
  getUsageStack,
  listPendingIdlePrompts,
  listPendingRuleProcesses,
  listRecentLogs,
  listReminders,
  type IdlePrompt,
  type LearnHeatmapCell,
  type PendingRuleProcess,
  type RecentLog,
  type Reminder,
  type TodaySummary,
  type UsageStackDay,
} from "../api";

type UseHomeDataOptions = {
  todaySummary: Ref<TodaySummary>;
  recentLogs: Ref<RecentLog[]>;
  learnHeatmap: Ref<LearnHeatmapCell[]>;
  pendingRuleProcesses: Ref<PendingRuleProcess[]>;
  idlePrompts: Ref<IdlePrompt[]>;
  reminders: Ref<Reminder[]>;
  homeUsageStack: Ref<UsageStackDay[]>;
  getHeatmapFetchDays: () => number;
  getHeatmapGoalSeconds: () => number;
  setErrorMessage: (error: unknown) => void;
};

export function useHomeData({
  todaySummary,
  recentLogs,
  learnHeatmap,
  pendingRuleProcesses,
  idlePrompts,
  reminders,
  homeUsageStack,
  getHeatmapFetchDays,
  getHeatmapGoalSeconds,
  setErrorMessage,
}: UseHomeDataOptions) {
  const loadingHome = ref(false);

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

  return {
    refreshHomeData,
  };
}
