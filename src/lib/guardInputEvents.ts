import {
  isGuardRuleMappedType,
  isGuardRuleSortKey,
  type GuardRuleMappedType,
  type GuardRuleSortKey,
} from "./guardRules";

export function guardCheckedFromEvent(event: Event): boolean {
  return (event.target as HTMLInputElement).checked;
}

export function guardTextValueFromEvent(event: Event): string {
  return (event.target as HTMLInputElement).value;
}

export function guardRuleSortKeyFromEvent(event: Event): GuardRuleSortKey | null {
  const value = (event.target as HTMLSelectElement).value;
  return isGuardRuleSortKey(value) ? value : null;
}

export function guardRuleMappedTypeFromEvent(event: Event): GuardRuleMappedType | null {
  const value = (event.target as HTMLSelectElement).value;
  return isGuardRuleMappedType(value) ? value : null;
}
