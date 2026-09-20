import { resolveIdlePrompt } from "./commands/foreground";
import type { IdlePrompt } from "./types";

const prompt: IdlePrompt = {
  id: 1,
  start_timestamp: 100,
  end_timestamp: 400,
  duration_ms: 300_000,
  attribution_process_name: "code.exe",
};

export const idleAttributionProcessContract: string | null = prompt.attribution_process_name;

void resolveIdlePrompt({
  prompt_id: prompt.id,
  decision: "APP",
});
