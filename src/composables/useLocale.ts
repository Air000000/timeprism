import { ref } from "vue";

export type LocaleCode = "zh-CN" | "en-US";

const LOCALE_STORAGE_KEY = "timeprism-locale";
const locale = ref<LocaleCode>("zh-CN");

function tx(zh: string, en: string): string {
  return locale.value === "zh-CN" ? zh : en;
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
  try {
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored === "zh-CN" || stored === "en-US") {
      applyLocale(stored);
      return;
    }
  } catch {
    // Ignore read errors and continue with system preference.
  }

  const browserLang = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-cn";
  applyLocale(browserLang.startsWith("zh") ? "zh-CN" : "en-US");
}

export function useLocale() {
  return {
    locale,
    tx,
    applyLocale,
    initLocale,
  };
}
