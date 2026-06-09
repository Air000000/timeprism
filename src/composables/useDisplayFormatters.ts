import type { Ref } from "vue";
import type { IdlePrompt } from "../api";
import { formatSeconds } from "../lib/time";
import type { LocaleCode } from "./useLocale";

type TranslateFn = (zh: string, en: string) => string;
type RuleMappedType = "LEARN" | "REST" | "IGNORE";

type UseDisplayFormattersOptions = {
  locale: Ref<LocaleCode>;
  tx: TranslateFn;
};

export function useDisplayFormatters({
  locale,
  tx,
}: UseDisplayFormattersOptions) {
  function formatClock(unixSeconds: number): string {
    const date = new Date(unixSeconds * 1000);
    return date.toLocaleTimeString(locale.value, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }

  function mappedTypeText(mappedType: RuleMappedType) {
    if (mappedType === "LEARN") {
      return tx("学习", "Learn");
    }
    if (mappedType === "REST") {
      return tx("休息", "Break");
    }
    return tx("未分类", "Unclassified");
  }

  function cleanProcessName(name: string): string {
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

  function formatIdlePromptSpan(item: IdlePrompt): string {
    const start = new Date(item.start_timestamp * 1000);
    const end = new Date(item.end_timestamp * 1000);
    const durationSeconds = Math.max(0, Math.floor(item.duration_ms / 1000));
    return `${start.toLocaleTimeString(locale.value, { hour: "2-digit", minute: "2-digit" })} - ${end.toLocaleTimeString(locale.value, { hour: "2-digit", minute: "2-digit" })} (${formatSeconds(durationSeconds)})`;
  }

  return {
    formatClock,
    mappedTypeText,
    cleanProcessName,
    formatIdlePromptSpan,
  };
}
