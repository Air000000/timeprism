import type { IdlePrompt } from "../api";

type TranslateFn = (zh: string, en: string) => string;
type GuardFeedbackTone = "info" | "ok" | "warn" | "error";

export type GuardIdleDecision = "LEARN" | "REST" | "IDLE" | "SKIP";

export type GuardIdleDecisionFeedback = {
  type: GuardFeedbackTone;
  text: string;
};

export function selectGuardIdlePrompt(
  prompts: IdlePrompt[],
  currentPrompt: IdlePrompt | null,
  promptId?: number,
): IdlePrompt | null {
  return promptId
    ? prompts.find((item) => item.id === promptId) ?? null
    : currentPrompt;
}

export function shouldRememberGuardIdleDecision(
  rememberChoice: boolean,
  decision: GuardIdleDecision,
): boolean {
  return rememberChoice && decision !== "SKIP";
}

export function guardIdleDecisionFeedback(
  decision: GuardIdleDecision,
  rememberChoice: boolean,
  tx: TranslateFn,
): GuardIdleDecisionFeedback {
  if (decision === "LEARN") {
    return {
      type: "ok",
      text: rememberChoice
        ? tx("已将该空闲时段归类为学习，并记忆本次选择。", "This idle segment is marked as Learn and remembered.")
        : tx("已将该空闲时段归类为学习。", "This idle segment is marked as Learn."),
    };
  }

  if (decision === "REST") {
    return {
      type: "ok",
      text: rememberChoice
        ? tx("已将该空闲时段归类为休息，并记忆本次选择。", "This idle segment is marked as Break and remembered.")
        : tx("已将该空闲时段归类为休息。", "This idle segment is marked as Break."),
    };
  }

  if (decision === "IDLE") {
    return {
      type: "info",
      text: rememberChoice
        ? tx("已将该空闲时段归类为离开，并记忆本次选择。", "This idle segment is marked as Away and remembered.")
        : tx("已将该空闲时段归类为离开（不计入学习/休息）。", "This idle segment is marked as Away (excluded from Learn/Break)."),
    };
  }

  return {
    type: "warn",
    text: tx("该空闲时段已暂缓，后续将继续采样。", "This idle segment is postponed. Sampling will continue."),
  };
}
