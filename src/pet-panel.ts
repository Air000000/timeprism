import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getLearnHeatmap, getUsageStack, type LearnHeatmapCell, type UsageStackDay } from "./api";
import { getStoredOrBrowserLocale, translateForLocale, type LocaleCode } from "./lib/locale";
import {
  queryPetPanelElements,
  renderPetPanelShell,
  renderPetPanelWeekHeaders,
} from "./lib/petPanelDom";
import {
  buildPetPanelHeatmapDayCells,
  getHeatmapFetchDays,
  heatCellClass,
  monthCellRows,
  petPanelHeatmapMonthLabel,
  petPanelHeatmapPadCount,
  pickCurrentBusinessDay,
} from "./lib/petPanelMetrics";
import {
  buildPetPanelStackParts,
  petPanelStackPercent,
  petPanelStackSignature,
} from "./lib/petPanelStack";
import {
  petPanelBusinessDayText,
  petPanelEmptyStackText,
  petPanelModeTitle,
  petPanelTotalText,
  petPanelWeekHeaders,
} from "./lib/petPanelText";
import { formatSeconds } from "./lib/time";
import "./pet-panel.css";

function getLocale(): LocaleCode {
  return getStoredOrBrowserLocale();
}

function tx(zh: string, en: string): string {
  return translateForLocale(getLocale(), zh, en);
}

const root = document.querySelector<HTMLDivElement>("#pet-panel-app");
if (!root) {
  throw new Error("pet panel root not found");
}

renderPetPanelShell(root, tx);
const {
  panelTitle,
  panelHeatmap,
  panelStack,
  heatMonthLabel,
  miniWeekHeader,
  miniHeatmapGrid,
  heatPrevMonthBtn,
  heatNextMonthBtn,
} = queryPetPanelElements();

function renderWeekHeaders() {
  renderPetPanelWeekHeaders(miniWeekHeader, petPanelWeekHeaders(getLocale()));
}

let mode: "heatmap" | "stack" = "heatmap";
let timer: number | null = null;
let lastStackSig = "";
let active = true;
const viewMonthDate = new Date(new Date().getFullYear(), new Date().getMonth(), 1);

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
  heatMonthLabel.textContent = petPanelHeatmapMonthLabel(viewMonthDate);

  for (let i = 0; i < petPanelHeatmapPadCount(viewMonthDate); i += 1) {
    const pad = document.createElement("div");
    pad.className = "mini-heat-cell pad";
    miniHeatmapGrid.appendChild(pad);
  }

  for (const { dayKey, cell, isToday } of buildPetPanelHeatmapDayCells(cells, viewMonthDate)) {
    const node = document.createElement("div");
    node.className = `mini-heat-cell ${heatCellClass(cell.level)}`;
    if (isToday) {
      node.classList.add("today");
    }
    node.title = `${dayKey} ${formatSeconds(cell.learn_seconds)}`;
    miniHeatmapGrid.appendChild(node);
  }
}

function renderStack(days: UsageStackDay[]) {
  const day = pickCurrentBusinessDay(days);
  if (!day || day.total_seconds <= 0) {
    panelStack.replaceChildren();
    const empty = document.createElement("div");
    empty.className = "mini-empty";
    empty.textContent = petPanelEmptyStackText(tx);
    panelStack.appendChild(empty);
    return;
  }

  const sig = petPanelStackSignature(day);
  if (sig === lastStackSig) {
    return;
  }
  lastStackSig = sig;
  panelStack.replaceChildren();

  const dayText = document.createElement("div");
  dayText.className = "stack-day";
  dayText.textContent = petPanelBusinessDayText(day.day, tx);

  const parts = buildPetPanelStackParts(day, tx);

  const bar = document.createElement("div");
  bar.className = "mini-stack-bar";
  for (const item of parts) {
    const pct = petPanelStackPercent(item.seconds, day.total_seconds);
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
  meta.textContent = petPanelTotalText(day.total_seconds, tx);

  panelStack.append(dayText, bar, meta);
}

function updateMode(next: "heatmap" | "stack") {
  mode = next;
  panelTitle.textContent = petPanelModeTitle(next, tx);
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
      const cells = await getLearnHeatmap(getHeatmapFetchDays(viewMonthDate), 7200);
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
