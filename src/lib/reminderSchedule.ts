import type { Reminder } from "../api";
import type { LocaleCode } from "./locale";
import {
  currentLocalDayKey,
  currentLocalWeekday,
  dayKeyFromUnixSeconds,
  timeMinutesLabel,
} from "./time";

type TranslateFn = (zh: string, en: string) => string;

function formatReminderDateTime(unixSeconds: number, locale: LocaleCode): string {
  const date = new Date(unixSeconds * 1000);
  return date.toLocaleString(locale, {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function reminderWeekdayShortLabel(day: number, locale: LocaleCode): string {
  const zh = ["日", "一", "二", "三", "四", "五", "六"];
  const en = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  return locale === "zh-CN" ? (zh[day] ?? `${day}`) : (en[day] ?? `${day}`);
}

export function formatReminderWeeklyDaysText(
  days: number[] | null | undefined,
  locale: LocaleCode,
  tx: TranslateFn,
): string {
  const safe = (days ?? [])
    .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
    .sort((a, b) => a - b);
  if (safe.length === 0) {
    return tx("未选择", "No days");
  }
  return safe.map((day) => reminderWeekdayShortLabel(day, locale)).join(" ");
}

export function formatHomeReminderDueText(
  item: Reminder,
  locale: LocaleCode,
  tx: TranslateFn,
): string {
  if (item.snooze_until && item.snooze_until > Math.floor(Date.now() / 1000)) {
    return `${tx("稍后至", "Snoozed until")} ${formatReminderDateTime(item.snooze_until, locale)}`;
  }

  if (item.repeat_rule === "DAILY") {
    if (item.daily_time_minutes === null) {
      return tx("每日 · 不提醒", "Daily · No reminder");
    }
    const clock = timeMinutesLabel(item.daily_time_minutes ?? 0);
    const doneText = item.done ? tx("（今日已完成）", "(done today)") : "";
    return `${tx("每日", "Daily")} ${clock} ${doneText}`.trim();
  }

  if (item.repeat_rule === "WEEKLY") {
    const clock = item.daily_time_minutes === null
      ? tx("不提醒", "No reminder")
      : timeMinutesLabel(item.daily_time_minutes ?? 0);
    const doneText = item.done ? tx("（本轮已完成）", "(done this turn)") : "";
    return `${tx("每周", "Weekly")} ${formatReminderWeeklyDaysText(item.weekly_days, locale, tx)} ${clock} ${doneText}`.trim();
  }

  if (item.remind_at === null) {
    return tx("一次性 · 不提醒", "One-time · No reminder");
  }

  return formatReminderDateTime(item.next_due_timestamp, locale);
}

export function isHomeReminderVisibleToday(item: Reminder): boolean {
  const todayKey = currentLocalDayKey();
  const completedToday = item.completed_day_key === todayKey;

  if (item.repeat_rule === "DAILY") {
    return true;
  }

  if (item.repeat_rule === "WEEKLY") {
    const todayWeekday = currentLocalWeekday();
    return completedToday || (item.weekly_days ?? []).includes(todayWeekday);
  }

  if (item.remind_at === null) {
    return !item.done;
  }

  const oneShotDayKey = dayKeyFromUnixSeconds(item.remind_at) ?? dayKeyFromUnixSeconds(item.created_at);
  return completedToday || oneShotDayKey === todayKey;
}
