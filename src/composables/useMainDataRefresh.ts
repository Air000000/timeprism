import type { Ref } from "vue";
import type { MainViewKey } from "./useAppNavigation";

type RefreshFn = () => Promise<void> | void;

type MainDataRefreshHandlers = {
  refreshHomeData: RefreshFn;
  refreshInsightsData: RefreshFn;
  refreshGuardData: RefreshFn;
  refreshSettingsData: RefreshFn;
};

export function useMainDataRefresh(currentMainView: Ref<MainViewKey>) {
  let handlers: MainDataRefreshHandlers | null = null;

  function bindMainDataRefreshHandlers(next: MainDataRefreshHandlers) {
    handlers = next;
  }

  async function refreshData() {
    if (!handlers) {
      throw new Error("Main data refresh handlers are not bound.");
    }

    await handlers.refreshHomeData();
    if (currentMainView.value === "insights") {
      await handlers.refreshInsightsData();
    } else if (currentMainView.value === "guard") {
      await handlers.refreshGuardData();
    } else if (currentMainView.value === "privacy") {
      await handlers.refreshSettingsData();
    }
  }

  return {
    bindMainDataRefreshHandlers,
    refreshData,
  };
}
