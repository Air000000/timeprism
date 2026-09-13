import { useMainWindowLifecycle } from "./useMainWindowLifecycle";

export const mainWindowLifecycleContract: Parameters<typeof useMainWindowLifecycle>[0] = {
  initLocale: () => {},
  initThemeModeSafely: () => {},
  loadTrackingState: async () => {},
  loadHeatmapGoalSecondsSetting: async () => {},
  resetGuardFeedback: () => {},
  resetPrivacyFeedback: () => {},
  refreshHomeData: async () => {},
  startSettingsWarmup: () => {},
  startMainRefreshPolling: () => {},
  startInsightsSectionNavigationListener: async () => {},
  stopMainRefreshPolling: () => {},
  stopAutoCaptureSampler: () => {},
  cleanupSettingsWarmup: () => {},
  cleanupInsightsSectionNavigation: () => {},
  cleanupHeatmapGoalSetting: () => {},
};
