import type { MenuItemOptions } from "@tauri-apps/api/menu";

type TranslateFn = (zh: string, en: string) => string;

export type PetPanelWindowMode = "summary" | "heatmap" | "stack";

type PetContextMenuActions = {
  setPanelMode: (next: PetPanelWindowMode) => void;
  showMainWindow: () => void;
  hidePetWindow: () => void;
  closePetWindow: () => void;
};

export function buildPetContextMenuItems(
  tx: TranslateFn,
  currentPanelMode: PetPanelWindowMode,
  actions: PetContextMenuActions,
): MenuItemOptions[] {
  return [
    {
      id: "show-calendar",
      text: tx("学习日历", "Learning Calendar"),
      action: () => {
        actions.setPanelMode(currentPanelMode === "heatmap" ? "summary" : "heatmap");
      },
    },
    {
      id: "show-breakdown",
      text: tx("周活跃", "Weekly Activity"),
      action: () => {
        actions.setPanelMode(currentPanelMode === "stack" ? "summary" : "stack");
      },
    },
    {
      id: "show-main",
      text: tx("打开主界面", "Open Main"),
      action: actions.showMainWindow,
    },
    {
      id: "hide-pet",
      text: tx("隐藏桌宠", "Hide Pet"),
      action: actions.hidePetWindow,
    },
    {
      id: "close-pet",
      text: tx("关闭桌宠", "Close Pet"),
      action: actions.closePetWindow,
    },
  ];
}
