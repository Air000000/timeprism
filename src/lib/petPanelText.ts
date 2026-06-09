import type { LocaleCode } from "./locale";

export function petPanelWeekHeaders(locale: LocaleCode): string[] {
  return locale === "zh-CN"
    ? ["日", "一", "二", "三", "四", "五", "六"]
    : ["S", "M", "T", "W", "T", "F", "S"];
}
