import type { HistorySubViewKey, MainViewKey } from "./useAppNavigation";

type MainViewActionsOptions = {
  selectHistoryView: (next: HistorySubViewKey) => void;
  selectMainView: (next: MainViewKey) => void;
  refreshInsightsData: () => Promise<void> | void;
  refreshGuardData: () => Promise<void> | void;
  showSettingsViewAndRefresh: () => void;
};

export function useMainViewActions({
  selectHistoryView,
  selectMainView,
  refreshInsightsData,
  refreshGuardData,
  showSettingsViewAndRefresh,
}: MainViewActionsOptions) {
  function switchHistorySubView(next: HistorySubViewKey) {
    selectHistoryView(next);
    void refreshInsightsData();
  }

  function setMainView(next: MainViewKey) {
    selectMainView(next);
    if (next === "insights") {
      void refreshInsightsData();
    } else if (next === "guard") {
      void refreshGuardData();
    } else if (next === "privacy") {
      showSettingsViewAndRefresh();
    }
  }

  return {
    switchHistorySubView,
    setMainView,
  };
}
