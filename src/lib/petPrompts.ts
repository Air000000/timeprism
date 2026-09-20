export type IdlePromptLite = {
  id: number;
  start_timestamp: number;
  end_timestamp: number;
  duration_ms: number;
  attribution_process_name?: string | null;
};

export type PendingRuleProcessLite = {
  process_name: string;
  total_seconds: number;
};

export type ReminderLite = {
  id: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  remind_at: number | null;
  daily_time_minutes: number | null;
  weekly_days: number[] | null;
  next_due_timestamp: number;
  done: boolean;
};

export type PromptAction = {
  label: string;
  run: () => Promise<void>;
};

export type PromptDescriptor = {
  key: string;
  title: string;
  detail: string;
  actions: PromptAction[];
};

export function pruneExpiredPromptSnoozes(snoozeUntilByKey: Map<string, number>, nowMs: number) {
  for (const [key, until] of snoozeUntilByKey.entries()) {
    if (until <= nowMs) {
      snoozeUntilByKey.delete(key);
    }
  }
}

export function availablePromptDescriptors(
  descriptors: PromptDescriptor[],
  snoozeUntilByKey: Map<string, number>,
  nowMs: number,
): PromptDescriptor[] {
  return descriptors.filter((item) => {
    const until = snoozeUntilByKey.get(item.key) ?? 0;
    return until <= nowMs;
  });
}

export function pickActivePromptDescriptor(
  descriptors: PromptDescriptor[],
  currentPromptKey: string,
): PromptDescriptor | null {
  if (descriptors.length === 0) {
    return null;
  }
  return descriptors.find((item) => item.key === currentPromptKey) ?? descriptors[0];
}
