<script setup lang="ts">
import { computed, watch } from "vue";
import { summonPetWindow } from "./api";
import AppTopNav from "./components/AppTopNav.vue";
import GuardView from "./components/GuardView.vue";
import HomeView from "./components/HomeView.vue";
import IdlePromptBanner from "./components/IdlePromptBanner.vue";
import InsightsView from "./components/InsightsView.vue";
import SettingsView from "./components/SettingsView.vue";
import TrackingOnboardingView from "./components/TrackingOnboardingView.vue";
import { useAutoCaptureSampler } from "./composables/useAutoCaptureSampler";
import { useAppNavigation } from "./composables/useAppNavigation";
import { useAppSettingsSection } from "./composables/useAppSettingsSection";
import { useDisplayFormatters } from "./composables/useDisplayFormatters";
import { useErrorMessage } from "./composables/useErrorMessage";
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
import { useLocale } from "./composables/useLocale";
import { useMainDataBuffers } from "./composables/useMainDataBuffers";
import { useMainDataRefresh } from "./composables/useMainDataRefresh";
import { useMainRefreshPolling } from "./composables/useMainRefreshPolling";
import { useMainViewActions } from "./composables/useMainViewActions";
import { useMainWindowLifecycle } from "./composables/useMainWindowLifecycle";
import { useReminders } from "./composables/useReminders";
import { useThemeMode } from "./composables/useThemeMode";
import { useTrackingState } from "./composables/useTrackingState";
import {
  formatSeconds,
  timeMinutesLabel,
  toDateTimeLocalValue,
} from "./lib/time";
import { trackingUiMode } from "./lib/trackingUi";

const { locale, tx, initLocale, watchLocaleChanges, onLocaleChange } = useLocale();
watchLocaleChanges();
const { themeMode, toggleThemeMode, initThemeModeSafely } = useThemeMode();
const { error, setErrorMessage, clearErrorMessage } = useErrorMessage();
const {
  trackingReady,
  onboardingCompleted,
  autoCaptureEnabled,
  captureShouldRun,
  trackingActionLoading,
  loadTrackingState,
  completeOnboarding,
  persistAutoCaptureEnabled,
} = useTrackingState({ setErrorMessage });
const trackingMode = computed(() =>
  trackingUiMode(trackingReady.value, onboardingCompleted.value),
);

async function handleStartTracking(): Promise<void> {
  try {
    await completeOnboarding();
  } catch {
    return;
  }

  try {
    await summonPetWindow();
  } catch (e) {
    setErrorMessage(e);
  }
}

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
const { bindMainDataRefreshHandlers, refreshData } = useMainDataRefresh(currentMainView);
const {
  startInsightsSectionNavigationListener,
  cleanupInsightsSectionNavigation,
} = useInsightsSectionNavigation({
  currentMainView,
  selectHistoryView,
});
const {
  privacyViewMounted,
  settingsCtx,
  showSettingsViewAndRefresh,
  startSettingsWarmup,
  cleanupSettingsWarmup,
  refreshSettingsData,
  resetPrivacyFeedback,
} = useAppSettingsSection({
  tx,
  locale,
  onLocaleChange,
  themeMode,
  toggleThemeMode,
  refreshData,
  setErrorMessage,
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

const {
  recentLogs,
  learnHeatmap,
  homeUsageStack,
} = useMainDataBuffers();
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
const {
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
  autoCaptureEnabled,
  persistAutoCaptureEnabled,
});
const { switchHistorySubView, setMainView } = useMainViewActions({
  selectHistoryView,
  selectMainView,
  refreshInsightsData,
  refreshGuardData,
  showSettingsViewAndRefresh,
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
  startAutoCaptureSampler,
  stopAutoCaptureSampler,
} = useAutoCaptureSampler({
  autoCaptureFeedback,
  tx,
});
watch(
  captureShouldRun,
  (shouldRun) => {
    if (shouldRun) {
      startAutoCaptureSampler();
    } else {
      stopAutoCaptureSampler();
    }
  },
  { immediate: true },
);
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
  clearErrorMessage,
});
bindMainDataRefreshHandlers({
  refreshHomeData,
  refreshInsightsData,
  refreshGuardData,
  refreshSettingsData,
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

useMainWindowLifecycle({
  initLocale,
  initThemeModeSafely,
  loadTrackingState,
  loadHeatmapGoalSecondsSetting,
  resetGuardFeedback,
  resetPrivacyFeedback,
  refreshHomeData,
  startSettingsWarmup,
  startMainRefreshPolling,
  startInsightsSectionNavigationListener,
  stopMainRefreshPolling,
  stopAutoCaptureSampler,
  cleanupSettingsWarmup,
  cleanupInsightsSectionNavigation,
  cleanupHeatmapGoalSetting,
});
</script>

<template>
  <main v-if="trackingMode === 'loading'" class="tracking-bootstrap-shell" aria-live="polite">
    <section class="tracking-bootstrap-card">
      <div class="tracking-bootstrap-mark" aria-hidden="true">TP</div>
      <div>
        <h1>{{ tx("正在准备 TimePrism", "Preparing TimePrism") }}</h1>
        <p>
          {{
            tx(
              "正在读取本地记录设置。自动采样会保持关闭，直到状态确认完成。",
              "Reading local tracking settings. Auto capture remains off until the state is confirmed.",
            )
          }}
        </p>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>
  </main>

  <TrackingOnboardingView
    v-else-if="trackingMode === 'onboarding'"
    :tx="tx"
    :loading="trackingActionLoading"
    :error="error"
    :on-start="handleStartTracking"
  />

  <main v-else class="layout">
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

<style scoped>
.tracking-bootstrap-shell {
  width: 860px;
  max-width: 100%;
  height: 100%;
  margin: 0 auto;
  padding: 28px;
  display: grid;
  place-items: center;
}

.tracking-bootstrap-card {
  width: min(100%, 520px);
  padding: 24px;
  display: grid;
  grid-template-columns: 54px minmax(0, 1fr);
  gap: 16px;
  align-items: center;
  border: 1px solid var(--card-edge);
  border-radius: 22px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft), var(--inner-top);
  backdrop-filter: blur(16px);
}

.tracking-bootstrap-mark {
  width: 54px;
  height: 54px;
  border-radius: 17px;
  display: grid;
  place-items: center;
  color: #f3fbff;
  background: linear-gradient(145deg, #2ab6a8 0%, #3b74d2 100%);
  font-family: Arial, sans-serif;
  font-weight: 800;
}

.tracking-bootstrap-card div:nth-child(2) {
  display: grid;
  gap: 5px;
}

.tracking-bootstrap-card h1,
.tracking-bootstrap-card p {
  margin: 0;
}

.tracking-bootstrap-card h1 {
  font-size: 18px;
}

.tracking-bootstrap-card p {
  color: var(--text-soft);
  font-size: 12px;
  line-height: 1.55;
}

.tracking-bootstrap-card .error {
  grid-column: 1 / -1;
  color: var(--danger);
}
</style>
