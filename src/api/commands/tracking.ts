import { invokeCommand } from "../client";
import type { TrackingState } from "../types";

export function getTrackingState(): Promise<TrackingState> {
  return invokeCommand<TrackingState>("get_tracking_state");
}

export function completeTrackingOnboarding(): Promise<TrackingState> {
  return invokeCommand<TrackingState>("complete_tracking_onboarding");
}

export function setAutoCaptureEnabled(enabled: boolean): Promise<TrackingState> {
  return invokeCommand<TrackingState>("set_auto_capture_enabled", { enabled });
}
