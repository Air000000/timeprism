import type { LearnHeatmapCell } from "../api";
import { localDayKeyFromDate } from "./time";

export type HeatmapGoalProgress = {
  goalDays: number;
  elapsedDays: number;
};

export function heatmapGreenMaxSeconds(cells: readonly LearnHeatmapCell[]): number {
  return Math.max(
    1,
    ...cells
      .filter((cell) => cell.level === "GREEN")
      .map((cell) => cell.learn_seconds),
  );
}

export function buildHomeMonthGoalProgress(
  heatmapByDay: ReadonlyMap<string, LearnHeatmapCell>,
  today = new Date(),
): HeatmapGoalProgress {
  const year = today.getFullYear();
  const month = today.getMonth();
  const elapsedDays = today.getDate();
  let goalDays = 0;
  for (let day = 1; day <= elapsedDays; day += 1) {
    const dayKey = localDayKeyFromDate(new Date(year, month, day));
    if ((heatmapByDay.get(dayKey)?.level ?? "GRAY") === "GREEN") {
      goalDays += 1;
    }
  }
  return {
    goalDays,
    elapsedDays,
  };
}

export function homeMonthActiveStreakDays(
  heatmapByDay: ReadonlyMap<string, LearnHeatmapCell>,
  today = new Date(),
): number {
  let streak = 0;
  for (let day = today.getDate(); day >= 1; day -= 1) {
    const dayKey = localDayKeyFromDate(new Date(today.getFullYear(), today.getMonth(), day));
    const cell = heatmapByDay.get(dayKey);
    if ((cell?.learn_seconds ?? 0) <= 0) {
      break;
    }
    streak += 1;
  }
  return streak;
}

export function heatmapFetchDays(viewMonthDate: Date, now = new Date()): number {
  const viewYear = viewMonthDate.getFullYear();
  const viewMonth = viewMonthDate.getMonth();
  const viewStart = new Date(viewYear, viewMonth, 1, 0, 0, 0, 0);
  const diffMs = now.getTime() - viewStart.getTime();
  const diffDays = Math.max(0, Math.ceil(diffMs / 86_400_000));
  const days = diffDays + 62;
  return Math.min(1800, Math.max(120, days));
}
