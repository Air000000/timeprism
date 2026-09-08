export type GuardWorkflowState = {
  currentStepIndex: 1 | 2 | 3 | 4;
  step1Complete: boolean;
  step2Complete: boolean;
  step2Unlocked: boolean;
  step3Unlocked: boolean;
  step4Unlocked: boolean;
};

export function buildGuardWorkflowState(
  pendingRuleProcessCount: number,
  idlePromptCount: number,
  step3Done: boolean,
): GuardWorkflowState {
  const step1Complete = pendingRuleProcessCount === 0;
  const step2Unlocked = step1Complete;
  const step2Complete = idlePromptCount === 0;
  const step3Unlocked = step1Complete && step2Complete;
  const step4Unlocked = step3Unlocked && step3Done;

  let currentStepIndex: GuardWorkflowState["currentStepIndex"] = 4;
  if (!step1Complete) {
    currentStepIndex = 1;
  } else if (!step2Complete) {
    currentStepIndex = 2;
  } else if (!step3Done) {
    currentStepIndex = 3;
  }

  return {
    currentStepIndex,
    step1Complete,
    step2Complete,
    step2Unlocked,
    step3Unlocked,
    step4Unlocked,
  };
}
