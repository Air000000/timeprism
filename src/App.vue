<script setup lang="ts">
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import AppTopNav from "./components/AppTopNav.vue";
import GuardView from "./components/GuardView.vue";
import HomeView from "./components/HomeView.vue";
import IdlePromptBanner from "./components/IdlePromptBanner.vue";
import InsightsView from "./components/InsightsView.vue";
import SettingsView from "./components/SettingsView.vue";
import type {
  GuardViewContext,
  HomeViewContext,
  IdlePromptBannerContext,
  InsightsViewContext,
  SettingsViewContext,
} from "./components/viewContexts";
import { useAutoCaptureSampler } from "./composables/useAutoCaptureSampler";
import {
  useAppNavigation,
  type HistorySubViewKey,
  type MainViewKey,
} from "./composables/useAppNavigation";
import { useDisplayFormatters } from "./composables/useDisplayFormatters";
import { useGuardData } from "./composables/useGuardData";
import { useGuardWorkflow } from "./composables/useGuardWorkflow";
import { useHeatmapCalendar } from "./composables/useHeatmapCalendar";
import { useHeatmapGoalSetting } from "./composables/useHeatmapGoalSetting";
import { useHomeData } from "./composables/useHomeData";
import { useHomeOverview } from "./composables/useHomeOverview";
import { useHomeRhythm } from "./composables/useHomeRhythm";
import { useHomeSchedule } from "./composables/useHomeSchedule";
import { useInsightsData } from "./composables/useInsightsData";
import { useInsightsSectionNavigation } from "./composables/useInsightsSectionNavigation";
import { useLocale } from "./composables/useLocale";
import { useReminders } from "./composables/useReminders";
import { useSettingsPrivacy } from "./composables/useSettingsPrivacy";
import { useThemeMode } from "./composables/useThemeMode";
import {
  formatSeconds,
  timeMinutesLabel,
  toDateTimeLocalValue,
} from "./lib/time";
import {
  type LearnHeatmapCell,
  type RecentLog,
  type UsageStackDay,
} from "./api";

const { locale, tx, applyLocale, initLocale } = useLocale();
const { themeMode, applyTheme, toggleThemeMode, initThemeMode } = useThemeMode();
const {
  formatClock,
  mappedTypeText,
  cleanProcessName,
  formatIdlePromptSpan,
} = useDisplayFormatters({ locale, tx });
const {
  currentMainView,
  mainViews,
  historySubView,
  historySubViews,
  selectHistoryView,
  selectMainView,
  selectGuardView,
} = useAppNavigation(tx);
const {
  scrollToInsightsSection,
  cleanupInsightsSectionNavigation,
} = useInsightsSectionNavigation({
  currentMainView,
  selectHistoryView,
});
const {
  privacy,
  autoStartEnabled,
  whitelist,
  whitelistInput,
  privacyFeedback,
  privacyFeedbackType,
  refreshSettingsData,
  resetPrivacyFeedback,
  handleSavePrivacySettings,
  handleAddWhitelist,
  handleRemoveWhitelist,
  onAutoStartChange,
  onWhitelistInput,
} = useSettingsPrivacy({ tx, refreshData, setErrorMessage });
const {
  reminders,
  reminderActionLoading,
  reminderListForPanel,
  sortedReminders,
  handleUpsertReminder,
  handleDeleteReminder,
  handleReminderDone,
  handleReminderReorder,
  handleReminderSnooze,
} = useReminders({ tx, refreshData, setErrorMessage });
const {
  homeScheduleItems,
  reminderDueText,
} = useHomeSchedule({
  locale,
  tx,
  reminders,
  sortedReminders,
});

const recentLogs = ref<RecentLog[]>([]);
const learnHeatmap = ref<LearnHeatmapCell[]>([]);
const homeUsageStack = ref<UsageStackDay[]>([]);
const error = ref("");
const privacyViewMounted = ref(false);
const {
  learnGoalSliderMinutes,
  getHeatmapGoalSeconds,
  loadHeatmapGoalSecondsSetting,
  cleanupHeatmapGoalSetting,
} = useHeatmapGoalSetting({ setErrorMessage });
const { homeMonthRhythmBars } = useHomeRhythm(homeUsageStack);
const {
  monthTitleText,
  weekHeaders,
  calendarHeatmapCells,
  heatmapCellClass,
  heatmapDayText,
  shiftHeatmapMonth,
  getHeatmapFetchDays,
  homeMonthGoalProgress,
  homeMonthActiveStreakDays,
} = useHeatmapCalendar({
  locale,
  learnHeatmap,
});
const {
  topApps,
  topAppsBarWidth,
  allTimeTopApps,
  allTimeIncludeIgnore,
  allTimeBarWidth,
  setAllTimeFilter,
  onAllTimeIgnoreToggle,
  recentTimelineGroups,
  recentDurationWidth,
  refreshInsightsData,
} = useInsightsData({
  recentLogs,
  learnHeatmap,
  getHeatmapFetchDays,
  getHeatmapGoalSeconds,
  refreshData,
  setErrorMessage,
});
const {
  autoCaptureEnabled,
  autoCaptureFeedback,
  guardFeedback,
  guardFeedbackType,
  foregroundDiagnostics,
  pendingRuleProcesses,
  idlePrompts,
  idleActionLoading,
  idleRememberChoice,
  ruleSearch,
  ruleSort,
  filteredSortedRules,
  currentIdlePrompt,
  resetGuardFeedback,
  refreshGuardData,
  handleResolveIdle,
  captureBlockReasonText,
  captureRuleText,
  canSaveRuleFromDiagnostic,
  handleSaveRuleFromDiagnostic,
  handleUpdateExistingRule,
  handleSavePendingRule,
  onAutoCaptureToggle,
  onIdleRememberChoiceChange,
  onRuleSearchInput,
  onRuleSortChange,
  onRuleMappedTypeChange,
} = useGuardData({
  tx,
  mappedTypeText,
  refreshData,
  setErrorMessage,
});
const {
  startAutoCaptureSampler,
  stopAutoCaptureSampler,
} = useAutoCaptureSampler({
  autoCaptureEnabled,
  autoCaptureFeedback,
  tx,
});

const {
  guardStep2Unlocked,
  guardStep3Unlocked,
  guardStep4Unlocked,
  guardStep3Done,
  guardCurrentStepText,
  guardCurrentStepIndex,
  guardStepLabels,
  markGuardStep3Done,
} = useGuardWorkflow({
  tx,
  pendingRuleProcesses,
  idlePrompts,
  guardFeedback,
  guardFeedbackType,
});

const {
  todaySummary,
  todayLearnSeconds,
  todayRestSeconds,
  goalProgressPct,
  goalProgressFillNum,
  goalOverflowTier,
  recentSummary,
  dueReminderCount,
  currentStatusLabel,
  currentStatusTone,
  homePendingSummary,
} = useHomeOverview({
  tx,
  learnGoalSliderMinutes,
  recentLogs,
  reminders,
  pendingRuleProcesses,
  idlePrompts,
  autoCaptureEnabled,
  cleanProcessName,
  formatClock,
});
const { refreshHomeData } = useHomeData({
  todaySummary,
  recentLogs,
  learnHeatmap,
  pendingRuleProcesses,
  idlePrompts,
  reminders,
  homeUsageStack,
  getHeatmapFetchDays,
  getHeatmapGoalSeconds,
  setErrorMessage,
});

function switchHistorySubView(next: HistorySubViewKey) {
  selectHistoryView(next);
  void refreshInsightsData();
}

function openGuardWorkflow() {
  selectGuardView();
  void refreshGuardData();
}

function setMainView(next: MainViewKey) {
  selectMainView(next);
  if (next === "insights") {
    void refreshInsightsData();
  } else if (next === "guard") {
    void refreshGuardData();
  } else if (next === "privacy") {
    privacyViewMounted.value = true;
    window.setTimeout(() => {
      void refreshSettingsData();
    }, 0);
  }
}

function setErrorMessage(e: unknown) {
  error.value = `${e}`;
}

async function refreshData() {
  await refreshHomeData();
  if (currentMainView.value === "insights") {
    await refreshInsightsData();
  } else if (currentMainView.value === "guard") {
    await refreshGuardData();
  } else if (currentMainView.value === "privacy") {
    await refreshSettingsData();
  }
}

let pollTimer: number | null = null;
let navigateSectionUnlisten: UnlistenFn | null = null;
let settingsWarmTimer: number | null = null;

function onLocaleChange(event: Event) {
  const input = event.target as HTMLSelectElement;
  if (input.value === "zh-CN" || input.value === "en-US") {
    locale.value = input.value;
  }
}

const idlePromptBannerCtx = computed<IdlePromptBannerContext | null>(() => {
  if (!currentIdlePrompt.value) {
    return null;
  }

  return {
    tx,
    currentIdlePrompt: currentIdlePrompt.value,
    formatIdlePromptSpan,
    idleRememberChoice: idleRememberChoice.value,
    onIdleRememberChoiceChange,
    idleActionLoading: idleActionLoading.value,
    handleResolveIdle,
  };
});

const homeCtx = computed(() => ({
  tx,
  formatSeconds,
  todayLearnSeconds: todayLearnSeconds.value,
  todayRestSeconds: todayRestSeconds.value,
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
  openGuardWorkflow,
  pendingRuleCount: pendingRuleProcesses.value.length,
  idlePromptCount: idlePrompts.value.length,
  dueReminderCount: dueReminderCount.value,
  homeMonthRhythmBars: homeMonthRhythmBars.value,
  homeMonthGoalProgress: homeMonthGoalProgress.value,
  homeMonthActiveStreakDays: homeMonthActiveStreakDays.value,
  homePendingSummary: homePendingSummary.value,
}) satisfies HomeViewContext & Record<string, unknown>);

const insightsCtx = computed(() => ({
  tx,
  formatSeconds,
  cleanProcessName,
  formatClock,
  historySubViews: historySubViews.value,
  historySubView: historySubView.value,
  switchHistorySubView,
  topApps: topApps.value,
  topAppsBarWidth,
  allTimeTopApps: allTimeTopApps.value,
  setAllTimeFilter,
  allTimeIncludeIgnore: allTimeIncludeIgnore.value,
  onAllTimeIgnoreToggle,
  allTimeBarWidth,
  recentTimelineGroups: recentTimelineGroups.value,
  recentDurationWidth,
}) satisfies InsightsViewContext & Record<string, unknown>);

const guardCtx = computed<GuardViewContext>(() => ({
  tx,
  guardCurrentStepText: guardCurrentStepText.value,
  guardStepLabels: guardStepLabels.value,
  guardCurrentStepIndex: guardCurrentStepIndex.value,
  autoCaptureEnabled: autoCaptureEnabled.value,
  onAutoCaptureToggle,
  pendingRuleProcesses: pendingRuleProcesses.value,
  cleanProcessName,
  formatClock,
  formatSeconds,
  handleSavePendingRule,
  guardStep2Unlocked: guardStep2Unlocked.value,
  idlePrompts: idlePrompts.value,
  formatIdlePromptSpan,
  idleActionLoading: idleActionLoading.value,
  handleResolveIdle,
  guardStep3Unlocked: guardStep3Unlocked.value,
  ruleSearch: ruleSearch.value,
  onRuleSearchInput,
  ruleSort: ruleSort.value,
  onRuleSortChange,
  filteredSortedRules: filteredSortedRules.value,
  onRuleMappedTypeChange,
  handleUpdateExistingRule,
  markGuardStep3Done,
  guardStep3Done: guardStep3Done.value,
  guardStep4Unlocked: guardStep4Unlocked.value,
  foregroundDiagnostics: foregroundDiagnostics.value,
  captureBlockReasonText,
  captureRuleText,
  canSaveRuleFromDiagnostic,
  handleSaveRuleFromDiagnostic,
}));

const settingsCtx = computed<SettingsViewContext>(() => ({
  tx,
  locale: locale.value,
  onLocaleChange,
  themeMode: themeMode.value,
  toggleThemeMode,
  autoStartEnabled: autoStartEnabled.value,
  onAutoStartChange,
  privacy: privacy.value,
  handleSavePrivacySettings,
  privacyFeedbackType: privacyFeedbackType.value,
  privacyFeedback: privacyFeedback.value,
  whitelistInput: whitelistInput.value,
  onWhitelistInput,
  handleAddWhitelist,
  whitelist: whitelist.value,
  handleRemoveWhitelist,
}));

onMounted(async () => {
  initLocale();
  try {
    initThemeMode();
  } catch {
    applyTheme("light");
  }

  await loadHeatmapGoalSecondsSetting();

  resetGuardFeedback();
  resetPrivacyFeedback();
  void refreshHomeData();
  settingsWarmTimer = window.setTimeout(() => {
    privacyViewMounted.value = true;
    void refreshSettingsData();
  }, 1200);
  pollTimer = window.setInterval(() => {
    void refreshHomeData();
    if (currentMainView.value === "guard") {
      void refreshGuardData();
    }
  }, 5000);

  startAutoCaptureSampler();

  navigateSectionUnlisten = await listen<string>("navigate-insights-section", (event) => {
    void scrollToInsightsSection(event.payload || "heatmap");
  });
});

watch(locale, (next, prev) => {
  if (next === prev) {
    return;
  }
  applyLocale(next);
});

onUnmounted(() => {
  if (pollTimer !== null) {
    window.clearInterval(pollTimer);
    pollTimer = null;
  }
  stopAutoCaptureSampler();
  if (settingsWarmTimer !== null) {
    window.clearTimeout(settingsWarmTimer);
    settingsWarmTimer = null;
  }
  if (navigateSectionUnlisten) {
    navigateSectionUnlisten();
    navigateSectionUnlisten = null;
  }
  cleanupInsightsSectionNavigation();
  cleanupHeatmapGoalSetting();
});
</script>

<template>
  <main class="layout">
    <AppTopNav
      :tx="tx"
      :main-views="mainViews"
      :current-main-view="currentMainView"
      @select="setMainView"
    />

    <div class="content-scroll">
      <IdlePromptBanner v-if="idlePromptBannerCtx" :ctx="idlePromptBannerCtx" />

      <HomeView v-if="currentMainView === 'home'" :ctx="homeCtx" />
      <InsightsView v-if="currentMainView === 'insights'" :ctx="insightsCtx" />
      <GuardView v-if="currentMainView === 'guard'" :ctx="guardCtx" />

      <SettingsView v-if="privacyViewMounted" v-show="currentMainView === 'privacy'" :ctx="settingsCtx" />

      <p v-if="error" class="error">{{ error }}</p>
    </div>
  </main>
</template>
