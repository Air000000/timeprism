import type { AppRuleMappedType } from "../api";
import type {
  IdlePromptLite,
  PendingRuleProcessLite,
  PromptDescriptor,
  ReminderLite,
} from "./petPrompts";

type TranslateFn = (zh: string, en: string) => string;
type IdleDecision = "LEARN" | "REST" | "IDLE" | "SKIP";
type RuleMappedType = AppRuleMappedType;

type PetPromptDescriptorActions = {
  completeReminder: (reminder: ReminderLite) => Promise<void>;
  snoozeReminder: (reminder: ReminderLite) => Promise<void>;
  resolveIdle: (idle: IdlePromptLite, decision: IdleDecision, promptKey: string) => Promise<void>;
  saveRule: (processName: string, mappedType: RuleMappedType) => Promise<void>;
  postponeRule: (promptKey: string) => Promise<void>;
};

type BuildPetPromptDescriptorsOptions = {
  dueReminders: ReminderLite[];
  idleItems: IdlePromptLite[];
  pendingItems: PendingRuleProcessLite[];
  tx: TranslateFn;
  cleanProcessName: (processName: string) => string;
  formatReminderDueText: (reminder: ReminderLite) => string;
  formatSeconds: (seconds: number) => string;
  actions: PetPromptDescriptorActions;
};

export function buildPetPromptDescriptors({
  dueReminders,
  idleItems,
  pendingItems,
  tx,
  cleanProcessName,
  formatReminderDueText,
  formatSeconds,
  actions,
}: BuildPetPromptDescriptorsOptions): PromptDescriptor[] {
  const descriptors: PromptDescriptor[] = [];

  for (const reminder of dueReminders) {
    descriptors.push({
      key: `reminder-${reminder.id}`,
      title: tx("日程提醒", "Reminder Due"),
      detail: `${reminder.content} · ${formatReminderDueText(reminder)}`,
      actions: [
        {
          label: tx("完成", "Done"),
          run: () => actions.completeReminder(reminder),
        },
        {
          label: tx("稍后10分钟", "Snooze 10m"),
          run: () => actions.snoozeReminder(reminder),
        },
      ],
    });
  }

  for (const idle of idleItems) {
    const idleSeconds = Math.max(0, Math.floor(idle.duration_ms / 1000));
    const key = `idle-${idle.id}`;
    descriptors.push({
      key,
      title: tx("离开时段待确认", "Idle Segment Confirmation"),
      detail: tx(
        `持续 ${formatSeconds(idleSeconds)}，请尽快归类`,
        `${formatSeconds(idleSeconds)} idle time, please classify`,
      ),
      actions: [
        {
          label: tx("学习", "Learn"),
          run: () => actions.resolveIdle(idle, "LEARN", key),
        },
        {
          label: tx("休息", "Break"),
          run: () => actions.resolveIdle(idle, "REST", key),
        },
        {
          label: tx("离开", "Away"),
          run: () => actions.resolveIdle(idle, "IDLE", key),
        },
        {
          label: tx("稍后提醒", "Remind later"),
          run: () => actions.resolveIdle(idle, "SKIP", key),
        },
      ],
    });
  }

  for (const pending of pendingItems) {
    const process = pending.process_name;
    const key = `rule-${process}`;
    descriptors.push({
      key,
      title: tx("新软件待判定", "New App Needs Classification"),
      detail: `${cleanProcessName(process)} · ${formatSeconds(
        Math.max(0, pending.total_seconds),
      )}`,
      actions: [
        {
          label: tx("学习", "Learn"),
          run: () => actions.saveRule(process, "LEARN"),
        },
        {
          label: tx("休息", "Break"),
          run: () => actions.saveRule(process, "REST"),
        },
        {
          label: tx("未分类", "Unclassified"),
          run: () => actions.saveRule(process, "IGNORE"),
        },
        {
          label: tx("稍后提醒", "Remind later"),
          run: () => actions.postponeRule(key),
        },
      ],
    });
  }

  return descriptors;
}
