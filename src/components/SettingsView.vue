<script setup lang="ts">
import type { SettingsViewContext } from "./viewContexts";

const props = defineProps<{ ctx: SettingsViewContext }>();
</script>

<template>
  <section class="card privacy-card">
    <h2>{{ props.ctx.tx("设置", "Settings") }}</h2>
    <div class="privacy-scroll-shell">
      <div class="privacy-grid">
        <article class="guard-panel">
          <h3>{{ props.ctx.tx("界面与显示", "Interface & Display") }}</h3>
          <div class="row" style="margin-top: 4px;">
            <label for="language-select">{{ props.ctx.tx("界面语言", "Language") }}</label>
            <select id="language-select" :value="props.ctx.locale" @change="props.ctx.onLocaleChange">
              <option value="zh-CN">中文</option>
              <option value="en-US">English</option>
            </select>
          </div>
          <div class="row theme-toggle-row" style="margin-top: 2px;">
            <label>{{ props.ctx.tx("深色模式", "Dark mode") }}</label>
            <button type="button" class="compact-btn" @click="props.ctx.toggleThemeMode">
              {{ props.ctx.themeMode === "dark" ? props.ctx.tx("切换浅色", "Switch to Light") : props.ctx.tx("切换深色", "Switch to Dark") }}
            </button>
          </div>
          <div class="toggle-row">
            <input id="auto-start" type="checkbox" :checked="props.ctx.autoStartEnabled" @change="props.ctx.onAutoStartChange" />
            <label for="auto-start">{{ props.ctx.tx("开机自启", "Launch at startup") }}</label>
          </div>
        </article>

        <article class="guard-panel">
          <h3>{{ props.ctx.tx("采样策略", "Sampling Policy") }}</h3>
          <div class="toggle-row">
            <input id="curtain" type="checkbox" v-model="props.ctx.privacy.curtain_enabled" />
            <label for="curtain">{{ props.ctx.tx("拉窗帘模式（停止记录所有应用日志）", "Curtain mode (stop recording all app logs)") }}</label>
          </div>

          <div class="row" style="margin-top: 8px;">
            <label for="browser-title-mode">{{ props.ctx.tx("浏览器标题策略", "Browser title policy") }}</label>
            <select id="browser-title-mode" v-model="props.ctx.privacy.browser_title_mode">
              <option value="FULL">{{ props.ctx.tx("FULL - 记录完整标题", "FULL - Keep full title") }}</option>
              <option value="BLUR">{{ props.ctx.tx("BLUR - 模糊显示为 Web Browser", "BLUR - Mask as Web Browser") }}</option>
              <option value="NONE">{{ props.ctx.tx("NONE - 不采集标题", "NONE - No title capture") }}</option>
            </select>
            <p class="hint">{{ props.ctx.tx("建议默认 BLUR；如需最高隐私请选择 NONE。", "BLUR is recommended; choose NONE for maximum privacy.") }}</p>
          </div>

          <div class="toggle-row">
            <input id="whitelist-only" type="checkbox" v-model="props.ctx.privacy.whitelist_only_enabled" />
            <label for="whitelist-only">{{ props.ctx.tx("仅记录白名单进程", "Record only whitelisted apps") }}</label>
          </div>

          <div class="actions" style="margin-top: 8px;">
            <button @click="props.ctx.handleSavePrivacySettings">{{ props.ctx.tx("保存设置", "Save settings") }}</button>
          </div>
          <p class="guard-feedback" :class="props.ctx.privacyFeedbackType">{{ props.ctx.privacyFeedback }}</p>
        </article>

        <article class="guard-panel">
          <h3>{{ props.ctx.tx("白名单管理", "Whitelist") }}</h3>
          <div class="row" style="margin-top: 4px;">
            <label for="whitelist-input">{{ props.ctx.tx("添加白名单进程", "Add whitelist process") }}</label>
            <input id="whitelist-input" :value="props.ctx.whitelistInput" placeholder="e.g. pycharm64.exe" @input="props.ctx.onWhitelistInput" />
          </div>
          <div class="actions">
            <button @click="props.ctx.handleAddWhitelist">{{ props.ctx.tx("添加白名单", "Add") }}</button>
          </div>

          <ul class="whitelist-list whitelist-scroll">
            <li v-for="proc in props.ctx.whitelist" :key="proc">
              <span>{{ proc }}</span>
              <button @click="props.ctx.handleRemoveWhitelist(proc)">{{ props.ctx.tx("移除", "Remove") }}</button>
            </li>
            <li v-if="props.ctx.whitelist.length === 0" class="muted">{{ props.ctx.tx("白名单为空", "Whitelist is empty") }}</li>
          </ul>
        </article>
      </div>
    </div>
  </section>
</template>
