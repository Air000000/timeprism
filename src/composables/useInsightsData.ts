import { computed, ref, type Ref } from "vue";
import {
  getLearnHeatmap,
  listRecentLogs,
  listTopAppsAllTime,
  listTopAppsToday,
  type LearnHeatmapCell,
  type RecentLog,
  type TopApp,
} from "../api";
import {
  buildRecentTimelineGroups,
  insightsBarWidth,
} from "../lib/insightsMetrics";

type AllTimeFilter = "ALL" | "LEARN" | "REST";

type UseInsightsDataOptions = {
  recentLogs: Ref<RecentLog[]>;
  learnHeatmap: Ref<LearnHeatmapCell[]>;
  getHeatmapFetchDays: () => number;
  getHeatmapGoalSeconds: () => number;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

export function useInsightsData({
  recentLogs,
  learnHeatmap,
  getHeatmapFetchDays,
  getHeatmapGoalSeconds,
  refreshData,
  setErrorMessage,
}: UseInsightsDataOptions) {
  const loadingInsights = ref(false);
  const topApps = ref<TopApp[]>([]);
  const allTimeTopApps = ref<TopApp[]>([]);
  const allTimeFilter = ref<AllTimeFilter>("ALL");
  const allTimeIncludeIgnore = ref(true);

  const recentTimelineGroups = computed(() => buildRecentTimelineGroups(recentLogs.value));

  async function refreshInsightsData() {
    if (loadingInsights.value) {
      return;
    }

    loadingInsights.value = true;
    try {
      const [apps, allTimeApps, logs, heatmap] = await Promise.all([
        listTopAppsToday(6),
        listTopAppsAllTime(10, allTimeFilter.value, allTimeIncludeIgnore.value),
        listRecentLogs(12),
        getLearnHeatmap(getHeatmapFetchDays(), getHeatmapGoalSeconds()),
      ]);
      topApps.value = apps;
      allTimeTopApps.value = allTimeApps;
      recentLogs.value = logs;
      learnHeatmap.value = heatmap;
    } catch (e) {
      setErrorMessage(e);
    } finally {
      loadingInsights.value = false;
    }
  }

  function allTimeBarWidth(seconds: number): string {
    return insightsBarWidth(seconds, allTimeTopApps.value[0]?.seconds ?? 0);
  }

  function topAppsBarWidth(seconds: number): string {
    return insightsBarWidth(seconds, topApps.value[0]?.seconds ?? 0);
  }

  function setAllTimeFilter(next: AllTimeFilter) {
    allTimeFilter.value = next;
    void refreshData();
  }

  function onAllTimeIgnoreToggle(event: Event) {
    const input = event.target as HTMLInputElement;
    allTimeIncludeIgnore.value = input.checked;
    void refreshData();
  }

  function recentDurationWidth(durationMs: number): string {
    return insightsBarWidth(durationMs, recentLogs.value[0]?.duration_ms ?? 0);
  }

  return {
    topApps,
    topAppsBarWidth,
    allTimeTopApps,
    allTimeIncludeIgnore,
    allTimeBarWidth,
    setAllTimeFilter,
    onAllTimeIgnoreToggle,
    recentTimelineGroups,
    recentDurationWidth,
    refreshInsightsData,
  };
}
