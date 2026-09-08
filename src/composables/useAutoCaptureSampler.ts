import type { Ref } from "vue";
import { captureForegroundOnce } from "../api";

type TranslateFn = (zh: string, en: string) => string;

type AutoCaptureSamplerOptions = {
  autoCaptureFeedback: Ref<string>;
  tx: TranslateFn;
  intervalMs?: number;
  autoCaptureEnabled?: Ref<boolean>;
};

export function useAutoCaptureSampler({
  autoCaptureFeedback,
  tx,
  intervalMs = 5000,
}: AutoCaptureSamplerOptions) {
  let captureTimer: number | null = null;

  function sampleAutoCapture() {
    void captureForegroundOnce(intervalMs)
      .then((stored) => {
        autoCaptureFeedback.value = stored
          ? tx("自动采样运行中（最近一条已入库）。", "Auto capture running (latest sample stored).")
          : tx("自动采样运行中（最近一条未入库：隐私拦截或基线样本）。", "Auto capture running (latest sample not stored: privacy block or baseline sample).");
      })
      .catch((e) => {
        autoCaptureFeedback.value = tx(`自动采样失败：${e}`, `Auto capture failed: ${e}`);
      });
  }

  function startAutoCaptureSampler() {
    stopAutoCaptureSampler();
    sampleAutoCapture();
    captureTimer = window.setInterval(sampleAutoCapture, intervalMs);
  }

  function stopAutoCaptureSampler() {
    if (captureTimer !== null) {
      window.clearInterval(captureTimer);
      captureTimer = null;
    }
  }

  return {
    startAutoCaptureSampler,
    stopAutoCaptureSampler,
  };
}
