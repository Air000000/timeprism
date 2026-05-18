import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getLearnHeatmap, getUsageStack, type LearnHeatmapCell, type UsageStackDay } from "./api";
import "./pet-panel.css";

const LOCALE_STORAGE_KEY = "timeprism-locale";
type LocaleCode = "zh-CN" | "en-US";

function getLocale(): LocaleCode {
  try {
    const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
    if (stored === "zh-CN" || stored === "en-US") {
      return stored;
    }
  } catch {
    // Ignore storage read failures.
  }
  const browserLang = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-cn";
  return browserLang.startsWith("zh") ? "zh-CN" : "en-US";
}

function tx(zh: string, en: string): string {
  return getLocale() === "zh-CN" ? zh : en;
}

const root = document.querySelector<HTMLDivElement>("#pet-panel-app");
if (!root) {
  throw new Error("pet panel root not found");
}

root.innerHTML = `
  <section class="panel-shell">
    <div id="panelTitle" class="panel-title">${tx("图表面板", "Panel")}</div>
    <div id="panelHeatmap" class="mini-heatmap">
      <div class="mini-head-row">
        <button id="heatPrevMonth" class="mini-month-btn" type="button">‹</button>
        <strong id="heatMonthLabel" class="mini-month-label">-</strong>
        <button id="heatNextMonth" class="mini-month-btn" type="button">›</button>
      </div>
      <div id="miniWeekHeader" class="mini-week-header"></div>
      <div id="miniHeatmapGrid" class="mini-heatmap-grid"></div>
    </div>
    <div id="panelStack" class="mini-stack" style="display:none"></div>
  </section>
`;

const panelTitle = document.querySelector<HTMLElement>("#panelTitle")!;
const panelHeatmap = document.querySelector<HTMLElement>("#panelHeatmap")!;
const panelStack = document.querySelector<HTMLElement>("#panelStack")!;
const heatMonthLabel = document.querySelector<HTMLElement>("#heatMonthLabel")!;
const miniWeekHeader = document.querySelector<HTMLElement>("#miniWeekHeader")!;
const miniHeatmapGrid = document.querySelector<HTMLElement>("#miniHeatmapGrid")!;
const heatPrevMonthBtn = document.querySelector<HTMLButtonElement>("#heatPrevMonth")!;
const heatNextMonthBtn = document.querySelector<HTMLButtonElement>("#heatNextMonth")!;

function renderWeekHeaders() {
  miniWeekHeader.replaceChildren();
  const weekHeaders = getLocale() === "zh-CN"
    ? ["日", "一", "二", "三", "四", "五", "六"]
    : ["S", "M", "T", "W", "T", "F", "S"];
  for (const w of weekHeaders) {
    const node = document.createElement("span");
    node.textContent = w;
    miniWeekHeader.appendChild(node);
  }
}

let mode: "heatmap" | "stack" = "heatmap";
let timer: number | null = null;
let lastStackSig = "";
let active = true;
const viewMonthDate = new Date(new Date().getFullYear(), new Date().getMonth(), 1);

function formatSeconds(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const h = Math.floor(safe / 3600).toString().padStart(2, "0");
  const m = Math.floor((safe % 3600) / 60).toString().padStart(2, "0");
  const s = Math.floor(safe % 60).toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

function heatCellClass(level: LearnHeatmapCell["level"]): string {
  if (level === "GREEN") return "green";
  if (level === "YELLOW") return "yellow";
  return "gray";
}

function businessDayKeyNow(): string {
  const now = new Date();
  const shifted = new Date(now.getTime() - 4 * 3600 * 1000);
  const y = shifted.getFullYear();
  const m = (shifted.getMonth() + 1).toString().padStart(2, "0");
  const d = shifted.getDate().toString().padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function getHeatmapFetchDays(): number {
  const now = new Date();
  const viewStart = new Date(viewMonthDate.getFullYear(), viewMonthDate.getMonth(), 1);
  const diffMs = now.getTime() - viewStart.getTime();
  const diffDays = Math.max(0, Math.ceil(diffMs / 86_400_000));
  const days = diffDays + 62;
  return Math.min(720, Math.max(120, days));
}

function monthCellRows(year: number, month: number): number {
  const firstWeekday = new Date(year, month - 1, 1).getDay();
  const monthDays = new Date(year, month, 0).getDate();
  return Math.ceil((firstWeekday + monthDays) / 7);
}

async function resizePanelForMode(nextMode: "heatmap" | "stack") {
  try {
    if (nextMode === "stack") {
      await invoke("resize_pet_panel", { width: 172, height: 118 });
      return;
    }

    const year = viewMonthDate.getFullYear();
    const month = viewMonthDate.getMonth() + 1;
    const rows = monthCellRows(year, month);
    const height = 88 + rows * 12;
    await invoke("resize_pet_panel", { width: 172, height });
  } catch (e) {
    console.error("[pet-panel] resize failed", e);
  }
}

function renderHeatmap(cells: LearnHeatmapCell[]) {
  miniHeatmapGrid.replaceChildren();
  const now = new Date();
  const year = viewMonthDate.getFullYear();
  const month = viewMonthDate.getMonth() + 1;
  const monthKey = `${year}-${month.toString().padStart(2, "0")}`;
  heatMonthLabel.textContent = `${year}/${month.toString().padStart(2, "0")}`;

  const monthDays = new Date(year, month, 0).getDate();
  const firstWeekday = new Date(year, month - 1, 1).getDay();
  const byDay = new Map(cells.map((cell) => [cell.day, cell]));
  const todayKey = `${year}-${month.toString().padStart(2, "0")}-${now.getDate().toString().padStart(2, "0")}`;

  for (let i = 0; i < firstWeekday; i += 1) {
    const pad = document.createElement("div");
    pad.className = "mini-heat-cell pad";
    miniHeatmapGrid.appendChild(pad);
  }

  for (let day = 1; day <= monthDays; day += 1) {
    const dayKey = `${monthKey}-${day.toString().padStart(2, "0")}`;
    const cell = byDay.get(dayKey) ?? { day: dayKey, learn_seconds: 0, level: "GRAY" as const };
    const node = document.createElement("div");
    node.className = `mini-heat-cell ${heatCellClass(cell.level)}`;
    if (dayKey === todayKey) {
      node.classList.add("today");
    }
    node.title = `${dayKey} ${formatSeconds(cell.learn_seconds)}`;
    miniHeatmapGrid.appendChild(node);
  }
}

function pickCurrentBusinessDay(days: UsageStackDay[]): UsageStackDay | null {
  if (days.length === 0) {
    return null;
  }
  const key = businessDayKeyNow();
  const exact = days.find((d) => d.day === key);
  if (exact) {
    return exact;
  }
  const sorted = [...days].sort((a, b) => b.day.localeCompare(a.day));
  return sorted[0] ?? null;
}

function renderStack(days: UsageStackDay[]) {
  const day = pickCurrentBusinessDay(days);
  if (!day || day.total_seconds <= 0) {
    panelStack.replaceChildren();
    const empty = document.createElement("div");
    empty.className = "mini-empty";
    empty.textContent = tx("暂无今日色块", "No stack data for today");
    panelStack.appendChild(empty);
    return;
  }

  const sig = `${day.day}|${day.total_seconds}|${day.learn_seconds}|${day.rest_seconds}`;
  if (sig === lastStackSig) {
    return;
  }
  lastStackSig = sig;
  panelStack.replaceChildren();

  const dayText = document.createElement("div");
  dayText.className = "stack-day";
  dayText.textContent = tx(`业务日 ${day.day}`, `Business day ${day.day}`);

  const ignore = Math.max(0, day.total_seconds - day.learn_seconds - day.rest_seconds);
  const parts = [
    { name: tx("学", "L"), seconds: day.learn_seconds, color: "#16a34a" },
    { name: tx("休", "B"), seconds: day.rest_seconds, color: "#8ec5ff" },
    { name: tx("未", "U"), seconds: ignore, color: "#64748b" },
  ].filter((item) => item.seconds > 0);

  const bar = document.createElement("div");
  bar.className = "mini-stack-bar";
  for (const item of parts) {
    const pct = day.total_seconds > 0 ? (item.seconds / day.total_seconds) * 100 : 0;
    const seg = document.createElement("div");
    seg.className = "mini-stack-seg";
    seg.style.width = `${Math.max(6, pct)}%`;
    seg.style.background = item.color;
    seg.title = `${item.name} ${formatSeconds(item.seconds)}`;
    if (pct >= 16) {
      const label = document.createElement("span");
      label.className = "mini-stack-pct";
      label.textContent = `${Math.round(pct)}%`;
      seg.appendChild(label);
    }
    bar.appendChild(seg);
  }

  const meta = document.createElement("div");
  meta.className = "mini-stack-meta";
  meta.textContent = tx(`总计 ${formatSeconds(day.total_seconds)}`, `Total ${formatSeconds(day.total_seconds)}`);

  panelStack.append(dayText, bar, meta);
}

function updateMode(next: "heatmap" | "stack") {
  mode = next;
  panelTitle.textContent = next === "heatmap"
    ? tx("学习日历", "Learning Calendar")
    : tx("周活跃", "Weekly Activity");
  panelHeatmap.style.display = next === "heatmap" ? "grid" : "none";
  panelStack.style.display = next === "stack" ? "grid" : "none";
  heatPrevMonthBtn.style.display = next === "heatmap" ? "inline-grid" : "none";
  heatNextMonthBtn.style.display = next === "heatmap" ? "inline-grid" : "none";
  void resizePanelForMode(next);
}

async function refresh() {
  if (!active) {
    return;
  }
  renderWeekHeaders();
  updateMode(mode);
  try {
    if (mode === "heatmap") {
      const cells = await getLearnHeatmap(getHeatmapFetchDays(), 7200);
      renderHeatmap(cells);
      return;
    }

    const days = await getUsageStack(8, "ALL");
    renderStack(days);
  } catch (e) {
    console.error("[pet-panel] refresh failed", e);
  }
}

void listen<string>("pet-panel-mode", (event) => {
  const payload = (event.payload || "").toLowerCase();
  if (payload === "stack") {
    updateMode("stack");
  } else {
    updateMode("heatmap");
  }
  void refresh();
});

heatPrevMonthBtn.addEventListener("click", () => {
  const y = viewMonthDate.getFullYear();
  const m = viewMonthDate.getMonth();
  viewMonthDate.setFullYear(y, m - 1, 1);
  void resizePanelForMode("heatmap");
  void refresh();
});

heatNextMonthBtn.addEventListener("click", () => {
  const y = viewMonthDate.getFullYear();
  const m = viewMonthDate.getMonth();
  viewMonthDate.setFullYear(y, m + 1, 1);
  void resizePanelForMode("heatmap");
  void refresh();
});

void listen<boolean>("pet-panel-active", (event) => {
  active = !!event.payload;
  if (active) {
    void refresh();
  }
});

updateMode("heatmap");
renderWeekHeaders();
void refresh();
timer = window.setInterval(() => {
  void refresh();
}, 5000);

window.addEventListener("beforeunload", () => {
  if (timer !== null) {
    window.clearInterval(timer);
    timer = null;
  }
});
