import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return tauriInvoke<T>(command, args);
}
