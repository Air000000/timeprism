import { computed, ref, type Ref } from "vue";
import type { LearnHeatmapCell } from "../api";
import {
  heatmapDayLabel,
  heatmapMonthKey,
  heatmapMonthTitle,
  heatmapWeekHeaders,
} from "../lib/heatmapCalendarText";
import type { LocaleCode } from "../lib/locale";
import { currentLocalDayKey, localDayKeyFromDate } from "../lib/time";

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

  const fullCurrentMonthHeatmap = computed<LearnHeatmapCell[]>(() => {
    const [yearText, monthText] = currentMonthKey.value.split("-");
    const year = Number.parseInt(yearText, 10);
    const month = Number.parseInt(monthText, 10);
    if (!Number.isFinite(year) || !Number.isFinite(month)) {
      return [];
    }

    const monthDays = new Date(year, month, 0).getDate();
    const byDay = new Map(currentMonthHeatmap.value.map((cell) => [cell.day, cell]));

    const cells: LearnHeatmapCell[] = [];
    for (let day = 1; day <= monthDays; day += 1) {
      const dayKey = `${yearText}-${monthText}-${day.toString().padStart(2, "0")}`;
      const existing = byDay.get(dayKey);
      if (existing) {
        cells.push(existing);
      } else {
        cells.push({
          day: dayKey,
          learn_seconds: 0,
          level: "GRAY",
        });
      }
    }

    return cells;
  });

  const calendarHeatmapCells = computed<Array<LearnHeatmapCell | null>>(() => {
    const [yearText, monthText] = currentMonthKey.value.split("-");
    const year = Number.parseInt(yearText, 10);
    const month = Number.parseInt(monthText, 10);
    if (!Number.isFinite(year) || !Number.isFinite(month)) {
      return [];
    }

    const firstWeekday = new Date(year, month - 1, 1).getDay();
    const padding = Array.from({ length: firstWeekday }, () => null);
    const cells = [...padding, ...fullCurrentMonthHeatmap.value];
    const trailing = Math.max(0, 42 - cells.length);
    return [...cells, ...Array.from({ length: trailing }, () => null)];
  });

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

  function isFutureDay(dayKey: string): boolean {
    return dayKey > currentLocalDayKey();
  }

  function isTodayDay(dayKey: string): boolean {
    return dayKey === currentLocalDayKey();
  }

  function heatmapCellClass(cell: LearnHeatmapCell): string[] {
    const classes = ["heat-cell", cell.level.toLowerCase()];
    if (cell.level === "GREEN") {
      const ratio = cell.learn_seconds / currentMonthGreenMaxSeconds.value;
      if (ratio >= 0.88) {
        classes.push("green-4");
      } else if (ratio >= 0.72) {
        classes.push("green-3");
      } else if (ratio >= 0.56) {
        classes.push("green-2");
      } else {
        classes.push("green-1");
      }
    }
    classes.push(isFutureDay(cell.day) ? "future-date" : "past-date");
    if (isTodayDay(cell.day)) {
      classes.push("today-cell");
    }
    return classes;
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
