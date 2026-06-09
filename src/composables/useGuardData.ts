import { computed, ref } from "vue";
import {
  getIdleMemoryState,
  listAppRules,
  listForegroundCaptureDiagnostics,
  listPendingIdlePrompts,
  listPendingRuleProcesses,
  resolveIdlePrompt,
  saveAppRule,
  type AppRule,
  type ForegroundCaptureDiagnostic,
  type IdleMemoryState,
  type IdlePrompt,
  type PendingRuleProcess,
} from "../api";
import {
  canSaveRuleFromDiagnostic,
  guardCaptureBlockReasonText,
  guardCaptureRuleText,
} from "../lib/guardDiagnostics";
import {
  guardIdleDecisionFeedback,
  type GuardIdleDecision as IdleDecision,
} from "../lib/guardIdle";
import {
  filterSortedGuardRules,
  isGuardRuleMappedType,
  isGuardRuleSortKey,
  type GuardRuleMappedType as RuleMappedType,
  type GuardRuleSortKey as RuleSortKey,
} from "../lib/guardRules";

type TranslateFn = (zh: string, en: string) => string;
type FeedbackTone = "info" | "ok" | "warn" | "error";

type UseGuardDataOptions = {
  tx: TranslateFn;
  mappedTypeText: (mappedType: RuleMappedType) => string;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

export function useGuardData({
  tx,
  mappedTypeText,
  refreshData,
  setErrorMessage,
}: UseGuardDataOptions) {
  const loadingGuard = ref(false);
  const autoCaptureEnabled = ref(true);
  const autoCaptureFeedback = ref("自动采样已开启");
  const guardFeedback = ref("尚未执行检测");
  const guardFeedbackType = ref<FeedbackTone>("info");
  const foregroundDiagnostics = ref<ForegroundCaptureDiagnostic[]>([]);
  const appRules = ref<AppRule[]>([]);
  const pendingRuleProcesses = ref<PendingRuleProcess[]>([]);
  const idlePrompts = ref<IdlePrompt[]>([]);
  const idleActionLoading = ref(false);
  const idleRememberChoice = ref(false);
  const idleMemoryState = ref<IdleMemoryState>({ remembered_decision: null });
  const ruleSearch = ref("");
  const ruleSort = ref<RuleSortKey>("alpha_asc");

  const filteredSortedRules = computed(() =>
    filterSortedGuardRules(appRules.value, ruleSearch.value, ruleSort.value));

  const currentIdlePrompt = computed(() => idlePrompts.value[0] ?? null);

  function resetGuardFeedback() {
    autoCaptureFeedback.value = tx("自动采样已开启", "Auto capture enabled");
    guardFeedback.value = tx("尚未执行检测", "No check executed yet");
  }

  async function refreshGuardData() {
    if (loadingGuard.value) {
      return;
    }

    loadingGuard.value = true;
    try {
      const [diagnostics, rules, pendingRules, pendingIdle, idleMemory] = await Promise.all([
        listForegroundCaptureDiagnostics(10, true),
        listAppRules(300),
        listPendingRuleProcesses(10),
        listPendingIdlePrompts(3),
        getIdleMemoryState(),
      ]);
      foregroundDiagnostics.value = diagnostics;
      appRules.value = rules;
      pendingRuleProcesses.value = pendingRules;
      idlePrompts.value = pendingIdle;
      idleMemoryState.value = idleMemory;
    } catch (e) {
      setErrorMessage(e);
    } finally {
      loadingGuard.value = false;
    }
  }

  async function handleResolveIdle(
    decision: IdleDecision,
    promptId?: number,
  ) {
    const prompt = promptId
      ? idlePrompts.value.find((item) => item.id === promptId) ?? null
      : currentIdlePrompt.value;
    if (!prompt || idleActionLoading.value) {
      return;
    }

    idleActionLoading.value = true;
    try {
      const rememberChoice = idleRememberChoice.value && decision !== "SKIP";
      await resolveIdlePrompt({
        prompt_id: prompt.id,
        decision,
        remember_this_session: rememberChoice,
      });

      const feedback = guardIdleDecisionFeedback(decision, rememberChoice, tx);
      guardFeedbackType.value = feedback.type;
      guardFeedback.value = feedback.text;

      idleRememberChoice.value = false;
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      guardFeedbackType.value = "error";
      guardFeedback.value = tx(`空闲时段分类失败：${e}`, `Failed to classify idle segment: ${e}`);
    } finally {
      idleActionLoading.value = false;
    }
  }

  const captureBlockReasonText = (reason: string | null): string =>
    guardCaptureBlockReasonText(reason, tx);

  const captureRuleText = (item: ForegroundCaptureDiagnostic): string =>
    guardCaptureRuleText(item, tx, mappedTypeText);

  async function handleSaveRuleFromDiagnostic(
    item: ForegroundCaptureDiagnostic,
    mappedType: RuleMappedType,
  ) {
    if (!canSaveRuleFromDiagnostic(item)) {
      return;
    }
    try {
      await saveAppRule({
        process_name: item.observed_process_name,
        mapped_type: mappedType,
        privacy_level: "NORMAL",
      });
      guardFeedbackType.value = "ok";
      guardFeedback.value = tx(
        `已保存规则：${item.observed_process_name} -> ${mappedTypeText(mappedType)}`,
        `Rule saved: ${item.observed_process_name} -> ${mappedTypeText(mappedType)}`,
      );
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      guardFeedbackType.value = "error";
      guardFeedback.value = tx(`保存规则失败：${e}`, `Failed to save rule: ${e}`);
    }
  }

  async function handleUpdateExistingRule(rule: AppRule) {
    try {
      await saveAppRule({
        process_name: rule.process_name,
        mapped_type: rule.mapped_type,
        privacy_level: rule.privacy_level,
      });
      guardFeedbackType.value = "ok";
      guardFeedback.value = tx(
        `已更新规则：${rule.process_name} -> ${mappedTypeText(rule.mapped_type)}`,
        `Rule updated: ${rule.process_name} -> ${mappedTypeText(rule.mapped_type)}`,
      );
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      guardFeedbackType.value = "error";
      guardFeedback.value = tx(`更新规则失败：${e}`, `Failed to update rule: ${e}`);
    }
  }

  async function handleSavePendingRule(
    item: PendingRuleProcess,
    mappedType: RuleMappedType,
  ) {
    try {
      await saveAppRule({
        process_name: item.process_name,
        mapped_type: mappedType,
        privacy_level: "NORMAL",
      });
      guardFeedbackType.value = "ok";
      guardFeedback.value = tx(
        `已保存规则：${item.process_name} -> ${mappedTypeText(mappedType)}`,
        `Rule saved: ${item.process_name} -> ${mappedTypeText(mappedType)}`,
      );
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      guardFeedbackType.value = "error";
      guardFeedback.value = tx(`保存规则失败：${e}`, `Failed to save rule: ${e}`);
    }
  }

  function onAutoCaptureToggle(event: Event) {
    const input = event.target as HTMLInputElement;
    autoCaptureEnabled.value = input.checked;
  }

  function onIdleRememberChoiceChange(event: Event) {
    const input = event.target as HTMLInputElement;
    idleRememberChoice.value = input.checked;
  }

  function onRuleSearchInput(event: Event) {
    const input = event.target as HTMLInputElement;
    ruleSearch.value = input.value;
  }

  function onRuleSortChange(event: Event) {
    const input = event.target as HTMLSelectElement;
    if (isGuardRuleSortKey(input.value)) {
      ruleSort.value = input.value;
    }
  }

  function onRuleMappedTypeChange(rule: AppRule, event: Event) {
    const input = event.target as HTMLSelectElement;
    if (isGuardRuleMappedType(input.value)) {
      rule.mapped_type = input.value;
    }
  }

  return {
    autoCaptureEnabled,
    autoCaptureFeedback,
    guardFeedback,
    guardFeedbackType,
    foregroundDiagnostics,
    pendingRuleProcesses,
    idlePrompts,
    idleActionLoading,
    idleRememberChoice,
    ruleSearch,
    ruleSort,
    filteredSortedRules,
    currentIdlePrompt,
    resetGuardFeedback,
    refreshGuardData,
    handleResolveIdle,
    captureBlockReasonText,
    captureRuleText,
    canSaveRuleFromDiagnostic,
    handleSaveRuleFromDiagnostic,
    handleUpdateExistingRule,
    handleSavePendingRule,
    onAutoCaptureToggle,
    onIdleRememberChoiceChange,
    onRuleSearchInput,
    onRuleSortChange,
    onRuleMappedTypeChange,
  };
}
