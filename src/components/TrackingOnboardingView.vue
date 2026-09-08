<script setup lang="ts">
type TranslateFn = (zh: string, en: string) => string;

const props = defineProps<{
  tx: TranslateFn;
  loading: boolean;
  error: string;
  onStart: () => Promise<void> | void;
}>();
</script>

<template>
  <main class="onboarding-shell">
    <section class="onboarding-card" aria-labelledby="tracking-onboarding-title">
      <header class="onboarding-header">
        <div class="brand-mark" aria-hidden="true">TP</div>
        <div class="brand-copy">
          <span class="eyebrow">TimePrism · {{ props.tx("本地优先", "Local-first") }}</span>
          <h1 id="tracking-onboarding-title">
            {{ props.tx("先确认记录边界，再开始使用", "Confirm the tracking boundary before you begin") }}
          </h1>
          <p>
            {{
              props.tx(
                "TimePrism 用前台应用采样帮助你回顾时间与注意力。首次启动默认不采样，只有你点击下方按钮后才会开始。",
                "TimePrism samples the foreground app to help you review time and attention. First launch starts with capture off; sampling begins only after you continue below.",
              )
            }}
          </p>
        </div>
      </header>

      <div class="privacy-grid">
        <article class="privacy-item">
          <span class="privacy-index">01</span>
          <div>
            <h2>{{ props.tx("数据保存在本地", "Stored locally") }}</h2>
            <p>
              {{
                props.tx(
                  "当前版本的记录写入本机 SQLite 数据库，用于统计、时间线与专注分析。",
                  "The current version writes tracking records to a local SQLite database for summaries, timelines, and focus analysis.",
                )
              }}
            </p>
          </div>
        </article>

        <article class="privacy-item">
          <span class="privacy-index">02</span>
          <div>
            <h2>{{ props.tx("先过隐私规则，再决定是否保存", "Privacy rules run before storage") }}</h2>
            <p>
              {{
                props.tx(
                  "浏览器标题可模糊处理，应用也可被忽略或限制；被拦截的样本不会写入使用记录。",
                  "Browser titles can be blurred and apps can be ignored or restricted; blocked samples are not written to usage history.",
                )
              }}
            </p>
          </div>
        </article>

        <article class="privacy-item">
          <span class="privacy-index">03</span>
          <div>
            <h2>{{ props.tx("随时暂停，状态会保留", "Pause anytime, persistently") }}</h2>
            <p>
              {{
                props.tx(
                  "开始后可在专注守护中暂停或恢复自动采样；选择会持久化，下次启动继续沿用。",
                  "After activation, pause or resume auto capture from Focus Guard. The choice persists across restarts.",
                )
              }}
            </p>
          </div>
        </article>
      </div>

      <div class="activation-note">
        <strong>{{ props.tx("点击开始后", "After you continue") }}</strong>
        <p>
          {{
            props.tx(
              "TimePrism 会启用约每 5 秒一次的前台应用采样，并显示桌面宠物。无需重启。",
              "TimePrism enables foreground-app sampling at roughly five-second intervals and shows the desktop pet. No restart is required.",
            )
          }}
        </p>
      </div>

      <p v-if="props.error" class="onboarding-error" role="alert">{{ props.error }}</p>

      <footer class="onboarding-actions">
        <span>
          {{
            props.tx(
              "继续即表示你已了解上述本地记录方式。",
              "Continuing confirms that you understand the local tracking behavior above.",
            )
          }}
        </span>
        <button
          type="button"
          class="start-button"
          :disabled="props.loading"
          @click="props.onStart"
        >
          {{
            props.loading
              ? props.tx("正在启用…", "Enabling…")
              : props.tx("开始使用 TimePrism", "Start using TimePrism")
          }}
        </button>
      </footer>
    </section>
  </main>
</template>

<style scoped>
.onboarding-shell {
  width: 860px;
  max-width: 100%;
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  padding: 28px;
  display: grid;
  place-items: center;
  overflow: auto;
}

.onboarding-card {
  width: min(100%, 760px);
  padding: 30px;
  display: grid;
  gap: 22px;
  border: 1px solid var(--card-edge);
  border-radius: 28px;
  background: var(--card-bg);
  box-shadow: var(--shadow-soft), var(--inner-top);
  backdrop-filter: blur(18px);
}

.onboarding-header {
  display: grid;
  grid-template-columns: 62px minmax(0, 1fr);
  gap: 18px;
  align-items: start;
}

.brand-mark {
  width: 62px;
  height: 62px;
  border-radius: 19px;
  display: grid;
  place-items: center;
  color: #f3fbff;
  background: linear-gradient(145deg, #2ab6a8 0%, #3b74d2 100%);
  box-shadow: 0 12px 26px rgba(41, 126, 173, 0.24);
  font-family: Arial, sans-serif;
  font-size: 20px;
  font-weight: 800;
  letter-spacing: 0.6px;
}

.brand-copy {
  display: grid;
  gap: 8px;
}

.eyebrow {
  color: var(--accent-main);
  font-size: 12px;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  max-width: 600px;
  font-size: clamp(25px, 3.3vw, 34px);
  line-height: 1.18;
  letter-spacing: -0.025em;
}

.brand-copy p,
.privacy-item p,
.activation-note p,
.onboarding-actions span {
  color: var(--text-soft);
  line-height: 1.65;
}

.brand-copy p {
  max-width: 610px;
  font-size: 14px;
}

.privacy-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.privacy-item {
  min-width: 0;
  padding: 16px;
  display: grid;
  gap: 12px;
  align-content: start;
  border: 1px solid var(--panel-edge);
  border-radius: 18px;
  background: var(--panel-bg);
}

.privacy-index {
  width: 34px;
  height: 26px;
  display: grid;
  place-items: center;
  border-radius: 999px;
  color: var(--accent-main);
  background: color-mix(in srgb, var(--accent-main) 13%, transparent);
  font-size: 11px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

.privacy-item div {
  display: grid;
  gap: 6px;
}

.privacy-item h2 {
  font-size: 14px;
  line-height: 1.35;
}

.privacy-item p {
  font-size: 12px;
}

.activation-note {
  padding: 15px 17px;
  display: grid;
  gap: 5px;
  border: 1px solid color-mix(in srgb, var(--accent-main) 35%, var(--panel-edge));
  border-radius: 16px;
  background: color-mix(in srgb, var(--accent-main) 7%, var(--panel-bg));
}

.activation-note strong {
  font-size: 13px;
}

.activation-note p {
  font-size: 12px;
}

.onboarding-error {
  padding: 10px 12px;
  border: 1px solid color-mix(in srgb, var(--danger) 45%, transparent);
  border-radius: 12px;
  color: var(--danger);
  background: color-mix(in srgb, var(--danger) 8%, transparent);
  font-size: 12px;
}

.onboarding-actions {
  display: flex;
  gap: 18px;
  align-items: center;
  justify-content: space-between;
}

.onboarding-actions span {
  max-width: 390px;
  font-size: 11px;
}

.start-button {
  min-width: 190px;
  min-height: 44px;
  padding: 0 20px;
  border: 1px solid var(--button-edge);
  border-radius: 14px;
  color: var(--button-text);
  background: var(--button-bg);
  box-shadow: var(--inner-top);
  font: inherit;
  font-size: 13px;
  font-weight: 800;
  cursor: pointer;
  transition: transform 120ms ease, background 120ms ease, opacity 120ms ease;
}

.start-button:hover:not(:disabled) {
  background: var(--button-hover);
  transform: translateY(-1px);
}

.start-button:disabled {
  cursor: wait;
  opacity: 0.62;
}

@media (max-width: 720px) {
  .onboarding-shell {
    padding: 18px;
  }

  .onboarding-card {
    padding: 22px;
  }

  .privacy-grid {
    grid-template-columns: 1fr;
  }

  .onboarding-actions {
    align-items: stretch;
    flex-direction: column;
  }

  .start-button {
    width: 100%;
  }
}
</style>
