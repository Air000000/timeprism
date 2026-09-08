import type { RecentLog } from "../api";

export type RecentTimelineGroup = {
  day: string;
  items: RecentLog[];
};

export function insightsLogDayKey(unixSeconds: number): string {
  const date = new Date((unixSeconds - 4 * 3600) * 1000);
  const y = date.getFullYear();
  const m = (date.getMonth() + 1).toString().padStart(2, "0");
  const d = date.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

export function insightsBarWidth(seconds: number, max: number): string {
  if (max <= 0 || seconds <= 0) {
    return "0%";
  }
  return `${Math.max(8, Math.min(100, (seconds / max) * 100)).toFixed(2)}%`;
}

export function buildRecentTimelineGroups(logs: readonly RecentLog[]): RecentTimelineGroup[] {
  const groups = new Map<string, RecentLog[]>();
  for (const item of logs) {
    const day = insightsLogDayKey(item.start_timestamp);
    const list = groups.get(day) ?? [];
    list.push(item);
    groups.set(day, list);
  }
  return Array.from(groups.entries()).map(([day, items]) => ({ day, items }));
}
