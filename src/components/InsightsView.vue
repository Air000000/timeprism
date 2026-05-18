<script setup lang="ts">
const props = defineProps<{ ctx: any }>();
</script>

<template>
  <section id="insights-stack-anchor" class="card insight-pane">
    <h3 style="margin-top: 14px;">{{ props.ctx.tx("历史记录深度区", "History Deep View") }}</h3>

    <div class="insight-pane-body">
      <div class="history-subnav" role="tablist" :aria-label="props.ctx.tx('历史分段导航', 'History segmented navigation')">
        <button
          v-for="view in props.ctx.historySubViews"
          :key="`history-${view.key}`"
          type="button"
          class="insight-subnav-btn"
          :class="{ active: props.ctx.historySubView === view.key }"
          @click="props.ctx.switchHistorySubView(view.key)"
        >
          {{ view.label }}
        </button>
      </div>

      <div v-if="props.ctx.historySubView === 'topApps'" class="insight-section">
        <h4>{{ props.ctx.tx("今日高频应用", "Top Apps (Today)") }}</h4>
        <p class="hint" v-if="props.ctx.topApps.length > 0">
          {{ props.ctx.tx("时长最高：", "Top app: ") }}{{ props.ctx.cleanProcessName(props.ctx.topApps[0].process_name) }}（{{ props.ctx.formatSeconds(props.ctx.topApps[0].seconds) }}）
        </p>
        <ul class="alltime-list">
          <li v-for="app in props.ctx.topApps" :key="`top-app-${app.process_name}`">
            <div class="alltime-head">
              <span>{{ props.ctx.cleanProcessName(app.process_name) }}</span>
              <strong>{{ props.ctx.formatSeconds(app.seconds) }}</strong>
            </div>
            <div class="alltime-track">
              <div class="alltime-fill" :style="{ width: props.ctx.topAppsBarWidth(app.seconds) }"></div>
            </div>
          </li>
          <li v-if="props.ctx.topApps.length === 0" class="muted">{{ props.ctx.tx("暂无应用使用记录。", "No app usage records yet.") }}</li>
        </ul>
      </div>

      <div v-if="props.ctx.historySubView === 'allTime'" class="insight-section">
        <h4>{{ props.ctx.tx("应用历史总时长", "All-time App Usage") }}</h4>
        <div class="actions" style="margin-bottom: 6px;">
          <button @click="props.ctx.setAllTimeFilter('ALL')">{{ props.ctx.tx("全部", "All") }}</button>
          <button @click="props.ctx.setAllTimeFilter('LEARN')">{{ props.ctx.tx("仅学习", "Learn only") }}</button>
          <button @click="props.ctx.setAllTimeFilter('REST')">{{ props.ctx.tx("仅休息", "Break only") }}</button>
        </div>
        <div class="toggle-row" style="margin-top: 0; margin-bottom: 6px;">
          <input id="alltime-ignore" type="checkbox" :checked="props.ctx.allTimeIncludeIgnore" @change="props.ctx.onAllTimeIgnoreToggle" />
          <label for="alltime-ignore">{{ props.ctx.tx("包含未分类进程", "Include unclassified apps") }}</label>
        </div>
        <ul class="alltime-list">
          <li v-for="app in props.ctx.allTimeTopApps" :key="`all-time-${app.process_name}`">
            <div class="alltime-head">
              <span>{{ props.ctx.cleanProcessName(app.process_name) }}</span>
              <strong>{{ props.ctx.formatSeconds(app.seconds) }}</strong>
            </div>
            <div class="alltime-track">
              <div class="alltime-fill" :style="{ width: props.ctx.allTimeBarWidth(app.seconds) }"></div>
            </div>
          </li>
          <li v-if="props.ctx.allTimeTopApps.length === 0" class="muted">{{ props.ctx.tx("暂无历史累计数据", "No historical aggregate data") }}</li>
        </ul>
      </div>

      <div v-if="props.ctx.historySubView === 'recent'" class="insight-section">
        <h4>{{ props.ctx.tx("最近记录", "Recent Logs") }}</h4>
        <div class="recent-timeline recent-scroll">
          <section v-for="group in props.ctx.recentTimelineGroups" :key="`timeline-${group.day}`" class="timeline-group">
            <h5 class="timeline-day">{{ group.day }}</h5>
            <ul class="timeline-list">
              <li v-for="item in group.items" :key="`recent-${item.id}`" class="timeline-item">
                <div class="timeline-time">{{ props.ctx.formatClock(item.start_timestamp) }}</div>
                <div class="timeline-dot"></div>
                <div class="timeline-card">
                  <div class="recent-main">
                    <strong>{{ props.ctx.cleanProcessName(item.process_name) }}</strong>
                    <span class="muted">{{ Math.max(0, Math.floor(item.duration_ms / 1000)) }}s</span>
                  </div>
                  <div class="recent-sub">
                    <span>{{ item.window_title }}</span>
                  </div>
                  <div class="alltime-track" style="margin-top: 8px;">
                    <div class="alltime-fill" :style="{ width: props.ctx.recentDurationWidth(item.duration_ms) }"></div>
                  </div>
                </div>
              </li>
            </ul>
          </section>
          <p v-if="props.ctx.recentTimelineGroups.length === 0" class="muted">{{ props.ctx.tx("暂无日志记录", "No logs yet") }}</p>
        </div>
      </div>
    </div>
  </section>
</template>
