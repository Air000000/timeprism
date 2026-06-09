import { invokeCommand } from "../client";

export async function startSession(categoryId: number): Promise<number> {
  return invokeCommand("start_session", { categoryId });
}

export async function stopActiveSession(): Promise<boolean> {
  return invokeCommand("stop_active_session");
}
