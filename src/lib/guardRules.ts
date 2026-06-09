import type { AppRule, AppRuleMappedType } from "../api";

export type GuardRuleMappedType = AppRuleMappedType;
export type GuardRuleSortKey = "alpha_asc" | "alpha_desc" | "time_desc" | "time_asc";

export function filterSortedGuardRules(
  rules: AppRule[],
  query: string,
  sortKey: GuardRuleSortKey,
): AppRule[] {
  const q = query.trim().toLowerCase();
  const list = q
    ? rules.filter((rule) => rule.process_name.toLowerCase().includes(q))
    : rules;

  const sorted = [...list];
  if (sortKey === "alpha_asc") {
    sorted.sort((a, b) => a.process_name.localeCompare(b.process_name));
  } else if (sortKey === "alpha_desc") {
    sorted.sort((a, b) => b.process_name.localeCompare(a.process_name));
  } else if (sortKey === "time_desc") {
    sorted.sort((a, b) => b.updated_at - a.updated_at);
  } else {
    sorted.sort((a, b) => a.updated_at - b.updated_at);
  }
  return sorted;
}

export function isGuardRuleSortKey(value: string): value is GuardRuleSortKey {
  return value === "alpha_asc"
    || value === "alpha_desc"
    || value === "time_desc"
    || value === "time_asc";
}

export function isGuardRuleMappedType(value: string): value is GuardRuleMappedType {
  return value === "LEARN" || value === "REST" || value === "IGNORE";
}
