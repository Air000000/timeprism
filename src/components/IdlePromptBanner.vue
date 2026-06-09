<script setup lang="ts">
import type { IdlePromptBannerContext } from "./viewContexts";

const props = defineProps<{ ctx: IdlePromptBannerContext }>();
</script>

<template>
  <section class="card idle-inline" :aria-label="props.ctx.tx('离开时段待确认', 'Idle Segment Confirmation')">
    <h2>{{ props.ctx.tx("离开时段待确认", "Idle Segment Confirmation") }}</h2>
    <p class="hint">{{ props.ctx.tx("系统空闲超过5分钟后触发，已回补完整时段。", "Triggered after 5+ minutes idle; full segment has been backfilled.") }}</p>
    <p class="idle-span">{{ props.ctx.formatIdlePromptSpan(props.ctx.currentIdlePrompt) }}</p>
    <div class="toggle-row" style="margin-top: 8px;">
      <input id="idle-remember-choice" type="checkbox" :checked="props.ctx.idleRememberChoice" @change="props.ctx.onIdleRememberChoiceChange" />
      <label for="idle-remember-choice">{{ props.ctx.tx("记住本次选择（后续离开时段自动应用）", "Remember this choice for later idle segments") }}</label>
    </div>
    <div class="diag-actions" style="margin-top: 8px;">
      <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('LEARN')">{{ props.ctx.tx("标记为学习", "Mark as Learn") }}</button>
      <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('REST')">{{ props.ctx.tx("标记为休息", "Mark as Break") }}</button>
      <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('IDLE')">{{ props.ctx.tx("标记为离开", "Mark as Away") }}</button>
      <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('SKIP')">{{ props.ctx.tx("稍后提醒", "Remind me later") }}</button>
    </div>
  </section>
</template>
