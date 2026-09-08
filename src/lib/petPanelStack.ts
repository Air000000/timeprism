import type { UsageStackDay } from "../api";

type TranslateFn = (zh: string, en: string) => string;

export type PetPanelStackPart = {
  name: string;
  seconds: number;
  color: string;
};

export function petPanelStackSignature(day: UsageStackDay): string {
  return `${day.day}|${day.total_seconds}|${day.learn_seconds}|${day.rest_seconds}`;
}

export function buildPetPanelStackParts(day: UsageStackDay, tx: TranslateFn): PetPanelStackPart[] {
  const ignore = Math.max(0, day.total_seconds - day.learn_seconds - day.rest_seconds);
  return [
    { name: tx("学", "L"), seconds: day.learn_seconds, color: "#16a34a" },
    { name: tx("休", "B"), seconds: day.rest_seconds, color: "#8ec5ff" },
    { name: tx("未", "U"), seconds: ignore, color: "#64748b" },
  ].filter((item) => item.seconds > 0);
}

export function petPanelStackPercent(seconds: number, totalSeconds: number): number {
  return totalSeconds > 0 ? (seconds / totalSeconds) * 100 : 0;
}
