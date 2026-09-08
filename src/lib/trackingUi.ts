export type TrackingUiMode = "loading" | "onboarding" | "app";

export function trackingUiMode(
  trackingReady: false,
  onboardingCompleted: boolean,
): "loading";
export function trackingUiMode(
  trackingReady: true,
  onboardingCompleted: false,
): "onboarding";
export function trackingUiMode(
  trackingReady: true,
  onboardingCompleted: true,
): "app";
export function trackingUiMode(
  trackingReady: boolean,
  onboardingCompleted: boolean,
): TrackingUiMode;
export function trackingUiMode(
  trackingReady: boolean,
  onboardingCompleted: boolean,
): TrackingUiMode {
  if (!trackingReady) {
    return "loading";
  }
  return onboardingCompleted ? "app" : "onboarding";
}
