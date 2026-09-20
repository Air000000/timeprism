import {
  completeTrackingOnboarding,
  getTrackingState,
  setAutoCaptureEnabled,
  type TrackingState,
} from "../api";

type ReadTrackingState = () => Promise<TrackingState>;
type CompleteTrackingOnboarding = () => Promise<TrackingState>;
type SetAutoCaptureEnabled = (enabled: boolean) => Promise<TrackingState>;

export const trackingApiContract = {
  get: getTrackingState satisfies ReadTrackingState,
  completeOnboarding: completeTrackingOnboarding satisfies CompleteTrackingOnboarding,
  setAutoCaptureEnabled: setAutoCaptureEnabled satisfies SetAutoCaptureEnabled,
};
