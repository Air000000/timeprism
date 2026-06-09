import type { Reminder } from "../api";
import { parseClockToMinutes, parseDateTimeLocalToUnix } from "./time";

type TranslateFn = (zh: string, en: string) => string;

export type ReminderUpsertInput = {
  id?: number;
  content: string;
  repeat_rule: Reminder["repeat_rule"];
  remind_at_text?: string;
  daily_time_text?: string;
  weekly_days?: number[];
  reminder_enabled: boolean;
};

export type ReminderSavePayload = {
  id?: number;
  content: string;
  repeat_rule: Reminder["repeat_rule"];
  remind_at?: number;
  daily_time_minutes?: number;
  weekly_days?: number[];
};

function normalizeWeeklyDays(days: number[] | undefined): number[] {
  return (days ?? [])
    .filter((day, index, arr) => Number.isInteger(day) && day >= 0 && day <= 6 && arr.indexOf(day) === index)
    .sort((a, b) => a - b);
}

export function buildReminderSavePayload(
  input: ReminderUpsertInput,
  tx: TranslateFn,
): ReminderSavePayload {
  const content = input.content.trim();
  if (!content) {
    throw new Error(tx("提醒内容不能为空", "Reminder content cannot be empty"));
  }

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
      weeklyDays = normalizeWeeklyDays(input.weekly_days);
      if (weeklyDays.length === 0) {
        throw new Error(tx("请选择每周重复的日期", "Please choose at least one weekday"));
      }
    }

    return {
      id: input.id,
      content,
      repeat_rule: input.repeat_rule,
      daily_time_minutes: dailyMinutes,
      weekly_days: weeklyDays,
    };
  }

  let remindAt: number | undefined;
  if (input.reminder_enabled) {
    const parsed = parseDateTimeLocalToUnix(input.remind_at_text ?? "");
    if (parsed === null) {
      throw new Error(tx("请选择有效提醒时间", "Please choose a valid reminder time"));
    }
    remindAt = parsed;
  }

  return {
    id: input.id,
    content,
    repeat_rule: "NONE",
    remind_at: remindAt,
  };
}
