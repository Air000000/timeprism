import { invokeCommand } from "../client";
import type { AppRule, DeviationCheck, PendingRuleProcess } from "../types";

export async function saveAppRule(input: {
  process_name: string;
  mapped_type: "LEARN" | "REST" | "IGNORE";
  privacy_level?: "NORMAL" | "BLUR_TITLE" | "WHITELIST_ONLY";
}): Promise<void> {
  return invokeCommand("save_app_rule", { input });
}

export async function checkFocusDeviation(input: {
  process_name: string;
  debounce_seconds?: number;
}): Promise<DeviationCheck> {
  return invokeCommand("check_focus_deviation", {
    processName: input.process_name,
    debounceSeconds: input.debounce_seconds,
  });
}

export async function snoozeFocusGuard(cooldown_seconds = 900): Promise<void> {
  return invokeCommand("snooze_focus_guard", { cooldownSeconds: cooldown_seconds });
}

export async function listAppRules(limit = 200): Promise<AppRule[]> {
  return invokeCommand("list_app_rules", { limit });
}

export async function listPendingRuleProcesses(limit = 10): Promise<PendingRuleProcess[]> {
  return invokeCommand("list_pending_rule_processes", { limit });
}
