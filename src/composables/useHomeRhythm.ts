import { computed, type Ref } from "vue";
import type { UsageStackDay } from "../api";
import { currentLocalDayKey, localDayKeyFromDate } from "../lib/time";

type HomeRhythmBar = {
  day: string;
  label: string;
  totalSeconds: number;
  computerTotalSeconds: number;
  learnSeconds: number;
  restSeconds: number;
  totalHeightPx: number;
  learnHeightPx: number;
  restHeightPx: number;
  totalHeight: string;
  learnHeight: string;
  restHeight: string;
  isToday: boolean;
};

const HOME_RHYTHM_MIN_HEIGHT = 18;
const HOME_RHYTHM_MAX_HEIGHT = 176;

function homeRhythmHeightForRatio(ratio: number): number {
  if (ratio <= 0) {
    return HOME_RHYTHM_MIN_HEIGHT;
  }
  const scaled = HOME_RHYTHM_MIN_HEIGHT + ratio * (HOME_RHYTHM_MAX_HEIGHT - HOME_RHYTHM_MIN_HEIGHT);
  return Math.round(Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.min(HOME_RHYTHM_MAX_HEIGHT, scaled)));
}

export function useHomeRhythm(homeUsageStack: Ref<UsageStackDay[]>) {
  const homeMonthRhythmBars = computed<HomeRhythmBar[]>(() => {
    const today = new Date();
    const byDay = new Map(homeUsageStack.value.map((item) => [item.day, item]));
    const dayKeys = Array.from({ length: 7 }, (_, index) => {
      const date = new Date(today.getFullYear(), today.getMonth(), today.getDate() - (6 - index));
      return localDayKeyFromDate(date);
    });
    const bars = dayKeys.map((dayKey) => {
      const day = byDay.get(dayKey);
      const learnSeconds = day?.learn_seconds ?? 0;
      const restSeconds = day?.rest_seconds ?? 0;
      const computerTotalSeconds = day?.total_seconds ?? 0;
      const totalSeconds = learnSeconds + restSeconds;
      return {
        day: dayKey,
        label: dayKey.slice(-2).replace(/^0/, ""),
        totalSeconds,
        computerTotalSeconds,
        learnSeconds,
        restSeconds,
        isToday: dayKey === currentLocalDayKey(),
      };
    });
    const maxSeconds = Math.max(1, ...bars.map((item) => item.totalSeconds));
    return bars.map((item) => {
      const ratio = item.totalSeconds <= 0 ? 0 : item.totalSeconds / maxSeconds;
      const totalHeightPx = homeRhythmHeightForRatio(ratio);
      const pieceTotal = Math.max(1, item.learnSeconds + item.restSeconds);
      const learnHeightPx = item.learnSeconds > 0
        ? Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.round(totalHeightPx * (item.learnSeconds / pieceTotal)))
        : 0;
      const restHeightPx = item.restSeconds > 0
        ? Math.max(HOME_RHYTHM_MIN_HEIGHT, Math.round(totalHeightPx * (item.restSeconds / pieceTotal)))
        : 0;
      return {
        day: item.day,
        label: item.label,
        totalSeconds: item.totalSeconds,
        computerTotalSeconds: item.computerTotalSeconds,
        learnSeconds: item.learnSeconds,
        restSeconds: item.restSeconds,
        totalHeightPx,
        learnHeightPx,
        restHeightPx,
        totalHeight: `${totalHeightPx}px`,
        learnHeight: `${learnHeightPx}px`,
        restHeight: `${restHeightPx}px`,
        isToday: item.isToday,
      };
    });
  });

  return {
    homeMonthRhythmBars,
  };
}
