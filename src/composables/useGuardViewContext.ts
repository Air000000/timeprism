import { computed, type Ref } from "vue";
import type {
  GuardViewContext,
  TranslateFn,
} from "../components/viewContexts";

type GuardViewContextOptions = {
  tx: TranslateFn;
  guardCurrentStepText: Readonly<Ref<string>>;
  guardStepLabels: Readonly<Ref<string[]>>;
  guardCurrentStepIndex: Readonly<Ref<number>>;
  autoCaptureEnabled: Readonly<Ref<boolean>>;
  onAutoCaptureToggle: GuardViewContext["onAutoCaptureToggle"];
  pendingRuleProcesses: Readonly<Ref<GuardViewContext["pendingRuleProcesses"]>>;
  cleanProcessName: GuardViewContext["cleanProcessName"];
  formatClock: GuardViewContext["formatClock"];
  formatSeconds: GuardViewContext["formatSeconds"];
  handleSavePendingRule: GuardViewContext["handleSavePendingRule"];
  guardStep2Unlocked: Readonly<Ref<boolean>>;
  idlePrompts: Readonly<Ref<GuardViewContext["idlePrompts"]>>;
  formatIdlePromptSpan: GuardViewContext["formatIdlePromptSpan"];
  idleActionLoading: Readonly<Ref<boolean>>;
  handleResolveIdle: GuardViewContext["handleResolveIdle"];
  guardStep3Unlocked: Readonly<Ref<boolean>>;
  ruleSearch: Readonly<Ref<string>>;
  onRuleSearchInput: GuardViewContext["onRuleSearchInput"];
  ruleSort: Readonly<Ref<GuardViewContext["ruleSort"]>>;
  onRuleSortChange: GuardViewContext["onRuleSortChange"];
  filteredSortedRules: Readonly<Ref<GuardViewContext["filteredSortedRules"]>>;
  onRuleMappedTypeChange: GuardViewContext["onRuleMappedTypeChange"];
  handleUpdateExistingRule: GuardViewContext["handleUpdateExistingRule"];
  markGuardStep3Done: GuardViewContext["markGuardStep3Done"];
  guardStep3Done: Readonly<Ref<boolean>>;
  guardStep4Unlocked: Readonly<Ref<boolean>>;
  foregroundDiagnostics: Readonly<Ref<GuardViewContext["foregroundDiagnostics"]>>;
  captureBlockReasonText: GuardViewContext["captureBlockReasonText"];
  captureRuleText: GuardViewContext["captureRuleText"];
  canSaveRuleFromDiagnostic: GuardViewContext["canSaveRuleFromDiagnostic"];
  handleSaveRuleFromDiagnostic: GuardViewContext["handleSaveRuleFromDiagnostic"];
};

export function useGuardViewContext({
  tx,
  guardCurrentStepText,
  guardStepLabels,
  guardCurrentStepIndex,
  autoCaptureEnabled,
  onAutoCaptureToggle,
  pendingRuleProcesses,
  cleanProcessName,
  formatClock,
  formatSeconds,
  handleSavePendingRule,
  guardStep2Unlocked,
  idlePrompts,
  formatIdlePromptSpan,
  idleActionLoading,
  handleResolveIdle,
  guardStep3Unlocked,
  ruleSearch,
  onRuleSearchInput,
  ruleSort,
  onRuleSortChange,
  filteredSortedRules,
  onRuleMappedTypeChange,
  handleUpdateExistingRule,
  markGuardStep3Done,
  guardStep3Done,
  guardStep4Unlocked,
  foregroundDiagnostics,
  captureBlockReasonText,
  captureRuleText,
  canSaveRuleFromDiagnostic,
  handleSaveRuleFromDiagnostic,
}: GuardViewContextOptions) {
  return computed<GuardViewContext>(() => ({
    tx,
    guardCurrentStepText: guardCurrentStepText.value,
    guardStepLabels: guardStepLabels.value,
    guardCurrentStepIndex: guardCurrentStepIndex.value,
    autoCaptureEnabled: autoCaptureEnabled.value,
    onAutoCaptureToggle,
    pendingRuleProcesses: pendingRuleProcesses.value,
    cleanProcessName,
    formatClock,
    formatSeconds,
    handleSavePendingRule,
    guardStep2Unlocked: guardStep2Unlocked.value,
    idlePrompts: idlePrompts.value,
    formatIdlePromptSpan,
    idleActionLoading: idleActionLoading.value,
    handleResolveIdle,
    guardStep3Unlocked: guardStep3Unlocked.value,
    ruleSearch: ruleSearch.value,
    onRuleSearchInput,
    ruleSort: ruleSort.value,
    onRuleSortChange,
    filteredSortedRules: filteredSortedRules.value,
    onRuleMappedTypeChange,
    handleUpdateExistingRule,
    markGuardStep3Done,
    guardStep3Done: guardStep3Done.value,
    guardStep4Unlocked: guardStep4Unlocked.value,
    foregroundDiagnostics: foregroundDiagnostics.value,
    captureBlockReasonText,
    captureRuleText,
    canSaveRuleFromDiagnostic,
    handleSaveRuleFromDiagnostic,
  }));
}
