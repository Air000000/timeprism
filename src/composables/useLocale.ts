import { ref, watch } from "vue";
import {
  getStoredOrBrowserLocale,
  isLocaleCode,
  LOCALE_STORAGE_KEY,
  translateForLocale,
  type LocaleCode,
} from "../lib/locale";

export type { LocaleCode } from "../lib/locale";

const locale = ref<LocaleCode>("zh-CN");

function tx(zh: string, en: string): string {
  return translateForLocale(locale.value, zh, en);
}

function applyLocale(next: LocaleCode) {
  locale.value = next;
  if (typeof document !== "undefined") {
    document.documentElement.lang = next;
  }
  try {
    window.localStorage.setItem(LOCALE_STORAGE_KEY, next);
  } catch {
    // Ignore persistence errors in restricted WebView contexts.
  }
}

function initLocale() {
  applyLocale(getStoredOrBrowserLocale());
}

function watchLocaleChanges() {
  return watch(locale, (next, prev) => {
    if (next === prev) {
      return;
    }
    applyLocale(next);
  });
}

function onLocaleChange(event: Event) {
  const input = event.target as HTMLSelectElement;
  if (isLocaleCode(input.value)) {
    locale.value = input.value;
  }
}

export function useLocale() {
  return {
    locale,
    tx,
    applyLocale,
    initLocale,
    watchLocaleChanges,
    onLocaleChange,
  };
}
