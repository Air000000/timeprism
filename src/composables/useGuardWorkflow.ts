import { computed, ref, watch, type Ref } from "vue";
import type { IdlePrompt, PendingRuleProcess } from "../api";
import { buildGuardWorkflowState } from "../lib/guardWorkflowState";

type TranslateFn = (zh: string, en: string) => string;
type FeedbackTone = "info" | "ok" | "warn" | "error";

type UseGuardWorkflowOptions = {
  tx: TranslateFn;
  pendingRuleProcesses: Ref<PendingRuleProcess[]>;
  idlePrompts: Ref<IdlePrompt[]>;
  guardFeedback: Ref<string>;
  guardFeedbackType: Ref<FeedbackTone>;
};

export function useGuardWorkflow({
  tx,
  pendingRuleProcesses,
  idlePrompts,
  guardFeedback,
  guardFeedbackType,
}: UseGuardWorkflowOptions) {
  const guardStep3Done = ref(false);

  const guardWorkflowState = computed(() =>
    buildGuardWorkflowState(
      pendingRuleProcesses.value.length,
      idlePrompts.value.length,
      guardStep3Done.value,
    ));
  const guardStep1Complete = computed(() => guardWorkflowState.value.step1Complete);
  const guardStep2Unlocked = computed(() => guardWorkflowState.value.step2Unlocked);
  const guardStep2Complete = computed(() => guardWorkflowState.value.step2Complete);
  const guardStep3Unlocked = computed(() => guardWorkflowState.value.step3Unlocked);
  const guardStep4Unlocked = computed(() => guardWorkflowState.value.step4Unlocked);

  const guardCurrentStepText = computed(() => {
    if (!guardStep1Complete.value) {
      return tx("步骤 1/4：处理待分类软件", "Step 1/4: Process pending apps");
    }
    if (!guardStep2Complete.value) {
      return tx("步骤 2/4：确认离开时段", "Step 2/4: Resolve idle segments");
    }
    if (!guardStep3Done.value) {
      return tx("步骤 3/4：复核已有规则", "Step 3/4: Review existing rules");
    }
    return tx("步骤 4/4：查看采样诊断", "Step 4/4: Review diagnostics");
  });

  const guardCurrentStepIndex = computed(() => guardWorkflowState.value.currentStepIndex);

  const guardStepLabels = computed(() => [
    tx("待处理软件", "Pending Apps"),
    tx("离开确认", "Idle Review"),
    tx("规则复核", "Rules Review"),
    tx("采样诊断", "Diagnostics"),
  ]);

  function markGuardStep3Done() {
    if (!guardStep3Unlocked.value) {
      return;
    }
    guardStep3Done.value = true;
    guardFeedbackType.value = "ok";
    guardFeedback.value = tx("规则复核已完成，可进入采样诊断。", "Rules review completed. Diagnostics unlocked.");
  }

  watch(guardStep3Unlocked, (unlocked) => {
    if (!unlocked) {
      guardStep3Done.value = false;
    }
  });

  return {
    guardStep2Unlocked,
    guardStep3Unlocked,
    guardStep4Unlocked,
    guardStep3Done,
    guardCurrentStepText,
    guardCurrentStepIndex,
    guardStepLabels,
    markGuardStep3Done,
  };
}
