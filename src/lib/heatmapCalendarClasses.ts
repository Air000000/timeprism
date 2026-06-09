import type { LearnHeatmapCell } from "../api";
import { currentLocalDayKey } from "./time";

export function heatmapCellClassNames(
  cell: LearnHeatmapCell,
  currentMonthGreenMaxSeconds: number,
): string[] {
  const classes = ["heat-cell", cell.level.toLowerCase()];
  if (cell.level === "GREEN") {
    const ratio = cell.learn_seconds / currentMonthGreenMaxSeconds;
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
  classes.push(cell.day > currentLocalDayKey() ? "future-date" : "past-date");
  if (cell.day === currentLocalDayKey()) {
    classes.push("today-cell");
  }
  return classes;
}
