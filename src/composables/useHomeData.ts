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
  const loadingHomeCore = ref(false);
  const loadingHomeAnalytics = ref(false);
  let initialHomeRefresh = true;

  async function refreshHomeCoreData() {
    if (loadingHomeCore.value) {
      return;
    }

    loadingHomeCore.value = true;
    try {
      const [summary, logs, pendingRules, pendingIdle, reminderRows] = await Promise.all([
        getTodaySummary(),
        listRecentLogs(12),
        listPendingRuleProcesses(10),
        listPendingIdlePrompts(3),
        listReminders(120, true),
      ]);
      todaySummary.value = summary;
      recentLogs.value = logs;
      pendingRuleProcesses.value = pendingRules;
      idlePrompts.value = pendingIdle;
      reminders.value = reminderRows;
    } catch (e) {
      setErrorMessage(e);
    } finally {
      loadingHomeCore.value = false;
    }
  }

  async function refreshHomeAnalyticsData() {
    if (loadingHomeAnalytics.value) {
      return;
    }

    loadingHomeAnalytics.value = true;
    try {
      const heatmap = await getLearnHeatmap(getHeatmapFetchDays(), getHeatmapGoalSeconds());
      learnHeatmap.value = heatmap;

      const rhythmStack = await getUsageStack(7, "ALL");
      homeUsageStack.value = rhythmStack;
    } catch (e) {
      setErrorMessage(e);
    } finally {
      loadingHomeAnalytics.value = false;
    }
  }

  function scheduleInitialHomeAnalyticsRefresh() {
    const run = () => {
      void refreshHomeAnalyticsData();
    };

    if (typeof window.requestIdleCallback === "function") {
      window.requestIdleCallback(run, { timeout: 1500 });
      return;
    }

    window.setTimeout(run, 250);
  }

  async function refreshHomeData() {
    if (initialHomeRefresh) {
      initialHomeRefresh = false;
      await refreshHomeCoreData();
      scheduleInitialHomeAnalyticsRefresh();
      return;
    }

    await Promise.all([refreshHomeCoreData(), refreshHomeAnalyticsData()]);
  }

  return {
    refreshHomeData,
  };
}
