import type { Reminder } from "../api";
import { defaultReminderWeeklyDays } from "./reminderWeekdays";

type ReminderDraftFormatters = {
  timeMinutesLabel: (minutes: number) => string;
  toDateTimeLocalValue: (timestamp: number) => string;
};

export type HomeReminderEditDraft = {
  content: string;
  dailyTime?: string;
  enabled: boolean;
  remindAt: string;
  repeatRule: Reminder["repeat_rule"];
  weeklyDays: number[];
};

export function buildHomeReminderEditDraft(
  item: Reminder,
  formatters: ReminderDraftFormatters,
): HomeReminderEditDraft {
  if (item.repeat_rule === "DAILY" || item.repeat_rule === "WEEKLY") {
    return {
      content: item.content,
      dailyTime: formatters.timeMinutesLabel(item.daily_time_minutes ?? 9 * 60),
      enabled: item.daily_time_minutes !== null,
      remindAt: "",
      repeatRule: item.repeat_rule,
      weeklyDays: item.repeat_rule === "WEEKLY"
        ? [...(item.weekly_days ?? defaultReminderWeeklyDays())].sort((a, b) => a - b)
        : defaultReminderWeeklyDays(),
    };
  }

  return {
    content: item.content,
    enabled: item.remind_at !== null,
    remindAt: item.remind_at !== null
      ? formatters.toDateTimeLocalValue(item.remind_at)
      : "",
    repeatRule: item.repeat_rule,
    weeklyDays: defaultReminderWeeklyDays(),
  };
}
