import { onMounted, onUnmounted } from "vue";

type MainWindowLifecycleOptions = {
  initLocale: () => void;
  initThemeModeSafely: () => void;
  loadHeatmapGoalSecondsSetting: () => Promise<void>;
  resetGuardFeedback: () => void;
  resetPrivacyFeedback: () => void;
  refreshHomeData: () => Promise<void> | void;
  startSettingsWarmup: () => void;
  startMainRefreshPolling: () => void;
  startAutoCaptureSampler: () => void;
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
  loadHeatmapGoalSecondsSetting,
  resetGuardFeedback,
  resetPrivacyFeedback,
  refreshHomeData,
  startSettingsWarmup,
  startMainRefreshPolling,
  startAutoCaptureSampler,
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

    await loadHeatmapGoalSecondsSetting();

    resetGuardFeedback();
    resetPrivacyFeedback();
    void refreshHomeData();
    startSettingsWarmup();
    startMainRefreshPolling();

    startAutoCaptureSampler();

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
