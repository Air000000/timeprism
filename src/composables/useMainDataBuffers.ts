import { ref } from "vue";
import type {
  LearnHeatmapCell,
  RecentLog,
  UsageStackDay,
} from "../api";

export function useMainDataBuffers() {
  const recentLogs = ref<RecentLog[]>([]);
  const learnHeatmap = ref<LearnHeatmapCell[]>([]);
  const homeUsageStack = ref<UsageStackDay[]>([]);

  return {
    recentLogs,
    learnHeatmap,
    homeUsageStack,
  };
}
