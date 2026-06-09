import { invokeCommand } from "../client";
import type { ForegroundCaptureDiagnostic, IdleMemoryState, IdlePrompt } from "../types";

export async function captureForegroundOnce(durationMs = 5000): Promise<boolean> {
  return invokeCommand("capture_foreground_once", { durationMs });
}

export async function listForegroundCaptureDiagnostics(
  limit = 10,
  uniqueByProcess = true,
): Promise<ForegroundCaptureDiagnostic[]> {
  return invokeCommand("list_foreground_capture_diagnostics", { limit, uniqueByProcess });
}

export async function listPendingIdlePrompts(limit = 5): Promise<IdlePrompt[]> {
  return invokeCommand("list_pending_idle_prompts", { limit });
}

export async function resolveIdlePrompt(input: {
  prompt_id: number;
  decision: "LEARN" | "REST" | "IDLE" | "SKIP";
  remember_this_session?: boolean;
}): Promise<boolean> {
  return invokeCommand("resolve_idle_prompt", { input });
}

export async function getIdleMemoryState(): Promise<IdleMemoryState> {
  return invokeCommand("get_idle_memory_state");
}

export async function clearIdleMemoryState(): Promise<void> {
  return invokeCommand("clear_idle_memory_state");
}
