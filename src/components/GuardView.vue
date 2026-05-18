<script setup lang="ts">
const props = defineProps<{ ctx: any }>();
</script>

<template>
  <section class="card guard-card">
    <h2>{{ props.ctx.tx("专注守护", "Focus Guard") }}</h2>

    <div class="guard-card-body">
      <p class="hint">{{ props.ctx.guardCurrentStepText }}</p>

      <div class="guard-stepper" role="list" :aria-label="props.ctx.tx('守护流程步骤', 'Guard workflow steps')">
        <div
          v-for="(label, idx) in props.ctx.guardStepLabels"
          :key="`guard-step-${idx + 1}`"
          class="guard-step"
          :class="{
            done: props.ctx.guardCurrentStepIndex > idx + 1,
            active: props.ctx.guardCurrentStepIndex === idx + 1,
            locked: props.ctx.guardCurrentStepIndex < idx + 1,
          }"
        >
          <span class="step-index">{{ idx + 1 }}</span>
          <span class="step-label">{{ label }}</span>
        </div>
      </div>

      <div class="toggle-row">
        <input id="auto-capture" type="checkbox" :checked="props.ctx.autoCaptureEnabled" @change="props.ctx.onAutoCaptureToggle" />
        <label for="auto-capture">{{ props.ctx.tx("自动采样前台窗口（每5秒）", "Auto-capture foreground window (every 5s)") }}</label>
      </div>

      <div class="guard-grid">
        <article class="guard-panel">
          <h3>{{ props.ctx.tx("待处理软件（最近10个）", "Pending Apps (Latest 10)") }}</h3>
          <ul class="pending-list panel-scroll" style="margin-top: 6px;">
            <li v-for="item in props.ctx.pendingRuleProcesses" :key="`pending-${item.process_name}`">
              <div class="recent-main">
                <strong>{{ props.ctx.cleanProcessName(item.process_name) }}</strong>
                <span class="muted">{{ props.ctx.formatClock(item.last_seen_timestamp) }}</span>
              </div>
              <div class="recent-sub">
                <span>{{ item.last_window_title }}</span>
                <span class="muted">{{ props.ctx.formatSeconds(item.total_seconds) }}</span>
              </div>
              <div class="diag-actions">
                <button type="button" @click="props.ctx.handleSavePendingRule(item, 'LEARN')">{{ props.ctx.tx("设为学习", "Set Learn") }}</button>
                <button type="button" @click="props.ctx.handleSavePendingRule(item, 'REST')">{{ props.ctx.tx("设为休息", "Set Break") }}</button>
                <button type="button" @click="props.ctx.handleSavePendingRule(item, 'IGNORE')">{{ props.ctx.tx("设为未分类", "Set Unclassified") }}</button>
              </div>
            </li>
            <li v-if="props.ctx.pendingRuleProcesses.length === 0" class="muted">{{ props.ctx.tx("已处理完当前已记录软件", "All recorded apps are processed") }}</li>
          </ul>
        </article>

        <article class="guard-panel" :class="{ locked: !props.ctx.guardStep2Unlocked }">
          <h3>{{ props.ctx.tx("离开时段待确认队列", "Idle Segment Queue") }}</h3>
          <p v-if="!props.ctx.guardStep2Unlocked" class="lock-note">{{ props.ctx.tx("请先完成步骤1：处理待分类软件。", "Complete Step 1 first: process pending apps.") }}</p>
          <ul class="pending-list panel-scroll" style="margin-top: 6px;">
            <li v-for="item in props.ctx.idlePrompts" :key="`idle-${item.id}`">
              <div class="recent-main">
                <strong>{{ props.ctx.tx("离开时段", "Idle Segment") }}</strong>
                <span class="muted">#{{ item.id }}</span>
              </div>
              <div class="recent-sub">
                <span>{{ props.ctx.formatIdlePromptSpan(item) }}</span>
              </div>
              <div class="diag-actions">
                <button type="button" :disabled="props.ctx.idleActionLoading || !props.ctx.guardStep2Unlocked" @click="props.ctx.handleResolveIdle('LEARN', item.id)">{{ props.ctx.tx("学习", "Learn") }}</button>
                <button type="button" :disabled="props.ctx.idleActionLoading || !props.ctx.guardStep2Unlocked" @click="props.ctx.handleResolveIdle('REST', item.id)">{{ props.ctx.tx("休息", "Break") }}</button>
                <button type="button" :disabled="props.ctx.idleActionLoading || !props.ctx.guardStep2Unlocked" @click="props.ctx.handleResolveIdle('IDLE', item.id)">{{ props.ctx.tx("离开", "Away") }}</button>
                <button type="button" :disabled="props.ctx.idleActionLoading || !props.ctx.guardStep2Unlocked" @click="props.ctx.handleResolveIdle('SKIP', item.id)">{{ props.ctx.tx("稍后提醒", "Remind later") }}</button>
              </div>
            </li>
            <li v-if="props.ctx.idlePrompts.length === 0" class="muted">{{ props.ctx.tx("暂无待确认离开时段", "No idle segments pending") }}</li>
          </ul>
        </article>

        <article class="guard-panel guard-panel-wide" :class="{ locked: !props.ctx.guardStep3Unlocked }">
          <h3>{{ props.ctx.tx("已有规则（可编辑）", "Existing Rules (Editable)") }}</h3>
          <p v-if="!props.ctx.guardStep3Unlocked" class="lock-note">{{ props.ctx.tx("请先完成步骤2：确认离开时段。", "Complete Step 2 first: resolve idle segments.") }}</p>
          <div class="rule-toolbar" style="margin-top: 6px;">
            <input :value="props.ctx.ruleSearch" @input="props.ctx.onRuleSearchInput" :placeholder="props.ctx.tx('搜索进程名，如 code.exe', 'Search process, e.g. code.exe')" />
            <select :value="props.ctx.ruleSort" @change="props.ctx.onRuleSortChange">
              <option value="alpha_asc">{{ props.ctx.tx("按首字母 A-Z", "Name A-Z") }}</option>
              <option value="alpha_desc">{{ props.ctx.tx("按首字母 Z-A", "Name Z-A") }}</option>
              <option value="time_desc">{{ props.ctx.tx("按时间 新到旧", "Time New-Old") }}</option>
              <option value="time_asc">{{ props.ctx.tx("按时间 旧到新", "Time Old-New") }}</option>
            </select>
          </div>
          <ul class="rule-list rule-scroll" style="margin-top: 6px;">
            <li v-for="rule in props.ctx.filteredSortedRules" :key="`rule-${rule.process_name}`">
              <div class="rule-main">
                <strong>{{ props.ctx.cleanProcessName(rule.process_name) }}</strong>
                <span class="muted">{{ rule.privacy_level }} · {{ props.ctx.formatClock(rule.updated_at) }}</span>
              </div>
              <div class="rule-actions">
                <select :value="rule.mapped_type" :disabled="!props.ctx.guardStep3Unlocked" @change="props.ctx.onRuleMappedTypeChange(rule, $event)">
                  <option value="LEARN">{{ props.ctx.tx("学习", "Learn") }}</option>
                  <option value="REST">{{ props.ctx.tx("休息", "Break") }}</option>
                  <option value="IGNORE">{{ props.ctx.tx("未分类", "Unclassified") }}</option>
                </select>
                <button type="button" :disabled="!props.ctx.guardStep3Unlocked" @click="props.ctx.handleUpdateExistingRule(rule)">{{ props.ctx.tx("保存修改", "Save") }}</button>
              </div>
            </li>
            <li v-if="props.ctx.filteredSortedRules.length === 0" class="muted">{{ props.ctx.tx("无匹配规则", "No matched rules") }}</li>
          </ul>
          <div class="actions" style="margin-top: 8px;">
            <button type="button" :disabled="!props.ctx.guardStep3Unlocked" @click="props.ctx.markGuardStep3Done">
              {{ props.ctx.guardStep3Done ? props.ctx.tx("规则复核已完成", "Rules review completed") : props.ctx.tx("标记规则复核完成", "Mark rules review complete") }}
            </button>
          </div>
        </article>

        <article class="guard-panel" :class="{ locked: !props.ctx.guardStep4Unlocked }">
          <h3>{{ props.ctx.tx("采样诊断（最近10次）", "Sampling Diagnostics (Latest 10)") }}</h3>
          <p v-if="!props.ctx.guardStep4Unlocked" class="lock-note">{{ props.ctx.tx("请先完成步骤3：复核已有规则。", "Complete Step 3 first: review existing rules.") }}</p>
          <ul class="recent-list recent-scroll" style="margin-top: 6px;">
            <li v-for="item in props.ctx.foregroundDiagnostics" :key="`diag-${item.id}`">
              <div class="recent-main">
                <strong>{{ props.ctx.cleanProcessName(item.observed_process_name) }}</strong>
                <span class="muted">{{ props.ctx.formatClock(Math.floor(item.captured_at_ms / 1000)) }}</span>
              </div>
              <div class="recent-sub">
                <span>{{ item.stored ? props.ctx.tx('已入库', 'Stored') : props.ctx.tx('未入库', 'Not stored') }} · {{ props.ctx.captureBlockReasonText(item.block_reason) }}</span>
                <span class="muted">{{ item.observed_window_title }}</span>
              </div>
              <div class="recent-sub">
                <span class="diag-rule-tag" :class="{ unsaved: !item.rule_saved }">
                  {{ props.ctx.tx("规则：", "Rule: ") }}{{ props.ctx.captureRuleText(item) }}
                </span>
                <span v-if="!item.rule_saved" class="muted">{{ props.ctx.tx("可一键保存分类规则", "One-click save rule") }}</span>
              </div>
              <div v-if="!item.rule_saved && props.ctx.canSaveRuleFromDiagnostic(item)" class="diag-actions">
                <button type="button" :disabled="!props.ctx.guardStep4Unlocked" @click="props.ctx.handleSaveRuleFromDiagnostic(item, 'LEARN')">{{ props.ctx.tx("设为学习", "Set Learn") }}</button>
                <button type="button" :disabled="!props.ctx.guardStep4Unlocked" @click="props.ctx.handleSaveRuleFromDiagnostic(item, 'REST')">{{ props.ctx.tx("设为休息", "Set Break") }}</button>
                <button type="button" :disabled="!props.ctx.guardStep4Unlocked" @click="props.ctx.handleSaveRuleFromDiagnostic(item, 'IGNORE')">{{ props.ctx.tx("设为未分类", "Set Unclassified") }}</button>
              </div>
            </li>
            <li v-if="props.ctx.foregroundDiagnostics.length === 0" class="muted">{{ props.ctx.tx("暂无采样诊断记录", "No diagnostic records yet") }}</li>
          </ul>
        </article>
      </div>
    </div>
  </section>
</template>
