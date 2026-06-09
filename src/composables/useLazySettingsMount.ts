import { ref } from "vue";

type LazySettingsMountOptions = {
  refreshSettingsData: () => Promise<void> | void;
};

export function useLazySettingsMount({
  refreshSettingsData,
}: LazySettingsMountOptions) {
  const privacyViewMounted = ref(false);
  let settingsWarmTimer: number | null = null;

  function showSettingsViewAndRefresh() {
    privacyViewMounted.value = true;
    window.setTimeout(() => {
      void refreshSettingsData();
    }, 0);
  }

  function startSettingsWarmup() {
    settingsWarmTimer = window.setTimeout(() => {
      privacyViewMounted.value = true;
      void refreshSettingsData();
    }, 1200);
  }

  function cleanupSettingsWarmup() {
    if (settingsWarmTimer !== null) {
      window.clearTimeout(settingsWarmTimer);
      settingsWarmTimer = null;
    }
  }

  return {
    privacyViewMounted,
    showSettingsViewAndRefresh,
    startSettingsWarmup,
    cleanupSettingsWarmup,
  };
}
