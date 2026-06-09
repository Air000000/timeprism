import { computed, type Ref } from "vue";
import type {
  InsightsViewContext,
  TranslateFn,
} from "../components/viewContexts";

type InsightsViewContextOptions = {
  tx: TranslateFn;
  formatSeconds: InsightsViewContext["formatSeconds"];
  cleanProcessName: InsightsViewContext["cleanProcessName"];
  formatClock: InsightsViewContext["formatClock"];
  historySubViews: Readonly<Ref<InsightsViewContext["historySubViews"]>>;
  historySubView: Readonly<Ref<InsightsViewContext["historySubView"]>>;
  switchHistorySubView: InsightsViewContext["switchHistorySubView"];
  topApps: Readonly<Ref<InsightsViewContext["topApps"]>>;
  topAppsBarWidth: InsightsViewContext["topAppsBarWidth"];
  allTimeTopApps: Readonly<Ref<InsightsViewContext["allTimeTopApps"]>>;
  setAllTimeFilter: InsightsViewContext["setAllTimeFilter"];
  allTimeIncludeIgnore: Readonly<Ref<boolean>>;
  onAllTimeIgnoreToggle: InsightsViewContext["onAllTimeIgnoreToggle"];
  allTimeBarWidth: InsightsViewContext["allTimeBarWidth"];
  recentTimelineGroups: Readonly<Ref<InsightsViewContext["recentTimelineGroups"]>>;
  recentDurationWidth: InsightsViewContext["recentDurationWidth"];
};

export function useInsightsViewContext({
  tx,
  formatSeconds,
  cleanProcessName,
  formatClock,
  historySubViews,
  historySubView,
  switchHistorySubView,
  topApps,
  topAppsBarWidth,
  allTimeTopApps,
  setAllTimeFilter,
  allTimeIncludeIgnore,
  onAllTimeIgnoreToggle,
  allTimeBarWidth,
  recentTimelineGroups,
  recentDurationWidth,
}: InsightsViewContextOptions) {
  return computed<InsightsViewContext>(() => ({
    tx,
    formatSeconds,
    cleanProcessName,
    formatClock,
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
}
