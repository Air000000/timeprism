import { computed, type Ref } from "vue";
import type {
  IdlePrompt,
  PendingRuleProcess,
} from "../api";
import type {
  HomeViewContext,
  TranslateFn,
} from "../components/viewContexts";

type HomeViewContextOptions = {
  tx: TranslateFn;
  formatSeconds: HomeViewContext["formatSeconds"];
  todayLearnSeconds: Readonly<Ref<number>>;
  currentStatusLabel: Readonly<Ref<string>>;
  currentStatusTone: Readonly<Ref<HomeViewContext["currentStatusTone"]>>;
  goalProgressPct: Readonly<Ref<string>>;
  goalProgressFillNum: Readonly<Ref<number>>;
  goalOverflowTier: Readonly<Ref<HomeViewContext["goalOverflowTier"]>>;
  recentSummary: Readonly<Ref<string>>;
  homeScheduleItems: Readonly<Ref<HomeViewContext["homeScheduleItems"]>>;
  reminderListForPanel: Readonly<Ref<HomeViewContext["reminderListForPanel"]>>;
  reminderDueText: HomeViewContext["reminderDueText"];
  toDateTimeLocalValue: HomeViewContext["toDateTimeLocalValue"];
  timeMinutesLabel: HomeViewContext["timeMinutesLabel"];
  handleUpsertReminder: HomeViewContext["handleUpsertReminder"];
  handleDeleteReminder: HomeViewContext["handleDeleteReminder"];
  handleReminderDone: HomeViewContext["handleReminderDone"];
  handleReminderReorder: HomeViewContext["handleReminderReorder"];
  handleReminderSnooze: HomeViewContext["handleReminderSnooze"];
  reminderActionLoading: Readonly<Ref<boolean>>;
  shiftHeatmapMonth: HomeViewContext["shiftHeatmapMonth"];
  monthTitleText: Readonly<Ref<string>>;
  weekHeaders: Readonly<Ref<string[]>>;
  calendarHeatmapCells: Readonly<Ref<HomeViewContext["calendarHeatmapCells"]>>;
  heatmapCellClass: HomeViewContext["heatmapCellClass"];
  heatmapDayText: HomeViewContext["heatmapDayText"];
  pendingRuleProcesses: Readonly<Ref<PendingRuleProcess[]>>;
  idlePrompts: Readonly<Ref<IdlePrompt[]>>;
  dueReminderCount: Readonly<Ref<number>>;
  homeMonthRhythmBars: Readonly<Ref<HomeViewContext["homeMonthRhythmBars"]>>;
};

export function useHomeViewContext({
  tx,
  formatSeconds,
  todayLearnSeconds,
  currentStatusLabel,
  currentStatusTone,
  goalProgressPct,
  goalProgressFillNum,
  goalOverflowTier,
  recentSummary,
  homeScheduleItems,
  reminderListForPanel,
  reminderDueText,
  toDateTimeLocalValue,
  timeMinutesLabel,
  handleUpsertReminder,
  handleDeleteReminder,
  handleReminderDone,
  handleReminderReorder,
  handleReminderSnooze,
  reminderActionLoading,
  shiftHeatmapMonth,
  monthTitleText,
  weekHeaders,
  calendarHeatmapCells,
  heatmapCellClass,
  heatmapDayText,
  pendingRuleProcesses,
  idlePrompts,
  dueReminderCount,
  homeMonthRhythmBars,
}: HomeViewContextOptions) {
  return computed<HomeViewContext>(() => ({
    tx,
    formatSeconds,
    todayLearnSeconds: todayLearnSeconds.value,
    currentStatusLabel: currentStatusLabel.value,
    currentStatusTone: currentStatusTone.value,
    goalProgressPct: goalProgressPct.value,
    goalProgressFillNum: goalProgressFillNum.value,
    goalOverflowTier: goalOverflowTier.value,
    recentSummary: recentSummary.value,
    homeScheduleItems: homeScheduleItems.value,
    reminderListForPanel: reminderListForPanel.value,
    reminderDueText,
    toDateTimeLocalValue,
    timeMinutesLabel,
    handleUpsertReminder,
    handleDeleteReminder,
    handleReminderDone,
    handleReminderReorder,
    handleReminderSnooze,
    reminderActionLoading: reminderActionLoading.value,
    shiftHeatmapMonth,
    monthTitleText: monthTitleText.value,
    weekHeaders: weekHeaders.value,
    calendarHeatmapCells: calendarHeatmapCells.value,
    heatmapCellClass,
    heatmapDayText,
    pendingRuleCount: pendingRuleProcesses.value.length,
    idlePromptCount: idlePrompts.value.length,
    dueReminderCount: dueReminderCount.value,
    homeMonthRhythmBars: homeMonthRhythmBars.value,
  }));
}
