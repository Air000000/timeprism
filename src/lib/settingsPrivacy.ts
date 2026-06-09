import type { PrivacySettings } from "../api";

export function defaultPrivacySettings(): PrivacySettings {
  return {
    curtain_enabled: false,
    browser_title_mode: "BLUR",
    whitelist_only_enabled: false,
  };
}

export function normalizeWhitelistProcessName(input: string): string {
  return input.trim().toLowerCase();
}
