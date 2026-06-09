import { computed, ref } from "vue";
import {
  deleteReminder,
  saveReminder,
  setReminderDone,
  setReminderOrder,
  snoozeReminder,
  type Reminder,
} from "../api";
import { sortedReminders } from "../lib/reminderSort";
import { buildReminderSavePayload, type ReminderUpsertInput } from "../lib/reminderUpsert";

type TranslateFn = (zh: string, en: string) => string;

export { sortedReminders } from "../lib/reminderSort";
export type { ReminderUpsertInput } from "../lib/reminderUpsert";

type UseRemindersOptions = {
  tx: TranslateFn;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

export function useReminders({ tx, refreshData, setErrorMessage }: UseRemindersOptions) {
  const reminders = ref<Reminder[]>([]);
  const reminderActionLoading = ref(false);
  const reminderListForPanel = computed(() => sortedReminders(reminders.value));

  async function handleUpsertReminder(input: ReminderUpsertInput) {
    reminderActionLoading.value = true;
    try {
      await saveReminder(buildReminderSavePayload(input, tx));
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  async function handleDeleteReminder(id: number) {
    if (reminderActionLoading.value) {
      return;
    }
    reminderActionLoading.value = true;
    try {
      await deleteReminder(id);
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  async function handleReminderDone(id: number, done: boolean) {
    if (reminderActionLoading.value) {
      return;
    }
    reminderActionLoading.value = true;
    try {
      await setReminderDone({ id, done });
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  async function handleReminderReorder(orderedIds: number[]) {
    if (reminderActionLoading.value || orderedIds.length === 0) {
      return;
    }

    const orderedSet = new Set(orderedIds);
    if (orderedSet.size !== orderedIds.length) {
      throw new Error(tx("排序数据无效", "Invalid reminder ordering"));
    }

    const previous = reminders.value.map((item) => ({ ...item }));
    const sortedCurrent = sortedReminders(reminders.value);
    const untouched = sortedCurrent.filter((item) => !orderedSet.has(item.id));
    const orderedItems = orderedIds
      .map((id) => reminders.value.find((item) => item.id === id))
      .filter((item): item is Reminder => Boolean(item));
    const nextItems = [...orderedItems, ...untouched].map((item, index) => ({
      ...item,
      sort_order: index,
    }));

    reminders.value = nextItems;
    reminderActionLoading.value = true;
    try {
      await setReminderOrder({ ordered_ids: nextItems.map((item) => item.id) });
      await refreshData();
    } catch (e) {
      reminders.value = previous;
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  async function handleReminderSnooze(id: number, seconds = 600) {
    if (reminderActionLoading.value) {
      return;
    }
    reminderActionLoading.value = true;
    try {
      await snoozeReminder(id, seconds);
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  return {
    reminders,
    reminderActionLoading,
    reminderListForPanel,
    sortedReminders,
    handleUpsertReminder,
    handleDeleteReminder,
    handleReminderDone,
    handleReminderReorder,
    handleReminderSnooze,
  };
}
