export type LocaleCode = "zh-CN" | "en-US";

export const LOCALE_STORAGE_KEY = "timeprism-locale";

export function isLocaleCode(value: string | null | undefined): value is LocaleCode {
  return value === "zh-CN" || value === "en-US";
}

export function detectBrowserLocale(): LocaleCode {
  const browserLang = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-cn";
  return browserLang.startsWith("zh") ? "zh-CN" : "en-US";
}

export function readStoredLocale(): LocaleCode | null {
  try {
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    return isLocaleCode(stored) ? stored : null;
  } catch {
    return null;
  }
}

export function getStoredOrBrowserLocale(): LocaleCode {
  return readStoredLocale() ?? detectBrowserLocale();
}

export function translateForLocale(locale: LocaleCode, zh: string, en: string): string {
  return locale === "zh-CN" ? zh : en;
}
