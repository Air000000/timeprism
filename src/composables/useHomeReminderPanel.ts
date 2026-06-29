import { ref } from "vue";
import type { Reminder } from "../api";
import type { FeedbackTone, HomeViewContext } from "../components/viewContexts";
import {
  buildHomeReminderEditDraft,
  buildHomeReminderUpsertInput,
  defaultHomeReminderDraft,
} from "../lib/homeReminderDraft";
import {
  reminderContentEmptyFeedback,
  reminderDeletedFeedback,
  reminderDoneFeedback,
  reminderSavedFeedback,
  reminderSnoozedFeedback,
} from "../lib/homeReminderFeedback";
import {
  buildHomeReminderDropOrder,
  readHomeReminderDraggedId,
  setHomeReminderDragData,
  shouldIgnoreHomeReminderDrop,
} from "../lib/homeReminderReorder";
import {
  reminderWeekdayOptions,
  toggleReminderWeeklyDay,
  type ReminderWeekdayOption,
} from "../lib/reminderWeekdays";

export const weekdayOptions = reminderWeekdayOptions;

export function useHomeReminderPanel(getCtx: () => HomeViewContext) {
  const initialReminderDraft = defaultHomeReminderDraft();
  const scheduleModalOpen = ref(false);
  const reminderEditId = ref<number | null>(initialReminderDraft.id);
  const reminderDraftContent = ref(initialReminderDraft.content);
  const reminderDraftRepeat = ref<Reminder["repeat_rule"]>(initialReminderDraft.repeatRule);
  const reminderEnabled = ref(initialReminderDraft.enabled);
  const reminderDraftAt = ref(initialReminderDraft.remindAt);
  const reminderDraftDailyTime = ref(initialReminderDraft.dailyTime);
  const reminderDraftWeeklyDays = ref<number[]>(initialReminderDraft.weeklyDays);
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
    const draft = defaultHomeReminderDraft();
    reminderEditId.value = draft.id;
    reminderDraftContent.value = draft.content;
    reminderDraftRepeat.value = draft.repeatRule;
    reminderEnabled.value = draft.enabled;
    reminderDraftAt.value = draft.remindAt;
    reminderDraftDailyTime.value = draft.dailyTime;
    reminderDraftWeeklyDays.value = draft.weeklyDays;
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
      reminderFeedback.value = reminderContentEmptyFeedback(ctx.tx);
      return;
    }

    try {
      const isCreate = reminderEditId.value === null;
      await ctx.handleUpsertReminder(buildHomeReminderUpsertInput({
        id: reminderEditId.value,
        content,
        repeatRule: reminderDraftRepeat.value,
        remindAt: reminderDraftAt.value,
        dailyTime: reminderDraftDailyTime.value,
        weeklyDays: reminderDraftWeeklyDays.value,
        enabled: reminderEnabled.value,
      }));
      reminderFeedbackType.value = "ok";
      reminderFeedback.value = reminderSavedFeedback(isCreate, ctx.tx);
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
      reminderFeedback.value = reminderDeletedFeedback(ctx.tx);
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
      reminderFeedback.value = reminderDoneFeedback(done, ctx.tx);
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
      reminderFeedback.value = reminderSnoozedFeedback(ctx.tx);
    } catch (e) {
      reminderFeedbackType.value = "error";
      reminderFeedback.value = `${e}`;
    }
  }

  function handleReminderDragStart(item: Reminder, event?: DragEvent) {
    draggingReminderId.value = item.id;
    dropTargetReminderId.value = item.id;
    setHomeReminderDragData(event, item.id);
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
    const draggedId = readHomeReminderDraggedId(draggingReminderId.value, event);
    draggingReminderId.value = null;
    dropTargetReminderId.value = null;
    if (shouldIgnoreHomeReminderDrop(draggedId, targetItem.id)) {
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
