<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { cleanUiProcessName } from "../lib/displayFormatters";
import type { IdlePromptBannerContext } from "./viewContexts";

const props = defineProps<{ ctx: IdlePromptBannerContext }>();
const showOther = ref(false);

const attributionProcessName = computed(
  () => props.ctx.currentIdlePrompt.attribution_process_name,
);
const attributionAppName = computed(() => {
  const processName = attributionProcessName.value;
  return processName ? cleanUiProcessName(processName, props.ctx.tx) : "";
});

watch(
  () => props.ctx.currentIdlePrompt.id,
  () => {
    showOther.value = false;
  },
);
</script>

<template>
  <section class="card idle-inline" :aria-label="props.ctx.tx('离开时段待确认', 'Idle Segment Confirmation')">
    <h2>{{ props.ctx.tx("离开时段待确认", "Idle Segment Confirmation") }}</h2>
    <p class="hint">
      {{
        props.ctx.tx(
          "系统只知道这段时间没有键鼠输入，需要你确认实际发生了什么。",
          "TimePrism only knows there was no keyboard or mouse input; confirm what actually happened.",
        )
      }}
    </p>
    <p class="idle-span">{{ props.ctx.formatIdlePromptSpan(props.ctx.currentIdlePrompt) }}</p>

    <template v-if="attributionProcessName">
      <p class="hint" style="margin-top: 8px;">
        {{ props.ctx.tx("无输入前正在使用：", "App used before inactivity: ") }}<strong>{{ attributionAppName }}</strong>
      </p>
      <p style="margin-top: 6px;">
        {{
          props.ctx.tx(
            `这段时间你还在使用 ${attributionAppName} 吗？`,
            `Were you still using ${attributionAppName} during this time?`,
          )
        }}
      </p>
      <p class="hint" style="margin-top: 5px;">
        {{
          props.ctx.tx(
            `选择“继续使用 ${attributionAppName}”会把整段归入该应用，并沿用它现有的学习 / 休息 / 未分类规则。`,
            `Choosing “Continue ${attributionAppName}” attributes the whole segment to that app and keeps its existing Learn / Break / Unclassified rule.`,
          )
        }}
      </p>
      <div class="diag-actions" style="margin-top: 8px;">
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('APP')">
          {{ props.ctx.tx(`继续使用 ${attributionAppName}`, `Continue ${attributionAppName}`) }}
        </button>
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('IDLE')">
          {{ props.ctx.tx("离开电脑", "Away from computer") }}
        </button>
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="showOther = !showOther">
          {{ showOther ? props.ctx.tx("收起", "Less") : props.ctx.tx("其他…", "Other…") }}
        </button>
      </div>

      <div v-if="showOther" style="margin-top: 8px;">
        <p class="hint">
          {{
            props.ctx.tx(
              "如果只确定这段时间属于学习或休息，而不确定是否一直在这个应用里，可以只记录活动类型。",
              "If you only know this was Learn or Break time, record the activity type without attributing it to the app.",
            )
          }}
        </p>
        <div class="diag-actions" style="margin-top: 8px;">
          <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('LEARN')">
            {{ props.ctx.tx("学习", "Learn") }}
          </button>
          <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('REST')">
            {{ props.ctx.tx("休息", "Break") }}
          </button>
          <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('SKIP')">
            {{ props.ctx.tx("稍后处理", "Decide later") }}
          </button>
        </div>
        <div class="toggle-row" style="margin-top: 8px;">
          <input id="idle-remember-choice" type="checkbox" :checked="props.ctx.idleRememberChoice" @change="props.ctx.onIdleRememberChoiceChange" />
          <label for="idle-remember-choice">
            {{ props.ctx.tx("记住学习 / 休息 / 离开选择（不记住应用归因）", "Remember Learn / Break / Away choices (never app attribution)") }}
          </label>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="diag-actions" style="margin-top: 8px;">
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('LEARN')">
          {{ props.ctx.tx("学习", "Learn") }}
        </button>
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('REST')">
          {{ props.ctx.tx("休息", "Break") }}
        </button>
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('IDLE')">
          {{ props.ctx.tx("离开电脑", "Away from computer") }}
        </button>
        <button type="button" :disabled="props.ctx.idleActionLoading" @click="props.ctx.handleResolveIdle('SKIP')">
          {{ props.ctx.tx("稍后处理", "Decide later") }}
        </button>
      </div>
      <div class="toggle-row" style="margin-top: 8px;">
        <input id="idle-remember-choice" type="checkbox" :checked="props.ctx.idleRememberChoice" @change="props.ctx.onIdleRememberChoiceChange" />
        <label for="idle-remember-choice">
          {{ props.ctx.tx("记住本次选择（后续离开时段自动应用）", "Remember this choice for later idle segments") }}
        </label>
      </div>
    </template>
  </section>
</template>
