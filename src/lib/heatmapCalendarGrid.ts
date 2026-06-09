import type { LearnHeatmapCell } from "../api";

function parseMonthKey(monthKey: string): { year: number; month: number; yearText: string; monthText: string } | null {
  const [yearText, monthText] = monthKey.split("-");
  const year = Number.parseInt(yearText, 10);
  const month = Number.parseInt(monthText, 10);
  if (!Number.isFinite(year) || !Number.isFinite(month)) {
    return null;
  }
  return { year, month, yearText, monthText };
}

export function buildFullMonthHeatmap(
  monthKey: string,
  currentMonthHeatmap: readonly LearnHeatmapCell[],
): LearnHeatmapCell[] {
  const parsed = parseMonthKey(monthKey);
  if (!parsed) {
    return [];
  }

  const monthDays = new Date(parsed.year, parsed.month, 0).getDate();
  const byDay = new Map(currentMonthHeatmap.map((cell) => [cell.day, cell]));

  const cells: LearnHeatmapCell[] = [];
  for (let day = 1; day <= monthDays; day += 1) {
    const dayKey = `${parsed.yearText}-${parsed.monthText}-${day.toString().padStart(2, "0")}`;
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
}

export function buildCalendarHeatmapCells(
  monthKey: string,
  fullCurrentMonthHeatmap: readonly LearnHeatmapCell[],
): Array<LearnHeatmapCell | null> {
  const parsed = parseMonthKey(monthKey);
  if (!parsed) {
    return [];
  }

  const firstWeekday = new Date(parsed.year, parsed.month - 1, 1).getDay();
  const padding = Array.from({ length: firstWeekday }, () => null);
  const cells = [...padding, ...fullCurrentMonthHeatmap];
  const trailing = Math.max(0, 42 - cells.length);
  return [...cells, ...Array.from({ length: trailing }, () => null)];
}
