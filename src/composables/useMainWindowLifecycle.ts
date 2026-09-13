import { onMounted, onUnmounted } from "vue";

type MainWindowLifecycleOptions = {
  initLocale: () => void;
  initThemeModeSafely: () => void;
  loadTrackingState: () => Promise<void>;
  loadHeatmapGoalSecondsSetting: () => Promise<void>;
  resetGuardFeedback: () => void;
  resetPrivacyFeedback: () => void;
  refreshHomeData: () => Promise<void> | void;
  startSettingsWarmup: () => void;
  startMainRefreshPolling: () => void;
  startInsightsSectionNavigationListener: () => Promise<void>;
  stopMainRefreshPolling: () => void;
  stopAutoCaptureSampler: () => void;
  cleanupSettingsWarmup: () => void;
  cleanupInsightsSectionNavigation: () => void;
  cleanupHeatmapGoalSetting: () => void;
};

export function useMainWindowLifecycle({
  initLocale,
  initThemeModeSafely,
  loadTrackingState,
  loadHeatmapGoalSecondsSetting,
  resetGuardFeedback,
  resetPrivacyFeedback,
  refreshHomeData,
  startSettingsWarmup,
  startMainRefreshPolling,
  startInsightsSectionNavigationListener,
  stopMainRefreshPolling,
  stopAutoCaptureSampler,
  cleanupSettingsWarmup,
  cleanupInsightsSectionNavigation,
  cleanupHeatmapGoalSetting,
}: MainWindowLifecycleOptions) {
  onMounted(async () => {
    initLocale();
    initThemeModeSafely();

    await loadTrackingState();
    await loadHeatmapGoalSecondsSetting();

    resetGuardFeedback();
    resetPrivacyFeedback();
    void refreshHomeData();
    startSettingsWarmup();
    startMainRefreshPolling();

    await startInsightsSectionNavigationListener();
  });

  onUnmounted(() => {
    stopMainRefreshPolling();
    stopAutoCaptureSampler();
    cleanupSettingsWarmup();
    cleanupInsightsSectionNavigation();
    cleanupHeatmapGoalSetting();
  });
}
