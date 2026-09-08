import { computed, ref } from "vue";

export type MainViewKey = "home" | "insights" | "guard" | "privacy";
export type InsightsPrimaryViewKey = "history";
export type HistorySubViewKey = "topApps" | "allTime" | "recent";
export type MainViewOption = {
  key: MainViewKey;
  label: string;
};

type TranslateFn = (zh: string, en: string) => string;

export function useAppNavigation(tx: TranslateFn) {
  const currentMainView = ref<MainViewKey>("home");
  const insightsPrimaryView = ref<InsightsPrimaryViewKey>("history");
  const historySubView = ref<HistorySubViewKey>("topApps");

  const mainViews = computed<MainViewOption[]>(() => [
    { key: "home", label: tx("首页", "Home") },
    { key: "insights", label: tx("数据看板", "Insights") },
    { key: "guard", label: tx("专注守护", "Focus Guard") },
    { key: "privacy", label: tx("设置", "Settings") },
  ]);

  const historySubViews = computed<Array<{ key: HistorySubViewKey; label: string }>>(() => [
    { key: "recent", label: tx("最近记录", "Recent Logs") },
    { key: "topApps", label: tx("今日时长", "Top Apps") },
    { key: "allTime", label: tx("历史总时长", "All-time Usage") },
  ]);

  function selectInsightsView(next: InsightsPrimaryViewKey) {
    currentMainView.value = "insights";
    insightsPrimaryView.value = next;
  }

  function selectHistoryView(next: HistorySubViewKey) {
    currentMainView.value = "insights";
    insightsPrimaryView.value = "history";
    historySubView.value = next;
  }

  function selectMainView(next: MainViewKey) {
    currentMainView.value = next;
    if (next === "insights" && insightsPrimaryView.value === "history") {
      historySubView.value = "topApps";
    }
  }

  function selectGuardView() {
    currentMainView.value = "guard";
  }

  return {
    currentMainView,
    mainViews,
    insightsPrimaryView,
    historySubView,
    historySubViews,
    selectInsightsView,
    selectHistoryView,
    selectMainView,
    selectGuardView,
  };
}
