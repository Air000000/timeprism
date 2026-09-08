import { computed, ref, type Ref } from "vue";
import type { LearnHeatmapCell } from "../api";
import { heatmapCellClassNames } from "../lib/heatmapCalendarClasses";
import {
  buildCalendarHeatmapCells,
  buildFullMonthHeatmap,
} from "../lib/heatmapCalendarGrid";
import {
  buildHomeMonthGoalProgress,
  heatmapFetchDays,
  heatmapGreenMaxSeconds,
  homeMonthActiveStreakDays as calculateHomeMonthActiveStreakDays,
} from "../lib/heatmapCalendarProgress";
import {
  heatmapDayLabel,
  heatmapMonthKey,
  heatmapMonthTitle,
  heatmapWeekHeaders,
} from "../lib/heatmapCalendarText";
import type { LocaleCode } from "../lib/locale";

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

  const currentMonthGreenMaxSeconds = computed(() => heatmapGreenMaxSeconds(fullCurrentMonthHeatmap.value));

  const heatmapByDay = computed(() => new Map(learnHeatmap.value.map((cell) => [cell.day, cell])));

  const homeMonthGoalProgress = computed(() => buildHomeMonthGoalProgress(heatmapByDay.value));

  const homeMonthActiveStreakDays = computed(() => calculateHomeMonthActiveStreakDays(heatmapByDay.value));

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
    return heatmapFetchDays(viewMonthDate.value);
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
