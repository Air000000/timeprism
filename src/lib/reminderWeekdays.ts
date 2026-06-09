import type { LocaleCode } from "./locale";

const DEFAULT_REMINDER_WEEKLY_DAYS = [1, 2, 3, 4, 5];

export type ReminderWeekdayOption = {
  value: number;
  zh: string;
  en: string;
};

export const reminderWeekdayOptions: ReminderWeekdayOption[] = [
  { value: 0, zh: "日", en: "Sun" },
  { value: 1, zh: "一", en: "Mon" },
  { value: 2, zh: "二", en: "Tue" },
  { value: 3, zh: "三", en: "Wed" },
  { value: 4, zh: "四", en: "Thu" },
  { value: 5, zh: "五", en: "Fri" },
  { value: 6, zh: "六", en: "Sat" },
];

export function defaultReminderWeeklyDays(): number[] {
  return [...DEFAULT_REMINDER_WEEKLY_DAYS];
}

export function normalizeReminderWeeklyDays(days: readonly number[] | null | undefined): number[] {
  return (days ?? [])
    .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
    .sort((a, b) => a - b);
}

export function reminderWeekdayShortLabel(day: number, locale: LocaleCode): string {
  const option = reminderWeekdayOptions.find((item) => item.value === day);
  if (!option) {
    return `${day}`;
  }
  return locale === "zh-CN" ? option.zh : option.en;
}
