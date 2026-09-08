export type DockEdge = "left" | "right";
export type PetDockState = "free" | "dragging" | "docked_left" | "docked_right";

export type PetWindowSettleResult = {
  x: number;
  y: number;
  width: number;
  height: number;
  state: "free" | "dock_left" | "dock_right";
};

export function isDockedState(state: PetDockState): boolean {
  return state === "docked_left" || state === "docked_right";
}

export function dockEdgeForState(state: PetDockState): DockEdge | null {
  if (state === "docked_left") {
    return "left";
  }
  if (state === "docked_right") {
    return "right";
  }
  return null;
}

export function petDockStateFromSettleState(state: PetWindowSettleResult["state"]): PetDockState {
  if (state === "dock_left") {
    return "docked_left";
  }
  if (state === "dock_right") {
    return "docked_right";
  }
  return "free";
}
