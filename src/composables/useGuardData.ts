import { computed, ref, type Ref } from "vue";
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
  defaultGuardAutoCaptureFeedback,
  defaultGuardCheckFeedback,
  guardIdleResolveErrorFeedback,
  guardRuleSavedFeedback,
  guardRuleSaveErrorFeedback,
  guardRuleUpdatedFeedback,
} from "../lib/guardFeedback";
import {
  guardIdleDecisionFeedback,
  selectGuardIdlePrompt,
  shouldRememberGuardIdleDecision,
  type GuardIdleDecision as IdleDecision,
} from "../lib/guardIdle";
import {
  guardCheckedFromEvent,
  guardRuleMappedTypeFromEvent,
  guardRuleSortKeyFromEvent,
  guardTextValueFromEvent,
} from "../lib/guardInputEvents";
import {
  buildExistingGuardRuleSaveInput,
  buildNormalGuardRuleSaveInput,
} from "../lib/guardRuleSave";
import {
  filterSortedGuardRules,
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
  autoCaptureEnabled: Readonly<Ref<boolean>>;
  persistAutoCaptureEnabled: (enabled: boolean) => Promise<void>;
};

export function useGuardData({
  tx,
  mappedTypeText,
  refreshData,
  setErrorMessage,
  autoCaptureEnabled,
  persistAutoCaptureEnabled,
}: UseGuardDataOptions) {
  const loadingGuard = ref(false);
  const autoCaptureFeedback = ref(defaultGuardAutoCaptureFeedback(tx));
  const guardFeedback = ref(defaultGuardCheckFeedback(tx));
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
    autoCaptureFeedback.value = defaultGuardAutoCaptureFeedback(tx);
    guardFeedback.value = defaultGuardCheckFeedback(tx);
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
    const prompt = selectGuardIdlePrompt(idlePrompts.value, currentIdlePrompt.value, promptId);
    if (!prompt || idleActionLoading.value) {
      return;
    }

    idleActionLoading.value = true;
    try {
      const rememberChoice = shouldRememberGuardIdleDecision(idleRememberChoice.value, decision);
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
      guardFeedback.value = guardIdleResolveErrorFeedback(e, tx);
    } finally {
      idleActionLoading.value = false;
    }
  }

  const captureBlockReasonText = (reason: string | null): string =>
    guardCaptureBlockReasonText(reason, tx);

  const captureRuleText = (item: ForegroundCaptureDiagnostic): string =>
    guardCaptureRuleText(item, tx, mappedTypeText);

  async function runGuardRuleSave(
    task: () => Promise<void>,
    buildSuccessFeedback: () => string,
  ) {
    try {
      await task();
      guardFeedbackType.value = "ok";
      guardFeedback.value = buildSuccessFeedback();
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      guardFeedbackType.value = "error";
      guardFeedback.value = guardRuleSaveErrorFeedback(e, tx);
    }
  }

  async function handleSaveRuleFromDiagnostic(
    item: ForegroundCaptureDiagnostic,
    mappedType: RuleMappedType,
  ) {
    if (!canSaveRuleFromDiagnostic(item)) {
      return;
    }
    await runGuardRuleSave(
      () => saveAppRule(buildNormalGuardRuleSaveInput(item.observed_process_name, mappedType)),
      () =>
        guardRuleSavedFeedback(
          item.observed_process_name,
          mappedTypeText(mappedType),
          tx,
        ),
    );
  }

  async function handleUpdateExistingRule(rule: AppRule) {
    await runGuardRuleSave(
      () => saveAppRule(buildExistingGuardRuleSaveInput(rule)),
      () =>
        guardRuleUpdatedFeedback(
          rule.process_name,
          mappedTypeText(rule.mapped_type),
          tx,
        ),
    );
  }

  async function handleSavePendingRule(
    item: PendingRuleProcess,
    mappedType: RuleMappedType,
  ) {
    await runGuardRuleSave(
      () => saveAppRule(buildNormalGuardRuleSaveInput(item.process_name, mappedType)),
      () =>
        guardRuleSavedFeedback(
          item.process_name,
          mappedTypeText(mappedType),
          tx,
        ),
    );
  }

  async function onAutoCaptureToggle(event: Event): Promise<void> {
    const enabled = guardCheckedFromEvent(event);
    try {
      await persistAutoCaptureEnabled(enabled);
      autoCaptureFeedback.value = enabled
        ? tx("自动采样已恢复。", "Auto capture resumed.")
        : tx("自动采样已暂停。", "Auto capture paused.");
    } catch (e) {
      setErrorMessage(e);
      autoCaptureFeedback.value = tx(
        `切换自动采样失败：${e}`,
        `Failed to change auto capture: ${e}`,
      );
    }
  }

  function onIdleRememberChoiceChange(event: Event) {
    idleRememberChoice.value = guardCheckedFromEvent(event);
  }

  function onRuleSearchInput(event: Event) {
    ruleSearch.value = guardTextValueFromEvent(event);
  }

  function onRuleSortChange(event: Event) {
    const sortKey = guardRuleSortKeyFromEvent(event);
    if (sortKey) {
      ruleSort.value = sortKey;
    }
  }

  function onRuleMappedTypeChange(rule: AppRule, event: Event) {
    const mappedType = guardRuleMappedTypeFromEvent(event);
    if (mappedType) {
      rule.mapped_type = mappedType;
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
