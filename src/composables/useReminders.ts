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
import { parseClockToMinutes, parseDateTimeLocalToUnix } from "../lib/time";

type TranslateFn = (zh: string, en: string) => string;

export { sortedReminders } from "../lib/reminderSort";

export type ReminderUpsertInput = {
  id?: number;
  content: string;
  repeat_rule: Reminder["repeat_rule"];
  remind_at_text?: string;
  daily_time_text?: string;
  weekly_days?: number[];
  reminder_enabled: boolean;
};

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
    const content = input.content.trim();
    if (!content) {
      throw new Error(tx("提醒内容不能为空", "Reminder content cannot be empty"));
    }

    reminderActionLoading.value = true;
    try {
      if (input.repeat_rule === "DAILY" || input.repeat_rule === "WEEKLY") {
        let dailyMinutes: number | undefined;
        if (input.reminder_enabled) {
          const parsed = parseClockToMinutes(input.daily_time_text ?? "");
          if (parsed === null) {
            throw new Error(tx("每日时间格式错误，请使用 HH:MM", "Invalid daily time, use HH:MM"));
          }
          dailyMinutes = parsed;
        }

        let weeklyDays: number[] | undefined;
        if (input.repeat_rule === "WEEKLY") {
          weeklyDays = (input.weekly_days ?? [])
            .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
            .sort((a, b) => a - b);
          if (weeklyDays.length === 0) {
            throw new Error(tx("请选择每周重复的日期", "Please choose at least one weekday"));
          }
        }

        await saveReminder({
          id: input.id,
          content,
          repeat_rule: input.repeat_rule,
          daily_time_minutes: dailyMinutes,
          weekly_days: weeklyDays,
        });
      } else {
        let remindAt: number | undefined;
        if (input.reminder_enabled) {
          const parsed = parseDateTimeLocalToUnix(input.remind_at_text ?? "");
          if (parsed === null) {
            throw new Error(tx("请选择有效提醒时间", "Please choose a valid reminder time"));
          }
          remindAt = parsed;
        }

        await saveReminder({
          id: input.id,
          content,
          repeat_rule: "NONE",
          remind_at: remindAt,
        });
      }
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
