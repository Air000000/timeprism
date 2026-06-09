import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getLearnHeatmap, getUsageStack, type LearnHeatmapCell, type UsageStackDay } from "./api";
import { getStoredOrBrowserLocale, translateForLocale, type LocaleCode } from "./lib/locale";
import {
  applyPetPanelMode,
  queryPetPanelElements,
  renderPetPanelEmptyStack,
  renderPetPanelHeatmapGrid,
  renderPetPanelShell,
  renderPetPanelStack,
  renderPetPanelWeekHeaders,
  type PetPanelMode,
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
const panelElements = queryPetPanelElements();
const {
  panelStack,
  heatMonthLabel,
  miniWeekHeader,
  miniHeatmapGrid,
  heatPrevMonthBtn,
  heatNextMonthBtn,
} = panelElements;

function renderWeekHeaders() {
  renderPetPanelWeekHeaders(miniWeekHeader, petPanelWeekHeaders(getLocale()));
}

let mode: PetPanelMode = "heatmap";
let timer: number | null = null;
let lastStackSig = "";
let active = true;
const viewMonthDate = new Date(new Date().getFullYear(), new Date().getMonth(), 1);

async function resizePanelForMode(nextMode: PetPanelMode) {
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
  heatMonthLabel.textContent = petPanelHeatmapMonthLabel(viewMonthDate);
  renderPetPanelHeatmapGrid(
    miniHeatmapGrid,
    petPanelHeatmapPadCount(viewMonthDate),
    buildPetPanelHeatmapDayCells(cells, viewMonthDate),
    heatCellClass,
    formatSeconds,
  );
}

function renderStack(days: UsageStackDay[]) {
  const day = pickCurrentBusinessDay(days);
  if (!day || day.total_seconds <= 0) {
    renderPetPanelEmptyStack(panelStack, petPanelEmptyStackText(tx));
    return;
  }

  const sig = petPanelStackSignature(day);
  if (sig === lastStackSig) {
    return;
  }
  lastStackSig = sig;
  renderPetPanelStack(
    panelStack,
    petPanelBusinessDayText(day.day, tx),
    buildPetPanelStackParts(day, tx),
    day.total_seconds,
    petPanelTotalText(day.total_seconds, tx),
    petPanelStackPercent,
    formatSeconds,
  );
}

function updateMode(next: PetPanelMode) {
  mode = next;
  applyPetPanelMode(panelElements, next, petPanelModeTitle(next, tx));
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
