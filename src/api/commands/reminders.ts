import { invokeCommand } from "../client";
import type { Reminder } from "../types";

export async function listReminders(
  limit = 50,
  includeCompleted = false,
): Promise<Reminder[]> {
  return invokeCommand("list_reminders", { limit, includeCompleted });
}

export async function listDueReminders(limit = 6): Promise<Reminder[]> {
  return invokeCommand("list_due_reminders", { limit });
}

export async function saveReminder(input: {
  id?: number;
  content: string;
  repeat_rule: "NONE" | "DAILY" | "WEEKLY";
  remind_at?: number;
  daily_time_minutes?: number;
  weekly_days?: number[];
}): Promise<number> {
  return invokeCommand("save_reminder", { input });
}

export async function deleteReminder(id: number): Promise<boolean> {
  return invokeCommand("delete_reminder", { id });
}

export async function setReminderDone(input: {
  id: number;
  done: boolean;
}): Promise<boolean> {
  return invokeCommand("set_reminder_done", { input });
}

export async function snoozeReminder(id: number, snoozeSeconds = 600): Promise<boolean> {
  return invokeCommand("snooze_reminder", { id, snoozeSeconds });
}

export async function setReminderOrder(input: {
  ordered_ids: number[];
}): Promise<boolean> {
  return invokeCommand("set_reminder_order", { input });
}
