import type { LocaleCode } from "./locale";

export function heatmapMonthKey(date: Date): string {
  const year = date.getFullYear();
  const month = (date.getMonth() + 1).toString().padStart(2, "0");
  return `${year}-${month}`;
}

export function heatmapMonthTitle(date: Date, locale: LocaleCode): string {
  const monthNames = locale === "zh-CN"
    ? ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"]
    : ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  return monthNames[date.getMonth()] ?? `${date.getMonth() + 1}`;
}

export function heatmapWeekHeaders(locale: LocaleCode): string[] {
  return locale === "zh-CN"
    ? ["日", "一", "二", "三", "四", "五", "六"]
    : ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
}

export function heatmapDayLabel(dayKey: string): string {
  return dayKey.slice(-2).replace(/^0/, "");
}
