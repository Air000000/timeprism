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

type AllTimeFilter = "ALL" | "LEARN" | "REST";

type UseInsightsDataOptions = {
  recentLogs: Ref<RecentLog[]>;
  learnHeatmap: Ref<LearnHeatmapCell[]>;
  getHeatmapFetchDays: () => number;
  getHeatmapGoalSeconds: () => number;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

function logDayKey(unixSeconds: number): string {
  const date = new Date((unixSeconds - 4 * 3600) * 1000);
  const y = date.getFullYear();
  const m = (date.getMonth() + 1).toString().padStart(2, "0");
  const d = date.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function barWidth(seconds: number, max: number): string {
  if (max <= 0 || seconds <= 0) {
    return "0%";
  }
  return `${Math.max(8, Math.min(100, (seconds / max) * 100)).toFixed(2)}%`;
}

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
    return barWidth(seconds, allTimeTopApps.value[0]?.seconds ?? 0);
  }

  function topAppsBarWidth(seconds: number): string {
    return barWidth(seconds, topApps.value[0]?.seconds ?? 0);
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
    return barWidth(durationMs, recentLogs.value[0]?.duration_ms ?? 0);
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
