import type { Reminder } from "../api";
import type { ReminderUpsertInput } from "./reminderUpsert";
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

export type HomeReminderUpsertDraft = {
  content: string;
  dailyTime: string;
  enabled: boolean;
  id: number | null;
  remindAt: string;
  repeatRule: Reminder["repeat_rule"];
  weeklyDays: number[];
};

export function defaultHomeReminderDraft(): HomeReminderUpsertDraft {
  return {
    content: "",
    dailyTime: "09:00",
    enabled: true,
    id: null,
    remindAt: "",
    repeatRule: "NONE",
    weeklyDays: defaultReminderWeeklyDays(),
  };
}

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

export function buildHomeReminderUpsertInput(
  draft: HomeReminderUpsertDraft,
): ReminderUpsertInput {
  return {
    id: draft.id ?? undefined,
    content: draft.content,
    repeat_rule: draft.repeatRule,
    remind_at_text: draft.remindAt,
    daily_time_text: draft.dailyTime,
    weekly_days: draft.weeklyDays,
    reminder_enabled: draft.enabled,
  };
}
