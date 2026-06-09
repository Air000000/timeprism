import type { Reminder } from "../api";

export function buildHomeReminderDropOrder(
  items: readonly Reminder[],
  draggedId: number,
  targetItem: Pick<Reminder, "id" | "done">,
): number[] | null {
  if (!Number.isFinite(draggedId) || draggedId === targetItem.id) {
    return null;
  }

  const allItems = [...items];
  const dragged = allItems.find((item) => item.id === draggedId);
  if (!dragged || dragged.done !== targetItem.done) {
    return null;
  }

  const ordered = allItems.filter((item) => item.done === dragged.done);
  const fromIndex = ordered.findIndex((item) => item.id === draggedId);
  const toIndex = ordered.findIndex((item) => item.id === targetItem.id);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return null;
  }

  const [moved] = ordered.splice(fromIndex, 1);
  ordered.splice(toIndex, 0, moved);

  const doneGroup = allItems.filter((item) => item.done);
  const undoneGroup = allItems.filter((item) => !item.done);
  return dragged.done
    ? [...undoneGroup.map((item) => item.id), ...ordered.map((item) => item.id)]
    : [...ordered.map((item) => item.id), ...doneGroup.map((item) => item.id)];
}
