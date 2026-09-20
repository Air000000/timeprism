import { computed, ref } from "vue";
import {
  completeTrackingOnboarding,
  getTrackingState,
  setAutoCaptureEnabled,
  type TrackingState,
} from "../api";

type UseTrackingStateOptions = {
  setErrorMessage: (error: unknown) => void;
};

export function useTrackingState({ setErrorMessage }: UseTrackingStateOptions) {
  const trackingState = ref<TrackingState | null>(null);
  const trackingReady = ref(false);
  const trackingActionLoading = ref(false);

  const onboardingCompleted = computed(
    () => trackingState.value?.onboarding_completed === true,
  );
  const autoCaptureEnabled = computed(
    () => trackingState.value?.auto_capture_enabled === true,
  );
  const captureShouldRun = computed(
    () =>
      trackingReady.value &&
      onboardingCompleted.value &&
      autoCaptureEnabled.value,
  );

  async function loadTrackingState(): Promise<void> {
    try {
      trackingState.value = await getTrackingState();
      trackingReady.value = true;
    } catch (error) {
      trackingReady.value = false;
      setErrorMessage(error);
    }
  }

  async function completeOnboarding(): Promise<void> {
    if (trackingActionLoading.value) {
      return;
    }

    trackingActionLoading.value = true;
    try {
      trackingState.value = await completeTrackingOnboarding();
      trackingReady.value = true;
    } catch (error) {
      setErrorMessage(error);
      throw error;
    } finally {
      trackingActionLoading.value = false;
    }
  }

  async function persistAutoCaptureEnabled(enabled: boolean): Promise<void> {
    if (trackingActionLoading.value) {
      return;
    }

    trackingActionLoading.value = true;
    try {
      trackingState.value = await setAutoCaptureEnabled(enabled);
    } catch (error) {
      setErrorMessage(error);
      throw error;
    } finally {
      trackingActionLoading.value = false;
    }
  }

  return {
    trackingReady,
    onboardingCompleted,
    autoCaptureEnabled,
    captureShouldRun,
    trackingActionLoading,
    loadTrackingState,
    completeOnboarding,
    persistAutoCaptureEnabled,
  };
}
