import type { PrivacySettings } from "../api";

export type WhitelistItemInput = {
  process_name: string;
  enabled: boolean;
};

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

export function buildPrivacySettingsUpdateInput(settings: PrivacySettings): PrivacySettings {
  return {
    curtain_enabled: settings.curtain_enabled,
    browser_title_mode: settings.browser_title_mode,
    whitelist_only_enabled: settings.whitelist_only_enabled,
  };
}

export function buildWhitelistItemInput(
  processName: string,
  enabled: boolean,
): WhitelistItemInput {
  return {
    process_name: processName,
    enabled,
  };
}
