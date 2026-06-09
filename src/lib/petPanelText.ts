import type { LocaleCode } from "./locale";
import { formatSeconds } from "./time";

type TranslateFn = (zh: string, en: string) => string;
type PetPanelMode = "heatmap" | "stack";

export function petPanelWeekHeaders(locale: LocaleCode): string[] {
  return locale === "zh-CN"
    ? ["日", "一", "二", "三", "四", "五", "六"]
    : ["S", "M", "T", "W", "T", "F", "S"];
}

export function petPanelEmptyStackText(tx: TranslateFn): string {
  return tx("暂无今日色块", "No stack data for today");
}

export function petPanelBusinessDayText(day: string, tx: TranslateFn): string {
  return tx(`业务日 ${day}`, `Business day ${day}`);
}

export function petPanelTotalText(totalSeconds: number, tx: TranslateFn): string {
  return tx(`总计 ${formatSeconds(totalSeconds)}`, `Total ${formatSeconds(totalSeconds)}`);
}

export function petPanelModeTitle(mode: PetPanelMode, tx: TranslateFn): string {
  return mode === "heatmap"
    ? tx("学习日历", "Learning Calendar")
    : tx("周活跃", "Weekly Activity");
}
