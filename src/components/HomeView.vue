<script setup lang="ts">
import { useHomeReminderPanel } from "../composables/useHomeReminderPanel";
import { useHomeRhythmTooltip } from "../composables/useHomeRhythmTooltip";
import type { HomeViewContext } from "./viewContexts";

const props = defineProps<{ ctx: HomeViewContext }>();

const {
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
} = useHomeReminderPanel(() => props.ctx);

const {
  compactRhythmDuration,
  handleRhythmEnter,
  handleRhythmLeave,
  handleRhythmMove,
  homeRhythmSummary,
  rhythmTooltip,
} = useHomeRhythmTooltip(() => props.ctx);
</script>

<template>
  <section class="card overview-card overview-card-hero">
    <div class="overview-grid overview-grid-hero">
      <article class="overview-metric overview-summary-card">
        <div class="overview-card-head">
          <span class="metric-label section-title">{{ props.ctx.tx("当前概览", "Current Overview") }}</span>
          <span class="status-chip" :class="`status-chip-${props.ctx.currentStatusTone}`">
            <span class="status-dot" :class="props.ctx.currentStatusTone"></span>
            {{ props.ctx.tx("实时状态", "Live Status") }}
          </span>
        </div>

        <div class="overview-summary-panel">
          <span class="overview-kicker">{{ props.ctx.tx("当前状态", "Current status") }}</span>
          <p class="overview-status-main"><strong>{{ props.ctx.currentStatusLabel }}</strong></p>
          <div class="status-recent status-recent-panel">
            <span class="metric-label status-recent-title">{{ props.ctx.tx("最近记录", "Recent") }}</span>
            <p class="hint status-recent-value">{{ props.ctx.recentSummary }}</p>
          </div>
        </div>

        <div class="overview-signal-grid">
          <div class="overview-signal-card">
            <span class="overview-kicker">{{ props.ctx.tx("待分类", "Pending apps") }}</span>
            <strong>{{ props.ctx.pendingRuleCount }}</strong>
          </div>
          <div class="overview-signal-card">
            <span class="overview-kicker">{{ props.ctx.tx("待确认", "Idle review") }}</span>
            <strong>{{ props.ctx.idlePromptCount }}</strong>
          </div>
          <div class="overview-signal-card">
            <span class="overview-kicker">{{ props.ctx.tx("提醒", "Reminders") }}</span>
            <strong>{{ props.ctx.dueReminderCount }}</strong>
          </div>
        </div>

        <p class="overview-card-footnote hint">{{ props.ctx.tx("把今天的状态看清楚，节奏就会更稳。", "See today's state clearly, and the rhythm stays steady.") }}</p>
      </article>

      <article class="overview-goal overview-goal-hero">
        <div class="overview-card-head overview-card-head-plain">
          <span class="metric-label section-title">{{ props.ctx.tx("今日目标", "Today's Goal") }}</span>
        </div>
        <div class="overview-goal-main overview-goal-main-hero overview-goal-main-compact">
          <div class="overview-goal-visual">
            <div class="overview-goal-ring-wrap">
              <div
                class="overview-goal-pie"
                :class="[`overflow-${props.ctx.goalOverflowTier}`]"
                :style="{ '--goal-angle': `${props.ctx.goalProgressFillNum * 3.6}deg` }"
              >
                <strong>{{ props.ctx.goalProgressPct }}</strong>
              </div>
            </div>
            <div class="overview-goal-readout">
              <strong class="overview-goal-readout-value">{{ props.ctx.formatSeconds(props.ctx.todayLearnSeconds) }}</strong>
            </div>
          </div>
        </div>
        <p class="overview-card-footnote hint">{{ props.ctx.tx("把注意力稳稳推进到今天的目标线。", "Keep moving your attention steadily toward today's target.") }}</p>
      </article>

      <article class="overview-status overview-schedule overview-schedule-hero">
        <div class="schedule-headline">
          <span class="metric-label section-title">{{ props.ctx.tx("今日日程", "Today's Schedule") }}</span>
          <button type="button" class="schedule-icon-btn" @click="openReminderComposer" :title="props.ctx.tx('添加日程', 'Add reminder')">
            <span aria-hidden="true">+</span>
          </button>
        </div>
        <div class="schedule-list-wrap">
          <ul class="schedule-line-list">
            <li
              v-for="item in props.ctx.homeScheduleItems"
              :key="`home-reminder-${item.id}`"
              :class="{
                done: item.done,
                dragging: draggingReminderId === item.id,
                'drop-target': dropTargetReminderId === item.id && draggingReminderId !== item.id,
              }"
              @dragenter.prevent="handleReminderDragEnter(item)"
              @dragover.prevent="handleReminderDragEnter(item)"
              @drop.prevent="handleReminderDrop(item, $event)"
            >
              <button
                type="button"
                class="schedule-line-check"
                :class="{ done: item.done }"
                :aria-label="item.done ? props.ctx.tx('恢复日程', 'Restore reminder') : props.ctx.tx('完成日程', 'Complete reminder')"
                @click="quickDoneReminder(item.id, !item.done)"
              >
                <span aria-hidden="true">{{ item.done ? "✓" : "" }}</span>
              </button>
              <div class="schedule-line-content">
                <div class="schedule-line-top">
                  <span class="schedule-line-title" :class="{ done: item.done }">{{ item.content }}</span>
                  <div class="schedule-line-tools">
                    <button
                      type="button"
                      class="schedule-line-tool"
                      :title="props.ctx.tx('编辑日程', 'Edit reminder')"
                      @click="startEditReminder(item)"
                    >
                      <span aria-hidden="true">&#9998;</span>
                    </button>
                    <button
                      type="button"
                      class="schedule-line-tool schedule-line-drag"
                      :title="props.ctx.tx('拖拽排序', 'Drag to reorder')"
                      draggable="true"
                      @dragstart="handleReminderDragStart(item, $event)"
                      @dragend="handleReminderDragEnd"
                    >
                      <span aria-hidden="true">&#8942;</span>
                    </button>
                  </div>
                </div>
                <span class="schedule-line-due">{{ props.ctx.reminderDueText(item) }}</span>
              </div>
            </li>
            <li v-if="props.ctx.homeScheduleItems.length === 0" class="schedule-empty muted">
              {{ props.ctx.tx("今日暂无日程。", "No schedule for today.") }}
            </li>
          </ul>
        </div>

        <p class="schedule-support hint">{{ props.ctx.tx("保持今天的节奏，逐项完成就好。", "Keep the rhythm today and finish one item at a time.") }}</p>

        <Teleport to="body">
          <div v-if="scheduleModalOpen" class="schedule-popover-mask" @click.self="closeScheduleSettings">
            <section class="schedule-popover card">
              <header class="schedule-popover-head">
                <h3>{{ reminderEditId === null ? props.ctx.tx("新建日程", "New Reminder") : props.ctx.tx("编辑日程", "Edit Reminder") }}</h3>
                <button type="button" class="schedule-close-btn" @click="closeScheduleSettings">&times;</button>
              </header>

              <div class="row schedule-row-compact" style="margin-top: 2px;">
                <label for="home-reminder-content">{{ props.ctx.tx("事项内容", "Task content") }}</label>
                <input id="home-reminder-content" v-model="reminderDraftContent" />
              </div>

              <div class="schedule-form-grid schedule-form-grid-compact">
                <div class="row schedule-row-compact" style="margin-bottom: 0;">
                  <label for="home-reminder-repeat">{{ props.ctx.tx("重复", "Repeat") }}</label>
                  <select id="home-reminder-repeat" v-model="reminderDraftRepeat">
                    <option value="NONE">{{ props.ctx.tx("一次性", "One-time") }}</option>
                    <option value="DAILY">{{ props.ctx.tx("每日", "Daily") }}</option>
                    <option value="WEEKLY">{{ props.ctx.tx("每周", "Weekly") }}</option>
                  </select>
                </div>

                <div class="row schedule-row-compact" style="margin-bottom: 0;">
                  <label>{{ props.ctx.tx("需要提醒", "Enable reminder") }}</label>
                  <label class="schedule-switch">
                    <input type="checkbox" v-model="reminderEnabled" />
                    <span class="schedule-switch-ui"></span>
                  </label>
                </div>
              </div>

              <div v-if="reminderDraftRepeat === 'WEEKLY'" class="row schedule-row-compact schedule-weekdays-row">
                <label>{{ props.ctx.tx("每周日期", "Weekdays") }}</label>
                <div class="weekday-chip-row">
                  <button
                    v-for="day in weekdayOptions"
                    :key="`weekday-${day.value}`"
                    type="button"
                    class="weekday-chip"
                    :class="{ active: reminderDraftWeeklyDays.includes(day.value) }"
                    @click="toggleWeeklyDay(day.value)"
                  >
                    {{ weekdayLabel(day) }}
                  </button>
                </div>
              </div>

              <div v-if="reminderEnabled" class="row schedule-row-compact" style="margin-bottom: 0;">
                <label v-if="reminderDraftRepeat === 'NONE'" for="home-reminder-once">{{ props.ctx.tx("提醒时间", "Reminder time") }}</label>
                <label v-else for="home-reminder-daily">{{ props.ctx.tx("提醒时间", "Reminder time") }}</label>
                <input
                  v-if="reminderDraftRepeat === 'NONE'"
                  id="home-reminder-once"
                  v-model="reminderDraftAt"
                  type="datetime-local"
                />
                <input
                  v-else
                  id="home-reminder-daily"
                  v-model="reminderDraftDailyTime"
                  type="time"
                  step="60"
                />
              </div>

              <div class="actions schedule-actions" style="margin-top: 8px;">
                <button type="button" :disabled="props.ctx.reminderActionLoading" @click="saveReminderFromModal">
                  {{ reminderEditId === null ? props.ctx.tx("创建日程", "Create") : props.ctx.tx("保存修改", "Save") }}
                </button>
                <button type="button" class="btn-secondary" :disabled="props.ctx.reminderActionLoading" @click="resetReminderDraft">
                  {{ props.ctx.tx("重置", "Reset") }}
                </button>
              </div>
              <p v-if="reminderFeedback" class="guard-feedback" :class="reminderFeedbackType">{{ reminderFeedback }}</p>

              <ul class="rule-list reminder-scroll compact-reminder-list">
                <li v-for="item in props.ctx.reminderListForPanel" :key="`home-setting-reminder-${item.id}`" class="reminder-row">
                  <div class="reminder-row-head">
                    <strong>{{ item.content }}</strong>
                    <span class="muted">{{ props.ctx.reminderDueText(item) }}</span>
                  </div>
                  <div class="reminder-actions">
                    <button type="button" :disabled="props.ctx.reminderActionLoading" @click="startEditReminder(item)">{{ props.ctx.tx("编辑", "Edit") }}</button>
                    <button type="button" :disabled="props.ctx.reminderActionLoading" @click="quickDoneReminder(item.id, !item.done)">
                      {{ item.done ? props.ctx.tx("恢复", "Restore") : props.ctx.tx("完成", "Done") }}
                    </button>
                    <button type="button" :disabled="props.ctx.reminderActionLoading" @click="quickSnoozeReminder(item.id)">{{ props.ctx.tx("稍后10分钟", "Snooze 10m") }}</button>
                    <button type="button" :disabled="props.ctx.reminderActionLoading" @click="quickDeleteReminder(item.id)">{{ props.ctx.tx("删除", "Delete") }}</button>
                  </div>
                </li>
              </ul>
            </section>
          </div>
        </Teleport>
      </article>

    </div>
  </section>

  <section class="home-focus-grid">
    <article class="card home-module-card home-calendar-card">
      <div class="home-module-header">
        <h3>{{ props.ctx.tx("学习日历", "Learning Calendar") }}</h3>
      </div>

      <div class="calendar-nav-row">
        <div class="month-nav-wrap">
          <button class="month-nav" @click="props.ctx.shiftHeatmapMonth(-1)">&lt;</button>
          <strong>{{ props.ctx.monthTitleText }}</strong>
          <button class="month-nav" @click="props.ctx.shiftHeatmapMonth(1)">&gt;</button>
        </div>
      </div>

      <div class="week-header compact-week-header">
        <span v-for="w in props.ctx.weekHeaders" :key="`home-week-${w}`">{{ w }}</span>
      </div>
      <div class="heatmap-grid compact-heatmap-grid">
        <div
          v-for="(cell, idx) in props.ctx.calendarHeatmapCells"
          :key="cell ? `home-${cell.day}` : `home-pad-${idx}`"
          :class="cell ? props.ctx.heatmapCellClass(cell) : ['heat-cell', 'pad-cell']"
          :title="cell ? `${cell.day} | ${props.ctx.formatSeconds(cell.learn_seconds)}` : ''"
        >
          <span v-if="cell" class="day-label compact-day-label">{{ props.ctx.heatmapDayText(cell.day) }}</span>
        </div>
      </div>
    </article>

    <article class="card home-module-card home-rhythm-card">
      <div class="home-module-header">
        <h3>{{ props.ctx.tx("最近节奏", "Recent Rhythm") }}</h3>
      </div>

      <section class="rhythm-glance-card">
        <div class="rhythm-glance-head">
          <span class="rhythm-section-title">{{ props.ctx.tx("最近7天节奏", "Recent 7-day rhythm") }}</span>
          <span class="rhythm-mini-tag">{{ props.ctx.tx("轻摘要", "Summary") }}</span>
        </div>
        <div class="rhythm-summary-grid">
          <div class="rhythm-summary-chip">
            <span class="rhythm-summary-label">{{ props.ctx.tx("7天累计", "7-day total") }}</span>
            <strong>{{ homeRhythmSummary.totalText }}</strong>
          </div>
          <div class="rhythm-summary-chip">
            <span class="rhythm-summary-label">{{ props.ctx.tx("日均投入", "Daily avg") }}</span>
            <strong>{{ homeRhythmSummary.averageText }}</strong>
          </div>
          <div class="rhythm-summary-chip">
            <span class="rhythm-summary-label">{{ props.ctx.tx("活跃天数", "Active days") }}</span>
            <strong>{{ homeRhythmSummary.activeDays }} / 7</strong>
          </div>
        </div>
        <p class="rhythm-summary-note">{{ props.ctx.tx("最佳单日", "Best day") }} {{ homeRhythmSummary.bestText }}</p>
        <div class="rhythm-bars">
          <div
            v-for="bar in props.ctx.homeMonthRhythmBars"
            :key="`rhythm-${bar.day}`"
            class="rhythm-bar-col"
            @mouseleave="handleRhythmLeave"
          >
            <span class="rhythm-bar-value" :class="{ today: bar.isToday }">{{ compactRhythmDuration(bar.totalSeconds) }}</span>
            <div class="rhythm-bar-track">
              <div class="rhythm-bar-stack" :class="{ today: bar.isToday }" :style="{ height: bar.totalHeight }">
                <div
                  v-if="bar.restSeconds > 0"
                  class="rhythm-bar rhythm-bar-rest"
                  :style="{ height: bar.restHeight }"
                  @mouseenter="handleRhythmEnter($event, bar, 'rest')"
                  @mousemove="handleRhythmMove($event)"
                ></div>
                <div
                  v-if="bar.learnSeconds > 0"
                  class="rhythm-bar rhythm-bar-learn"
                  :style="{ height: bar.learnHeight }"
                  @mouseenter="handleRhythmEnter($event, bar, 'learn')"
                  @mousemove="handleRhythmMove($event)"
                ></div>
              </div>
            </div>
            <span class="rhythm-bar-label" :class="{ today: bar.isToday }">{{ bar.label }}</span>
          </div>
        </div>
      </section>
    </article>
  </section>

  <Teleport to="body">
    <div
      v-if="rhythmTooltip?.visible"
      class="rhythm-hover-tooltip"
      :style="{
        left: `${rhythmTooltip.x}px`,
        top: `${rhythmTooltip.y}px`,
        background: rhythmTooltip.bgColor,
        borderColor: rhythmTooltip.borderColor,
      }"
    >
      <span class="rhythm-hover-kicker" :style="{ color: rhythmTooltip.accentColor }">{{ props.ctx.tx(`第 ${rhythmTooltip.dayLabel} 天`, `Day ${rhythmTooltip.dayLabel}`) }}</span>
      <strong :style="{ color: rhythmTooltip.accentColor }">{{ rhythmTooltip.segmentLabel }} · {{ rhythmTooltip.durationText }}</strong>
      <p class="rhythm-hover-share">{{ props.ctx.tx("占当天电脑使用", "Share of day total") }} <span>{{ rhythmTooltip.shareText }}</span></p>
      <p class="rhythm-hover-tone">{{ rhythmTooltip.toneText }}</p>
      <p class="rhythm-hover-context">{{ rhythmTooltip.contextText }}</p>
    </div>
  </Teleport>

</template>
