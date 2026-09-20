import { ref } from "vue";
import { useAutoCaptureSampler } from "./useAutoCaptureSampler";

const sampler = useAutoCaptureSampler({
  autoCaptureFeedback: ref(""),
  tx: (_zh, en) => en,
  intervalMs: 5000,
});

// Runtime policy belongs to useTrackingState/App, not to the sampler API.
useAutoCaptureSampler({
  // @ts-expect-error autoCaptureEnabled must not be accepted by the sampler.
  autoCaptureEnabled: ref(true),
  autoCaptureFeedback: ref(""),
  tx: (_zh, en) => en,
});

export const autoCaptureSamplerContract = sampler satisfies {
  startAutoCaptureSampler: () => void;
  stopAutoCaptureSampler: () => void;
};
