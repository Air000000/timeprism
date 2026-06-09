import { invokeCommand } from "../client";
import type { PrivacySettings } from "../types";

export async function getPrivacySettings(): Promise<PrivacySettings> {
  return invokeCommand("get_privacy_settings");
}

export async function updatePrivacySettings(input: {
  curtain_enabled: boolean;
  browser_title_mode: "FULL" | "BLUR" | "NONE";
  whitelist_only_enabled: boolean;
}): Promise<void> {
  return invokeCommand("update_privacy_settings", {
    input: {
      curtain_enabled: input.curtain_enabled,
      browser_title_mode: input.browser_title_mode,
      whitelist_only_enabled: input.whitelist_only_enabled,
    },
  });
}

export async function listWhitelist(): Promise<string[]> {
  return invokeCommand("list_whitelist");
}

export async function setWhitelistItem(input: {
  process_name: string;
  enabled: boolean;
}): Promise<void> {
  return invokeCommand("set_whitelist_item", {
    input: {
      process_name: input.process_name,
      enabled: input.enabled,
    },
  });
}
