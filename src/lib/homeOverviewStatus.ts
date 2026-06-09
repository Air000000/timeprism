type TranslateFn = (zh: string, en: string) => string;

export type HomeStatusTone = "ok" | "warn" | "alert" | "idle";

export type HomeOverviewStatusCounts = {
  idlePromptCount: number;
  dueReminderCount: number;
  pendingRuleProcessCount: number;
  autoCaptureEnabled: boolean;
};

export function homeCurrentStatusLabel(
  counts: HomeOverviewStatusCounts,
  tx: TranslateFn,
): string {
  if (counts.idlePromptCount > 0) {
    return tx("离开时段待确认", "Idle segments pending");
  }
  if (counts.dueReminderCount > 0) {
    return tx(`到点提醒 ${counts.dueReminderCount} 条`, `${counts.dueReminderCount} reminders due`);
  }
  if (counts.pendingRuleProcessCount > 0) {
    return tx("待处理软件规则", "App rules pending");
  }
  if (counts.autoCaptureEnabled) {
    return tx("自动采样运行中", "Auto capture running");
  }
  return tx("自动采样已暂停", "Auto capture paused");
}

export function homeCurrentStatusTone(counts: HomeOverviewStatusCounts): HomeStatusTone {
  if (counts.idlePromptCount > 0) {
    return "alert";
  }
  if (counts.dueReminderCount > 0 || counts.pendingRuleProcessCount > 0) {
    return "warn";
  }
  if (counts.autoCaptureEnabled) {
    return "ok";
  }
  return "idle";
}

export function homePendingSummaryText(
  counts: HomeOverviewStatusCounts,
  tx: TranslateFn,
): string {
  const parts: string[] = [];
  if (counts.pendingRuleProcessCount > 0) {
    parts.push(tx(`待分类 ${counts.pendingRuleProcessCount}`, `${counts.pendingRuleProcessCount} pending apps`));
  }
  if (counts.idlePromptCount > 0) {
    parts.push(tx(`待确认 ${counts.idlePromptCount}`, `${counts.idlePromptCount} idle reviews`));
  }
  if (counts.dueReminderCount > 0) {
    parts.push(tx(`提醒 ${counts.dueReminderCount}`, `${counts.dueReminderCount} reminders`));
  }
  if (parts.length === 0) {
    return tx("当前没有新的待处理项，首页会保持安静。", "No pending items right now, so home stays quiet.");
  }
  return parts.join(" · ");
}
