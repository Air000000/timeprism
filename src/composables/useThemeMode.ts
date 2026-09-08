import { ref } from "vue";

export type ThemeMode = "light" | "dark";

const THEME_STORAGE_KEY = "timeprism-theme";
const themeMode = ref<ThemeMode>("light");

function applyTheme(nextTheme: ThemeMode) {
  themeMode.value = nextTheme;
  if (typeof document !== "undefined") {
    document.body.setAttribute("data-theme", nextTheme);
  }
  try {
    window.localStorage.setItem(THEME_STORAGE_KEY, nextTheme);
  } catch {
    // Ignore persistence errors in restricted WebView contexts.
  }
}

function toggleThemeMode() {
  applyTheme(themeMode.value === "dark" ? "light" : "dark");
}

function initThemeMode() {
  try {
    const stored = window.localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark") {
      applyTheme(stored);
      return;
    }
  } catch {
    // Ignore read errors and continue with system preference.
  }

  const prefersDark = typeof window.matchMedia === "function"
    && window.matchMedia("(prefers-color-scheme: dark)").matches;
  applyTheme(prefersDark ? "dark" : "light");
}

function initThemeModeSafely() {
  try {
    initThemeMode();
  } catch {
    applyTheme("light");
  }
}

export function useThemeMode() {
  return {
    themeMode,
    applyTheme,
    toggleThemeMode,
    initThemeMode,
    initThemeModeSafely,
  };
}
