import { invokeCommand } from "../client";

export async function appendAppUsageLog(input: {
  process_name: string;
  window_title: string;
  start_timestamp: number;
  duration_ms: number;
}): Promise<boolean> {
  return invokeCommand("append_app_usage_log", {
    processName: input.process_name,
    windowTitle: input.window_title,
    startTimestamp: input.start_timestamp,
    durationMs: input.duration_ms,
  });
}
