import { computed, type Ref } from "vue";
import type { UsageStackDay } from "../api";
import { buildHomeRhythmBars, type HomeRhythmBar } from "../lib/homeRhythmBars";

export type { HomeRhythmBar } from "../lib/homeRhythmBars";

export function useHomeRhythm(homeUsageStack: Ref<UsageStackDay[]>) {
  const homeMonthRhythmBars = computed<HomeRhythmBar[]>(() => buildHomeRhythmBars(homeUsageStack.value));

  return {
    homeMonthRhythmBars,
  };
}
