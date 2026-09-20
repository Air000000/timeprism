import {
  guardIdleDecisionFeedback,
  shouldRememberGuardIdleDecision,
  type GuardIdleDecision,
} from "./guardIdle";

const decision: GuardIdleDecision = "APP";
export const rememberAppAttributionContract: boolean = shouldRememberGuardIdleDecision(true, decision);
export const appAttributionFeedbackContract: string = guardIdleDecisionFeedback(
  decision,
  false,
  (_zh, en) => en,
).text;
