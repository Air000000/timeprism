import type { LearnHeatmapCell, UsageStackDay } from "../api";

export function heatCellClass(level: LearnHeatmapCell["level"]): string {
  if (level === "GREEN") return "green";
  if (level === "YELLOW") return "yellow";
  return "gray";
}

export function businessDayKeyNow(now = new Date()): string {
  const shifted = new Date(now.getTime() - 4 * 3600 * 1000);
  const y = shifted.getFullYear();
  const m = (shifted.getMonth() + 1).toString().padStart(2, "0");
  const d = shifted.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function getHeatmapFetchDays(viewMonthDate: Date, now = new Date()): number {
  const viewStart = new Date(viewMonthDate.getFullYear(), viewMonthDate.getMonth(), 1);
  const diffMs = now.getTime() - viewStart.getTime();
  const diffDays = Math.max(0, Math.ceil(diffMs / 86_400_000));
  const days = diffDays + 62;
  return Math.min(720, Math.max(120, days));
}

export function monthCellRows(year: number, month: number): number {
  const firstWeekday = new Date(year, month - 1, 1).getDay();
  const monthDays = new Date(year, month, 0).getDate();
  return Math.ceil((firstWeekday + monthDays) / 7);
}

export function pickCurrentBusinessDay(days: UsageStackDay[], now = new Date()): UsageStackDay | null {
  if (days.length === 0) {
    return null;
  }
  const key = businessDayKeyNow(now);
  const exact = days.find((d) => d.day === key);
  if (exact) {
    return exact;
  }
  const sorted = [...days].sort((a, b) => b.day.localeCompare(a.day));
  return sorted[0] ?? null;
}
