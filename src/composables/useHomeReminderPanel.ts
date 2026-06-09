import { ref } from "vue";
import type { Reminder } from "../api";
import type { FeedbackTone, HomeViewContext } from "../components/viewContexts";
import { buildHomeReminderEditDraft } from "../lib/homeReminderDraft";
import { buildHomeReminderDropOrder } from "../lib/homeReminderReorder";
import {
  defaultReminderWeeklyDays,
  reminderWeekdayOptions,
  toggleReminderWeeklyDay,
  type ReminderWeekdayOption,
} from "../lib/reminderWeekdays";

export const weekdayOptions = reminderWeekdayOptions;

export function useHomeReminderPanel(getCtx: () => HomeViewContext) {
  const scheduleModalOpen = ref(false);
  const reminderEditId = ref<number | null>(null);
  const reminderDraftContent = ref("");
  const reminderDraftRepeat = ref<Reminder["repeat_rule"]>("NONE");
  const reminderEnabled = ref(true);
  const reminderDraftAt = ref("");
  const reminderDraftDailyTime = ref("09:00");
  const reminderDraftWeeklyDays = ref<number[]>(defaultReminderWeeklyDays());
  const reminderFeedback = ref("");
  const reminderFeedbackType = ref<FeedbackTone>("info");
  const draggingReminderId = ref<number | null>(null);
  const dropTargetReminderId = ref<number | null>(null);

  function weekdayLabel(day: ReminderWeekdayOption): string {
    return getCtx().tx(day.zh, day.en);
  }

  function toggleWeeklyDay(day: number) {
    reminderDraftWeeklyDays.value = toggleReminderWeeklyDay(reminderDraftWeeklyDays.value, day);
  }

  function resetReminderDraft() {
    reminderEditId.value = null;
    reminderDraftContent.value = "";
    reminderDraftRepeat.value = "NONE";
    reminderEnabled.value = true;
    reminderDraftAt.value = "";
    reminderDraftDailyTime.value = "09:00";
    reminderDraftWeeklyDays.value = defaultReminderWeeklyDays();
  }

  function openReminderComposer() {
    scheduleModalOpen.value = true;
    resetReminderDraft();
    reminderFeedbackType.value = "info";
    reminderFeedback.value = "";
  }

  function closeScheduleSettings() {
    scheduleModalOpen.value = false;
  }

  function startEditReminder(item: Reminder) {
    const ctx = getCtx();
    const draft = buildHomeReminderEditDraft(item, ctx);
    scheduleModalOpen.value = true;
    reminderEditId.value = item.id;
    reminderDraftContent.value = draft.content;
    reminderDraftRepeat.value = draft.repeatRule;
    reminderEnabled.value = draft.enabled;
    reminderDraftAt.value = draft.remindAt;
    reminderDraftWeeklyDays.value = draft.weeklyDays;
    if (draft.dailyTime !== undefined) {
      reminderDraftDailyTime.value = draft.dailyTime;
    }
  }

  async function saveReminderFromModal() {
    const ctx = getCtx();
    const content = reminderDraftContent.value.trim();
    if (!content) {
      reminderFeedbackType.value = "warn";
      reminderFeedback.value = ctx.tx("提醒内容不能为空", "Reminder content cannot be empty");
      return;
    }

    try {
      await ctx.handleUpsertReminder({
        id: reminderEditId.value ?? undefined,
        content,
        repeat_rule: reminderDraftRepeat.value,
        remind_at_text: reminderDraftAt.value,
        daily_time_text: reminderDraftDailyTime.value,
        weekly_days: reminderDraftWeeklyDays.value,
        reminder_enabled: reminderEnabled.value,
      });
      reminderFeedbackType.value = "ok";
      reminderFeedback.value = reminderEditId.value === null
        ? ctx.tx("提醒已创建", "Reminder created")
        : ctx.tx("提醒已更新", "Reminder updated");
      resetReminderDraft();
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  async function quickDeleteReminder(id: number) {
    const ctx = getCtx();
    try {
      await ctx.handleDeleteReminder(id);
      reminderFeedbackType.value = "warn";
      reminderFeedback.value = ctx.tx("提醒已删除", "Reminder deleted");
      if (reminderEditId.value === id) {
        resetReminderDraft();
      }
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  async function quickDoneReminder(id: number, done: boolean) {
    const ctx = getCtx();
    try {
      await ctx.handleReminderDone(id, done);
      reminderFeedbackType.value = "ok";
      reminderFeedback.value = done
        ? ctx.tx("提醒已完成", "Reminder marked done")
        : ctx.tx("提醒已恢复", "Reminder restored");
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  async function quickSnoozeReminder(id: number) {
    const ctx = getCtx();
    try {
      await ctx.handleReminderSnooze(id, 600);
      reminderFeedbackType.value = "info";
      reminderFeedback.value = ctx.tx("提醒已稍后10分钟", "Reminder snoozed 10m");
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  function handleReminderDragStart(item: Reminder, event?: DragEvent) {
    draggingReminderId.value = item.id;
    dropTargetReminderId.value = item.id;
    if (event?.dataTransfer) {
      event.dataTransfer.effectAllowed = "move";
      event.dataTransfer.dropEffect = "move";
      event.dataTransfer.setData("text/plain", String(item.id));
    }
  }

  function handleReminderDragEnter(item: Reminder) {
    if (draggingReminderId.value === null || draggingReminderId.value === item.id) {
      return;
    }
    dropTargetReminderId.value = item.id;
  }

  function handleReminderDragEnd() {
    draggingReminderId.value = null;
    dropTargetReminderId.value = null;
  }

  async function handleReminderDrop(targetItem: Reminder, event?: DragEvent) {
    const ctx = getCtx();
    const draggedId = draggingReminderId.value
      ?? Number.parseInt(event?.dataTransfer?.getData("text/plain") ?? "", 10);
    draggingReminderId.value = null;
    dropTargetReminderId.value = null;
    if (!Number.isFinite(draggedId) || draggedId === targetItem.id) {
      return;
    }

    const orderedIds = buildHomeReminderDropOrder(ctx.reminderListForPanel, draggedId, targetItem);
    if (!orderedIds) {
      return;
    }

    try {
      await ctx.handleReminderReorder(orderedIds);
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  return {
    closeScheduleSettings,
    draggingReminderId,
    dropTargetReminderId,
    handleReminderDragEnd,
    handleReminderDragEnter,
    handleReminderDragStart,
    handleReminderDrop,
    openReminderComposer,
    quickDeleteReminder,
    quickDoneReminder,
    quickSnoozeReminder,
    reminderDraftAt,
    reminderDraftContent,
    reminderDraftDailyTime,
    reminderDraftRepeat,
    reminderDraftWeeklyDays,
    reminderEditId,
    reminderEnabled,
    reminderFeedback,
    reminderFeedbackType,
    resetReminderDraft,
    saveReminderFromModal,
    scheduleModalOpen,
    startEditReminder,
    toggleWeeklyDay,
    weekdayLabel,
    weekdayOptions,
  };
}
