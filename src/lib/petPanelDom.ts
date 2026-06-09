import type { LearnHeatmapCell } from "../api";
import type { PetPanelHeatmapDayCell } from "./petPanelMetrics";
import type { PetPanelStackPart } from "./petPanelStack";

type TranslateFn = (zh: string, en: string) => string;
type HeatCellClassFn = (level: LearnHeatmapCell["level"]) => string;
type FormatSecondsFn = (seconds: number) => string;
type StackPercentFn = (seconds: number, totalSeconds: number) => number;

export type PetPanelMode = "heatmap" | "stack";

export type PetPanelElements = {
  panelTitle: HTMLElement;
  panelHeatmap: HTMLElement;
  panelStack: HTMLElement;
  heatMonthLabel: HTMLElement;
  miniWeekHeader: HTMLElement;
  miniHeatmapGrid: HTMLElement;
  heatPrevMonthBtn: HTMLButtonElement;
  heatNextMonthBtn: HTMLButtonElement;
};

export function renderPetPanelShell(root: HTMLElement, tx: TranslateFn) {
  root.innerHTML = `
  <section class="panel-shell">
    <div id="panelTitle" class="panel-title">${tx("图表面板", "Panel")}</div>
    <div id="panelHeatmap" class="mini-heatmap">
      <div class="mini-head-row">
        <button id="heatPrevMonth" class="mini-month-btn" type="button">&lt;</button>
        <strong id="heatMonthLabel" class="mini-month-label">-</strong>
        <button id="heatNextMonth" class="mini-month-btn" type="button">&gt;</button>
      </div>
      <div id="miniWeekHeader" class="mini-week-header"></div>
      <div id="miniHeatmapGrid" class="mini-heatmap-grid"></div>
    </div>
    <div id="panelStack" class="mini-stack" style="display:none"></div>
  </section>
`;
}

export function queryPetPanelElements(root: ParentNode = document): PetPanelElements {
  const panelTitle = root.querySelector<HTMLElement>("#panelTitle");
  const panelHeatmap = root.querySelector<HTMLElement>("#panelHeatmap");
  const panelStack = root.querySelector<HTMLElement>("#panelStack");
  const heatMonthLabel = root.querySelector<HTMLElement>("#heatMonthLabel");
  const miniWeekHeader = root.querySelector<HTMLElement>("#miniWeekHeader");
  const miniHeatmapGrid = root.querySelector<HTMLElement>("#miniHeatmapGrid");
  const heatPrevMonthBtn = root.querySelector<HTMLButtonElement>("#heatPrevMonth");
  const heatNextMonthBtn = root.querySelector<HTMLButtonElement>("#heatNextMonth");

  if (
    !panelTitle
    || !panelHeatmap
    || !panelStack
    || !heatMonthLabel
    || !miniWeekHeader
    || !miniHeatmapGrid
    || !heatPrevMonthBtn
    || !heatNextMonthBtn
  ) {
    throw new Error("pet panel controls not found");
  }

  return {
    panelTitle,
    panelHeatmap,
    panelStack,
    heatMonthLabel,
    miniWeekHeader,
    miniHeatmapGrid,
    heatPrevMonthBtn,
    heatNextMonthBtn,
  };
}

export function applyPetPanelMode(
  elements: Pick<
    PetPanelElements,
    "panelTitle" | "panelHeatmap" | "panelStack" | "heatPrevMonthBtn" | "heatNextMonthBtn"
  >,
  mode: PetPanelMode,
  title: string,
) {
  elements.panelTitle.textContent = title;
  elements.panelHeatmap.style.display = mode === "heatmap" ? "grid" : "none";
  elements.panelStack.style.display = mode === "stack" ? "grid" : "none";
  elements.heatPrevMonthBtn.style.display = mode === "heatmap" ? "inline-grid" : "none";
  elements.heatNextMonthBtn.style.display = mode === "heatmap" ? "inline-grid" : "none";
}

export function renderPetPanelWeekHeaders(container: HTMLElement, labels: string[]) {
  container.replaceChildren();
  for (const label of labels) {
    const node = document.createElement("span");
    node.textContent = label;
    container.appendChild(node);
  }
}

export function renderPetPanelHeatmapGrid(
  container: HTMLElement,
  padCount: number,
  dayCells: PetPanelHeatmapDayCell[],
  heatCellClass: HeatCellClassFn,
  formatSeconds: FormatSecondsFn,
) {
  container.replaceChildren();
  for (let i = 0; i < padCount; i += 1) {
    const pad = document.createElement("div");
    pad.className = "mini-heat-cell pad";
    container.appendChild(pad);
  }

  for (const { dayKey, cell, isToday } of dayCells) {
    const node = document.createElement("div");
    node.className = `mini-heat-cell ${heatCellClass(cell.level)}`;
    if (isToday) {
      node.classList.add("today");
    }
    node.title = `${dayKey} ${formatSeconds(cell.learn_seconds)}`;
    container.appendChild(node);
  }
}

export function renderPetPanelEmptyStack(container: HTMLElement, text: string) {
  container.replaceChildren();
  const empty = document.createElement("div");
  empty.className = "mini-empty";
  empty.textContent = text;
  container.appendChild(empty);
}

export function renderPetPanelStack(
  container: HTMLElement,
  dayTextValue: string,
  parts: PetPanelStackPart[],
  totalSeconds: number,
  totalText: string,
  stackPercent: StackPercentFn,
  formatSeconds: FormatSecondsFn,
) {
  container.replaceChildren();

  const dayText = document.createElement("div");
  dayText.className = "stack-day";
  dayText.textContent = dayTextValue;

  const bar = document.createElement("div");
  bar.className = "mini-stack-bar";
  for (const item of parts) {
    const pct = stackPercent(item.seconds, totalSeconds);
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
  meta.textContent = totalText;

  container.append(dayText, bar, meta);
}
