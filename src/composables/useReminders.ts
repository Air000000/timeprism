import { computed, ref } from "vue";
import {
  deleteReminder,
  saveReminder,
  setReminderDone,
  setReminderOrder,
  snoozeReminder,
  type Reminder,
} from "../api";
import {
  buildReminderReorderState,
  hasDuplicateReminderOrderIds,
} from "../lib/reminderReorder";
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

  async function runReminderMutation(task: () => Promise<void>) {
    reminderActionLoading.value = true;
    try {
      await task();
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      throw e;
    } finally {
      reminderActionLoading.value = false;
    }
  }

  async function handleUpsertReminder(input: ReminderUpsertInput) {
    await runReminderMutation(async () => {
      await saveReminder(buildReminderSavePayload(input, tx));
    });
  }

  async function handleDeleteReminder(id: number) {
    if (reminderActionLoading.value) {
      return;
    }

    await runReminderMutation(async () => {
      await deleteReminder(id);
    });
  }

  async function handleReminderDone(id: number, done: boolean) {
    if (reminderActionLoading.value) {
      return;
    }

    await runReminderMutation(async () => {
      await setReminderDone({ id, done });
    });
  }

  async function handleReminderReorder(orderedIds: number[]) {
    if (reminderActionLoading.value || orderedIds.length === 0) {
      return;
    }

    if (hasDuplicateReminderOrderIds(orderedIds)) {
      throw new Error(tx("排序数据无效", "Invalid reminder ordering"));
    }

    const { previous, nextItems, nextOrderedIds } = buildReminderReorderState(
      reminders.value,
      orderedIds,
    );

    reminders.value = nextItems;
    reminderActionLoading.value = true;
    try {
      await setReminderOrder({ ordered_ids: nextOrderedIds });
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

    await runReminderMutation(async () => {
      await snoozeReminder(id, seconds);
    });
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
