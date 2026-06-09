import type { Ref } from "vue";
import type { IdlePrompt } from "../api";
import {
  cleanUiProcessName,
  formatIdlePromptTimeSpan,
  formatUiClock,
  mappedRuleTypeText,
  type RuleMappedType,
} from "../lib/displayFormatters";
import type { LocaleCode } from "../lib/locale";

type TranslateFn = (zh: string, en: string) => string;

type UseDisplayFormattersOptions = {
  locale: Ref<LocaleCode>;
  tx: TranslateFn;
};

export function useDisplayFormatters({
  locale,
  tx,
}: UseDisplayFormattersOptions) {
  function formatClock(unixSeconds: number): string {
    return formatUiClock(unixSeconds, locale.value);
  }

  function mappedTypeText(mappedType: RuleMappedType) {
    return mappedRuleTypeText(mappedType, tx);
  }

  function cleanProcessName(name: string): string {
    return cleanUiProcessName(name, tx);
  }

  function formatIdlePromptSpan(item: IdlePrompt): string {
    return formatIdlePromptTimeSpan(item, locale.value);
  }

  return {
    formatClock,
    mappedTypeText,
    cleanProcessName,
    formatIdlePromptSpan,
  };
}
