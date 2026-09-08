<script setup lang="ts">
import type { MainViewKey, MainViewOption } from "../composables/useAppNavigation";
import type { TranslateFn } from "./viewContexts";

defineProps<{
  tx: TranslateFn;
  mainViews: MainViewOption[];
  currentMainView: MainViewKey;
}>();

const emit = defineEmits<{
  (event: "select", key: MainViewKey): void;
}>();
</script>

<template>
  <section class="card top-shell fixed-top-nav">
    <div class="top-brand" :aria-label="tx('品牌区', 'Brand section')">
      <div class="brand-logo" aria-hidden="true">TP</div>
      <div class="brand-copy">
        <strong>Time Prism</strong>
        <span>{{ tx("Focus OS", "Focus OS") }}</span>
      </div>
    </div>

    <div class="top-nav" role="tablist" :aria-label="tx('主页面导航', 'Main Navigation')">
      <button
        v-for="view in mainViews"
        :key="view.key"
        type="button"
        class="top-nav-btn"
        :class="{ active: currentMainView === view.key }"
        @click="emit('select', view.key)"
      >
        {{ view.label }}
      </button>
    </div>
  </section>
</template>
