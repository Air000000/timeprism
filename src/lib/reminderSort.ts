import type { Reminder } from "../api";

function reminderGroupRank(item: Reminder): number {
  return item.done ? 1 : 0;
}

function reminderRepeatRank(rule: Reminder["repeat_rule"]): number {
  if (rule === "NONE") {
    return 0;
  }
  if (rule === "DAILY") {
    return 1;
  }
  return 2;
}

function compareReminders(a: Reminder, b: Reminder): number {
  return reminderGroupRank(a) - reminderGroupRank(b)
    || a.sort_order - b.sort_order
    || reminderRepeatRank(a.repeat_rule) - reminderRepeatRank(b.repeat_rule)
    || a.next_due_timestamp - b.next_due_timestamp
    || b.updated_at - a.updated_at
    || a.id - b.id;
}

export function sortedReminders(items: Reminder[]): Reminder[] {
  return [...items].sort(compareReminders);
}
