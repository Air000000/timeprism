import type { IdlePrompt } from "../api";
import type { LocaleCode } from "./locale";
import { formatSeconds } from "./time";

type TranslateFn = (zh: string, en: string) => string;

export type RuleMappedType = "LEARN" | "REST" | "IGNORE";

export function formatUiClock(unixSeconds: number, locale: LocaleCode): string {
  const date = new Date(unixSeconds * 1000);
  return date.toLocaleTimeString(locale, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

export function mappedRuleTypeText(mappedType: RuleMappedType, tx: TranslateFn): string {
  if (mappedType === "LEARN") {
    return tx("学习", "Learn");
  }
  if (mappedType === "REST") {
    return tx("休息", "Break");
  }
  return tx("未分类", "Unclassified");
}

export function cleanUiProcessName(name: string, tx: TranslateFn): string {
  const normalized = name
    .replace(/^__idle_learn__\.exe$/i, tx("离开时段（已归类为学习）", "Away Segment (Learn)"))
    .replace(/^__idle_rest__\.exe$/i, tx("离开时段（已归类为休息）", "Away Segment (Break)"))
    .replace(/^__idle__\.exe$/i, tx("离开时段", "Away Segment"))
    .replace(/^idle_learn$/i, tx("离开时段（已归类为学习）", "Away Segment (Learn)"))
    .replace(/^idle_rest$/i, tx("离开时段（已归类为休息）", "Away Segment (Break)"))
    .replace(/^system\.idle$/i, tx("离开时段", "Away Segment"))
    .replace(/^idle\.segment$/i, tx("离开时段", "Away Segment"))
    .replace(/\.exe(?=\s*(\(|$))/gi, "")
    .replace(/\s*\([^)]*\)\s*$/g, "")
    .replace(/\s{2,}/g, " ")
    .trim();

  return normalized || tx("未知进程", "Unknown App");
}

export function formatIdlePromptTimeSpan(item: IdlePrompt, locale: LocaleCode): string {
  const start = new Date(item.start_timestamp * 1000);
  const end = new Date(item.end_timestamp * 1000);
  const durationSeconds = Math.max(0, Math.floor(item.duration_ms / 1000));
  return `${start.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" })} - ${end.toLocaleTimeString(locale, { hour: "2-digit", minute: "2-digit" })} (${formatSeconds(durationSeconds)})`;
}
