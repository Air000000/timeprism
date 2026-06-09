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

type TranslateFn = (zh: string, en: string) => string;
type FeedbackTone = "info" | "ok" | "warn" | "error";
type RuleMappedType = "LEARN" | "REST" | "IGNORE";
type IdleDecision = "LEARN" | "REST" | "IDLE" | "SKIP";
type RuleSortKey = "alpha_asc" | "alpha_desc" | "time_desc" | "time_asc";

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

  const filteredSortedRules = computed(() => {
    const q = ruleSearch.value.trim().toLowerCase();
    let list = appRules.value;
    if (q) {
      list = list.filter((rule) => rule.process_name.toLowerCase().includes(q));
    }

    const sorted = [...list];
    if (ruleSort.value === "alpha_asc") {
      sorted.sort((a, b) => a.process_name.localeCompare(b.process_name));
    } else if (ruleSort.value === "alpha_desc") {
      sorted.sort((a, b) => b.process_name.localeCompare(a.process_name));
    } else if (ruleSort.value === "time_desc") {
      sorted.sort((a, b) => b.updated_at - a.updated_at);
    } else {
      sorted.sort((a, b) => a.updated_at - b.updated_at);
    }
    return sorted;
  });

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

      if (decision === "LEARN") {
        guardFeedbackType.value = "ok";
        guardFeedback.value = rememberChoice
          ? tx("已将该空闲时段归类为学习，并记忆本次选择。", "This idle segment is marked as Learn and remembered.")
          : tx("已将该空闲时段归类为学习。", "This idle segment is marked as Learn.");
      } else if (decision === "REST") {
        guardFeedbackType.value = "ok";
        guardFeedback.value = rememberChoice
          ? tx("已将该空闲时段归类为休息，并记忆本次选择。", "This idle segment is marked as Break and remembered.")
          : tx("已将该空闲时段归类为休息。", "This idle segment is marked as Break.");
      } else if (decision === "IDLE") {
        guardFeedbackType.value = "info";
        guardFeedback.value = rememberChoice
          ? tx("已将该空闲时段归类为离开，并记忆本次选择。", "This idle segment is marked as Away and remembered.")
          : tx("已将该空闲时段归类为离开（不计入学习/休息）。", "This idle segment is marked as Away (excluded from Learn/Break).");
      } else {
        guardFeedbackType.value = "warn";
        guardFeedback.value = tx("该空闲时段已暂缓，后续将继续采样。", "This idle segment is postponed. Sampling will continue.");
      }

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

  function captureBlockReasonText(reason: string | null): string {
    if (!reason) {
      return "-";
    }
    if (reason === "baseline_only") {
      return tx("首次采样仅建立基线", "First sample only sets baseline");
    }
    if (reason === "curtain_enabled") {
      return tx("被窗帘模式拦截", "Blocked by Curtain mode");
    }
    if (reason === "incognito_window") {
      return tx("无痕/隐私窗口拦截", "Blocked by incognito/private window");
    }
    if (reason === "whitelist_blocked") {
      return tx("白名单策略降级", "Whitelisted-only policy fallback");
    }
    if (reason === "no_foreground_window") {
      return tx("未获取到前台窗口", "No foreground window detected");
    }
    if (reason === "elapsed_too_short") {
      return tx("采样间隔过短", "Sampling interval too short");
    }
    return reason;
  }

  function captureRuleText(item: ForegroundCaptureDiagnostic): string {
    if (!item.rule_saved) {
      return tx("未分类（规则未保存）", "Unclassified (rule not saved)");
    }
    return mappedTypeText(item.rule_mapped_type);
  }

  function canSaveRuleFromDiagnostic(item: ForegroundCaptureDiagnostic): boolean {
    return item.observed_process_name !== "unknown.exe";
  }

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

  function onRuleSearchInput(event: Event) {
    const input = event.target as HTMLInputElement;
    ruleSearch.value = input.value;
  }

  function onRuleSortChange(event: Event) {
    const input = event.target as HTMLSelectElement;
    if (input.value === "alpha_asc" || input.value === "alpha_desc" || input.value === "time_desc" || input.value === "time_asc") {
      ruleSort.value = input.value;
    }
  }

  function onRuleMappedTypeChange(rule: AppRule, event: Event) {
    const input = event.target as HTMLSelectElement;
    if (input.value === "LEARN" || input.value === "REST" || input.value === "IGNORE") {
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
    onRuleSearchInput,
    onRuleSortChange,
    onRuleMappedTypeChange,
  };
}
