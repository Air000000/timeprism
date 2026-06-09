import type { AppRule, AppRuleMappedType, AppRulePrivacyLevel } from "../api";

export type GuardRuleSaveInput = {
  process_name: string;
  mapped_type: AppRuleMappedType;
  privacy_level?: AppRulePrivacyLevel;
};

export function buildNormalGuardRuleSaveInput(
  processName: string,
  mappedType: AppRuleMappedType,
): GuardRuleSaveInput {
  return {
    process_name: processName,
    mapped_type: mappedType,
    privacy_level: "NORMAL",
  };
}

export function buildExistingGuardRuleSaveInput(rule: AppRule): GuardRuleSaveInput {
  return {
    process_name: rule.process_name,
    mapped_type: rule.mapped_type,
    privacy_level: rule.privacy_level,
  };
}
