type TranslateFn = (zh: string, en: string) => string;

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

export function renderPetPanelWeekHeaders(container: HTMLElement, labels: string[]) {
  container.replaceChildren();
  for (const label of labels) {
    const node = document.createElement("span");
    node.textContent = label;
    container.appendChild(node);
  }
}
