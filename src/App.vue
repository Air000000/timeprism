<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import AppTopNav from "./components/AppTopNav.vue";
import GuardView from "./components/GuardView.vue";
import HomeView from "./components/HomeView.vue";
import IdlePromptBanner from "./components/IdlePromptBanner.vue";
import InsightsView from "./components/InsightsView.vue";
import SettingsView from "./components/SettingsView.vue";
import { useAutoCaptureSampler } from "./composables/useAutoCaptureSampler";
import {
  useAppNavigation,
  type HistorySubViewKey,
  type MainViewKey,
} from "./composables/useAppNavigation";
import { useDisplayFormatters } from "./composables/useDisplayFormatters";
import { useGuardData } from "./composables/useGuardData";
import { useGuardViewContext } from "./composables/useGuardViewContext";
import { useGuardWorkflow } from "./composables/useGuardWorkflow";
import { useHeatmapCalendar } from "./composables/useHeatmapCalendar";
import { useHeatmapGoalSetting } from "./composables/useHeatmapGoalSetting";
import { useHomeData } from "./composables/useHomeData";
import { useHomeOverview } from "./composables/useHomeOverview";
import { useHomeRhythm } from "./composables/useHomeRhythm";
import { useHomeSchedule } from "./composables/useHomeSchedule";
import { useHomeViewContext } from "./composables/useHomeViewContext";
import { useIdlePromptBannerContext } from "./composables/useIdlePromptBannerContext";
import { useInsightsData } from "./composables/useInsightsData";
import { useInsightsSectionNavigation } from "./composables/useInsightsSectionNavigation";
import { useInsightsViewContext } from "./composables/useInsightsViewContext";
import { useLazySettingsMount } from "./composables/useLazySettingsMount";
import { useLocale } from "./composables/useLocale";
import { useMainRefreshPolling } from "./composables/useMainRefreshPolling";
import { useReminders } from "./composables/useReminders";
import { useSettingsPrivacy } from "./composables/useSettingsPrivacy";
import { useSettingsViewContext } from "./composables/useSettingsViewContext";
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

const { locale, tx, initLocale, watchLocaleChanges } = useLocale();
watchLocaleChanges();
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
} = useAppNavigation(tx);
const {
  startInsightsSectionNavigationListener,
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
  privacyViewMounted,
  showSettingsViewAndRefresh,
  startSettingsWarmup,
  cleanupSettingsWarmup,
} = useLazySettingsMount({ refreshSettingsData });
const settingsCtx = useSettingsViewContext({
  tx,
  locale,
  onLocaleChange,
  themeMode,
  toggleThemeMode,
  autoStartEnabled,
  onAutoStartChange,
  privacy,
  handleSavePrivacySettings,
  privacyFeedbackType,
  privacyFeedback,
  whitelistInput,
  onWhitelistInput,
  handleAddWhitelist,
  whitelist,
  handleRemoveWhitelist,
});
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
const insightsCtx = useInsightsViewContext({
  tx,
  formatSeconds,
  cleanProcessName,
  formatClock,
  historySubViews,
  historySubView,
  switchHistorySubView,
  topApps,
  topAppsBarWidth,
  allTimeTopApps,
  setAllTimeFilter,
  allTimeIncludeIgnore,
  onAllTimeIgnoreToggle,
  allTimeBarWidth,
  recentTimelineGroups,
  recentDurationWidth,
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
const idlePromptBannerCtx = useIdlePromptBannerContext({
  tx,
  currentIdlePrompt,
  formatIdlePromptSpan,
  idleRememberChoice,
  onIdleRememberChoiceChange,
  idleActionLoading,
  handleResolveIdle,
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
const guardCtx = useGuardViewContext({
  tx,
  guardCurrentStepText,
  guardStepLabels,
  guardCurrentStepIndex,
  autoCaptureEnabled,
  onAutoCaptureToggle,
  pendingRuleProcesses,
  cleanProcessName,
  formatClock,
  formatSeconds,
  handleSavePendingRule,
  guardStep2Unlocked,
  idlePrompts,
  formatIdlePromptSpan,
  idleActionLoading,
  handleResolveIdle,
  guardStep3Unlocked,
  ruleSearch,
  onRuleSearchInput,
  ruleSort,
  onRuleSortChange,
  filteredSortedRules,
  onRuleMappedTypeChange,
  handleUpdateExistingRule,
  markGuardStep3Done,
  guardStep3Done,
  guardStep4Unlocked,
  foregroundDiagnostics,
  captureBlockReasonText,
  captureRuleText,
  canSaveRuleFromDiagnostic,
  handleSaveRuleFromDiagnostic,
});

const {
  todaySummary,
  todayLearnSeconds,
  goalProgressPct,
  goalProgressFillNum,
  goalOverflowTier,
  recentSummary,
  dueReminderCount,
  currentStatusLabel,
  currentStatusTone,
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
const {
  startMainRefreshPolling,
  stopMainRefreshPolling,
} = useMainRefreshPolling({
  currentMainView,
  refreshHomeData,
  refreshGuardData,
});
const homeCtx = useHomeViewContext({
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
});

function switchHistorySubView(next: HistorySubViewKey) {
  selectHistoryView(next);
  void refreshInsightsData();
}

function setMainView(next: MainViewKey) {
  selectMainView(next);
  if (next === "insights") {
    void refreshInsightsData();
  } else if (next === "guard") {
    void refreshGuardData();
  } else if (next === "privacy") {
    showSettingsViewAndRefresh();
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

function onLocaleChange(event: Event) {
  const input = event.target as HTMLSelectElement;
  if (input.value === "zh-CN" || input.value === "en-US") {
    locale.value = input.value;
  }
}

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
  startSettingsWarmup();
  startMainRefreshPolling();

  startAutoCaptureSampler();

  await startInsightsSectionNavigationListener();
});

onUnmounted(() => {
  stopMainRefreshPolling();
  stopAutoCaptureSampler();
  cleanupSettingsWarmup();
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
