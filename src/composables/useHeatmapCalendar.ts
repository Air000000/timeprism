import { computed, ref, type Ref } from "vue";
import type { LearnHeatmapCell } from "../api";
import { heatmapCellClassNames } from "../lib/heatmapCalendarClasses";
import {
  buildCalendarHeatmapCells,
  buildFullMonthHeatmap,
} from "../lib/heatmapCalendarGrid";
import {
  heatmapDayLabel,
  heatmapMonthKey,
  heatmapMonthTitle,
  heatmapWeekHeaders,
} from "../lib/heatmapCalendarText";
import type { LocaleCode } from "../lib/locale";
import { localDayKeyFromDate } from "../lib/time";

type UseHeatmapCalendarOptions = {
  locale: Ref<LocaleCode>;
  learnHeatmap: Ref<LearnHeatmapCell[]>;
};

export function useHeatmapCalendar({
  locale,
  learnHeatmap,
}: UseHeatmapCalendarOptions) {
  const viewMonthDate = ref(new Date(new Date().getFullYear(), new Date().getMonth(), 1));

  const currentMonthKey = computed(() => heatmapMonthKey(viewMonthDate.value));

  const monthTitleText = computed(() => heatmapMonthTitle(viewMonthDate.value, locale.value));

  const weekHeaders = computed(() => heatmapWeekHeaders(locale.value));

  const currentMonthHeatmap = computed(() =>
    learnHeatmap.value.filter((cell) => cell.day.startsWith(currentMonthKey.value)),
  );

  const fullCurrentMonthHeatmap = computed<LearnHeatmapCell[]>(() =>
    buildFullMonthHeatmap(currentMonthKey.value, currentMonthHeatmap.value),
  );

  const calendarHeatmapCells = computed<Array<LearnHeatmapCell | null>>(() =>
    buildCalendarHeatmapCells(currentMonthKey.value, fullCurrentMonthHeatmap.value),
  );

  const currentMonthGreenMaxSeconds = computed(() =>
    Math.max(
      1,
      ...fullCurrentMonthHeatmap.value
        .filter((cell) => cell.level === "GREEN")
        .map((cell) => cell.learn_seconds),
    )
  );

  const heatmapByDay = computed(() => new Map(learnHeatmap.value.map((cell) => [cell.day, cell])));

  const homeMonthGoalProgress = computed(() => {
    const today = new Date();
    const year = today.getFullYear();
    const month = today.getMonth();
    const elapsedDays = today.getDate();
    let goalDays = 0;
    for (let day = 1; day <= elapsedDays; day += 1) {
      const dayKey = localDayKeyFromDate(new Date(year, month, day));
      if ((heatmapByDay.value.get(dayKey)?.level ?? "GRAY") === "GREEN") {
        goalDays += 1;
      }
    }
    return {
      goalDays,
      elapsedDays,
    };
  });

  const homeMonthActiveStreakDays = computed(() => {
    const today = new Date();
    let streak = 0;
    for (let day = today.getDate(); day >= 1; day -= 1) {
      const dayKey = localDayKeyFromDate(new Date(today.getFullYear(), today.getMonth(), day));
      const cell = heatmapByDay.value.get(dayKey);
      if ((cell?.learn_seconds ?? 0) <= 0) {
        break;
      }
      streak += 1;
    }
    return streak;
  });

  function heatmapCellClass(cell: LearnHeatmapCell): string[] {
    return heatmapCellClassNames(cell, currentMonthGreenMaxSeconds.value);
  }

  function heatmapDayText(dayKey: string): string {
    return heatmapDayLabel(dayKey);
  }

  function shiftHeatmapMonth(delta: number) {
    const year = viewMonthDate.value.getFullYear();
    const month = viewMonthDate.value.getMonth();
    viewMonthDate.value = new Date(year, month + delta, 1);
  }

  function getHeatmapFetchDays(): number {
    const now = new Date();
    const viewYear = viewMonthDate.value.getFullYear();
    const viewMonth = viewMonthDate.value.getMonth();
    const viewStart = new Date(viewYear, viewMonth, 1, 0, 0, 0, 0);
    const diffMs = now.getTime() - viewStart.getTime();
    const diffDays = Math.max(0, Math.ceil(diffMs / 86_400_000));
    const days = diffDays + 62;
    return Math.min(1800, Math.max(120, days));
  }

  return {
    monthTitleText,
    weekHeaders,
    calendarHeatmapCells,
    heatmapCellClass,
    heatmapDayText,
    shiftHeatmapMonth,
    getHeatmapFetchDays,
    homeMonthGoalProgress,
    homeMonthActiveStreakDays,
  };
}
