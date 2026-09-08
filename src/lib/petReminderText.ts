import type { LocaleCode } from "./locale";
import type { ReminderLite } from "./petPrompts";

type TranslateFn = (zh: string, en: string) => string;

function clampedReminderTime(minutes: number | null): string {
  const safeMinutes = Math.max(0, Math.min(1439, minutes ?? 9 * 60));
  const hour = Math.floor(safeMinutes / 60)
    .toString()
    .padStart(2, "0");
  const minute = (safeMinutes % 60).toString().padStart(2, "0");
  return `${hour}:${minute}`;
}

export function formatPetReminderDueText(
  item: ReminderLite,
  tx: TranslateFn,
  locale: LocaleCode,
): string {
  if (item.repeat_rule === "DAILY") {
    return `${tx("每日", "Daily")} ${clampedReminderTime(item.daily_time_minutes)}`;
  }

  if (item.repeat_rule === "WEEKLY") {
    const labels = (item.weekly_days ?? [])
      .map((day) => [
        tx("日", "Sun"),
        tx("一", "Mon"),
        tx("二", "Tue"),
        tx("三", "Wed"),
        tx("四", "Thu"),
        tx("五", "Fri"),
        tx("六", "Sat"),
      ][day] ?? `${day}`)
      .join(" ");
    return `${tx("每周", "Weekly")} ${labels} ${clampedReminderTime(item.daily_time_minutes)}`.trim();
  }

  const date = new Date(item.next_due_timestamp * 1000);
  return date.toLocaleString(locale, {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}
