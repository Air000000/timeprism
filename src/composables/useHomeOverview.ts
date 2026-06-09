import { computed, ref, type Ref } from "vue";
import type {
  IdlePrompt,
  PendingRuleProcess,
  RecentLog,
  Reminder,
  TodaySummary,
} from "../api";

type TranslateFn = (zh: string, en: string) => string;

type UseHomeOverviewOptions = {
  tx: TranslateFn;
  learnGoalSliderMinutes: Ref<number>;
  recentLogs: Ref<RecentLog[]>;
  reminders: Ref<Reminder[]>;
  pendingRuleProcesses: Ref<PendingRuleProcess[]>;
  idlePrompts: Ref<IdlePrompt[]>;
  autoCaptureEnabled: Ref<boolean>;
  cleanProcessName: (name: string) => string;
  formatClock: (unixSeconds: number) => string;
};

export function useHomeOverview({
  tx,
  learnGoalSliderMinutes,
  recentLogs,
  reminders,
  pendingRuleProcesses,
  idlePrompts,
  autoCaptureEnabled,
  cleanProcessName,
  formatClock,
}: UseHomeOverviewOptions) {
  const todaySummary = ref<TodaySummary>({
    learn_seconds: 0,
    rest_seconds: 0,
    active_session_id: null,
  });

  const todayLearnSeconds = computed(() => todaySummary.value.learn_seconds ?? 0);
  const todayRestSeconds = computed(() => todaySummary.value.rest_seconds ?? 0);
  const goalProgressRatio = computed(() => {
    const goal = Math.max(0, learnGoalSliderMinutes.value * 60);
    if (goal <= 0) {
      return 0;
    }
    return Math.max(0, todayLearnSeconds.value / goal);
  });
  const goalProgressPct = computed(() => `${(goalProgressRatio.value * 100).toFixed(0)}%`);
  const goalProgressNum = computed(() => Math.round(goalProgressRatio.value * 100));
  const goalProgressFillNum = computed(() => Math.max(0, Math.min(100, goalProgressNum.value)));
  const goalOverflowTier = computed<"none" | "active">(() => (
    goalProgressNum.value > 100 ? "active" : "none"
  ));

  const recentSummary = computed(() => {
    const latest = recentLogs.value[0];
    if (!latest) {
      return tx("暂无最近记录", "No recent records");
    }
    return `${cleanProcessName(latest.process_name)} · ${formatClock(latest.start_timestamp)} · ${Math.max(
      0,
      Math.floor(latest.duration_ms / 1000),
    )}s`;
  });

  const dueReminderCount = computed(() => {
    const now = Math.floor(Date.now() / 1000);
    return reminders.value.filter((item) => !item.done && item.next_due_timestamp <= now).length;
  });

  const currentStatusLabel = computed(() => {
    if (idlePrompts.value.length > 0) {
      return tx("离开时段待确认", "Idle segments pending");
    }
    if (dueReminderCount.value > 0) {
      return tx(`到点提醒 ${dueReminderCount.value} 条`, `${dueReminderCount.value} reminders due`);
    }
    if (pendingRuleProcesses.value.length > 0) {
      return tx("待处理软件规则", "App rules pending");
    }
    if (autoCaptureEnabled.value) {
      return tx("自动采样运行中", "Auto capture running");
    }
    return tx("自动采样已暂停", "Auto capture paused");
  });

  const currentStatusTone = computed<"ok" | "warn" | "alert" | "idle">(() => {
    if (idlePrompts.value.length > 0) {
      return "alert";
    }
    if (dueReminderCount.value > 0 || pendingRuleProcesses.value.length > 0) {
      return "warn";
    }
    if (autoCaptureEnabled.value) {
      return "ok";
    }
    return "idle";
  });

  const homePendingSummary = computed(() => {
    const parts: string[] = [];
    if (pendingRuleProcesses.value.length > 0) {
      parts.push(tx(`待分类 ${pendingRuleProcesses.value.length}`, `${pendingRuleProcesses.value.length} pending apps`));
    }
    if (idlePrompts.value.length > 0) {
      parts.push(tx(`待确认 ${idlePrompts.value.length}`, `${idlePrompts.value.length} idle reviews`));
    }
    if (dueReminderCount.value > 0) {
      parts.push(tx(`提醒 ${dueReminderCount.value}`, `${dueReminderCount.value} reminders`));
    }
    if (parts.length === 0) {
      return tx("当前没有新的待处理项，首页会保持安静。", "No pending items right now, so home stays quiet.");
    }
    return parts.join(" · ");
  });

  return {
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
  };
}
