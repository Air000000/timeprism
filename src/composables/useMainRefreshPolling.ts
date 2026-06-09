import type { Ref } from "vue";
import type { MainViewKey } from "./useAppNavigation";

type MainRefreshPollingOptions = {
  currentMainView: Ref<MainViewKey>;
  refreshHomeData: () => Promise<void> | void;
  refreshGuardData: () => Promise<void> | void;
  intervalMs?: number;
};

export function useMainRefreshPolling({
  currentMainView,
  refreshHomeData,
  refreshGuardData,
  intervalMs = 5000,
}: MainRefreshPollingOptions) {
  let pollTimer: number | null = null;

  function runMainRefreshPoll() {
    void refreshHomeData();
    if (currentMainView.value === "guard") {
      void refreshGuardData();
    }
  }

  function startMainRefreshPolling() {
    stopMainRefreshPolling();
    pollTimer = window.setInterval(runMainRefreshPoll, intervalMs);
  }

  function stopMainRefreshPolling() {
    if (pollTimer !== null) {
      window.clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  return {
    startMainRefreshPolling,
    stopMainRefreshPolling,
  };
}
