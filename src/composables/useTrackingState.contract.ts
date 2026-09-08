import type { Ref } from "vue";
import { useTrackingState } from "./useTrackingState";

const tracking = useTrackingState({
  setErrorMessage: (_error: unknown) => {},
});

export const trackingStateContract = tracking satisfies {
  trackingReady: Readonly<Ref<boolean>>;
  onboardingCompleted: Readonly<Ref<boolean>>;
  autoCaptureEnabled: Readonly<Ref<boolean>>;
  captureShouldRun: Readonly<Ref<boolean>>;
  trackingActionLoading: Readonly<Ref<boolean>>;
  loadTrackingState: () => Promise<void>;
  completeOnboarding: () => Promise<void>;
  persistAutoCaptureEnabled: (enabled: boolean) => Promise<void>;
};
