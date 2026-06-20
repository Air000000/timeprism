import type { Reminder } from "../api";
import { sortedReminders } from "./reminderSort";

export type ReminderReorderState = {
  previous: Reminder[];
  nextItems: Reminder[];
  nextOrderedIds: number[];
};

export function hasDuplicateReminderOrderIds(orderedIds: readonly number[]): boolean {
  return new Set(orderedIds).size !== orderedIds.length;
}

export function buildReminderReorderState(
  items: readonly Reminder[],
  orderedIds: readonly number[],
): ReminderReorderState {
  const orderedSet = new Set(orderedIds);
  const previous = items.map((item) => ({ ...item }));
  const sortedCurrent = sortedReminders([...items]);
  const untouched = sortedCurrent.filter((item) => !orderedSet.has(item.id));
  const orderedItems = orderedIds
    .map((id) => items.find((item) => item.id === id))
    .filter((item): item is Reminder => Boolean(item));
  const nextItems = [...orderedItems, ...untouched].map((item, index) => ({
    ...item,
    sort_order: index,
  }));

  return {
    previous,
    nextItems,
    nextOrderedIds: nextItems.map((item) => item.id),
  };
}
