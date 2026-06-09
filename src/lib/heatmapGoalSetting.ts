const DEFAULT_HEATMAP_GOAL_MINUTES = 120;
const HEATMAP_GOAL_STEP_MINUTES = 15;
const MAX_HEATMAP_GOAL_MINUTES = 1440;

export function normalizeHeatmapGoalMinutes(minutes: number): number {
  const rounded = Number.isFinite(minutes)
    ? Math.round(minutes / HEATMAP_GOAL_STEP_MINUTES) * HEATMAP_GOAL_STEP_MINUTES
    : DEFAULT_HEATMAP_GOAL_MINUTES;
  return Math.min(MAX_HEATMAP_GOAL_MINUTES, Math.max(0, rounded));
}

export function heatmapGoalSecondsToSliderMinutes(seconds: number): number {
  const minutes = Math.round(seconds / 60 / HEATMAP_GOAL_STEP_MINUTES) * HEATMAP_GOAL_STEP_MINUTES;
  return normalizeHeatmapGoalMinutes(minutes);
}

export function heatmapGoalMinutesToSeconds(minutes: number): number {
  return normalizeHeatmapGoalMinutes(minutes) * 60;
}
