import { ref } from "vue";

type LazySettingsMountOptions = {
  refreshSettingsData: () => Promise<void> | void;
};

export function useLazySettingsMount({
  refreshSettingsData,
}: LazySettingsMountOptions) {
  const privacyViewMounted = ref(false);

  function showSettingsViewAndRefresh() {
    privacyViewMounted.value = true;
    window.setTimeout(() => {
      void refreshSettingsData();
    }, 0);
  }

  function startSettingsWarmup() {
    // Settings data is intentionally loaded on demand so startup stays responsive.
  }

  function cleanupSettingsWarmup() {
    // Kept for lifecycle compatibility; there is no startup warmup timer to clean up.
  }

  return {
    privacyViewMounted,
    showSettingsViewAndRefresh,
    startSettingsWarmup,
    cleanupSettingsWarmup,
  };
}
