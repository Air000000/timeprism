import { trackingUiMode } from "./trackingUi";

const loading: "loading" = trackingUiMode(false, false);
const loadingEvenIfStateIsStale: "loading" = trackingUiMode(false, true);
const onboarding: "onboarding" = trackingUiMode(true, false);
const app: "app" = trackingUiMode(true, true);

export const trackingUiContract = {
  loading,
  loadingEvenIfStateIsStale,
  onboarding,
  app,
};
