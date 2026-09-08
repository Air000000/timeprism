import { computed, type Ref } from "vue";
import type { Reminder } from "../api";
import {
  formatHomeReminderDueText,
  isHomeReminderVisibleToday,
} from "../lib/reminderSchedule";
import type { LocaleCode } from "../lib/locale";

type TranslateFn = (zh: string, en: string) => string;

type UseHomeScheduleOptions = {
  locale: Ref<LocaleCode>;
  tx: TranslateFn;
  reminders: Ref<Reminder[]>;
  sortedReminders: (items: Reminder[]) => Reminder[];
};

export function useHomeSchedule({
  locale,
  tx,
  reminders,
  sortedReminders,
}: UseHomeScheduleOptions) {
  function reminderDueText(item: Reminder): string {
    return formatHomeReminderDueText(item, locale.value, tx);
  }

  const homeScheduleItems = computed(() =>
    sortedReminders(reminders.value)
      .filter((item) => isHomeReminderVisibleToday(item))
  );

  return {
    homeScheduleItems,
    reminderDueText,
  };
}
