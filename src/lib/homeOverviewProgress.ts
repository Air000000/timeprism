export type HomeGoalOverflowTier = "none" | "active";

export type HomeGoalProgress = {
  fillPercent: number;
  overflowTier: HomeGoalOverflowTier;
  percent: number;
  percentText: string;
  ratio: number;
};

export function buildHomeGoalProgress(
  learnSeconds: number,
  goalMinutes: number,
): HomeGoalProgress {
  const goal = Math.max(0, goalMinutes * 60);
  const ratio = goal <= 0 ? 0 : Math.max(0, learnSeconds / goal);
  const percent = Math.round(ratio * 100);
  return {
    fillPercent: Math.max(0, Math.min(100, percent)),
    overflowTier: percent > 100 ? "active" : "none",
    percent,
    percentText: `${(ratio * 100).toFixed(0)}%`,
    ratio,
  };
}
