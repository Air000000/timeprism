import { ref } from "vue";
import { useAutoCaptureSampler } from "./useAutoCaptureSampler";

const sampler = useAutoCaptureSampler({
  autoCaptureFeedback: ref(""),
  tx: (_zh, en) => en,
  intervalMs: 5000,
});

export const autoCaptureSamplerContract = sampler satisfies {
  startAutoCaptureSampler: () => void;
  stopAutoCaptureSampler: () => void;
};
