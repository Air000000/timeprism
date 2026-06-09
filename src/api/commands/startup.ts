import { invokeCommand } from "../client";

export async function getAutoStartEnabled(): Promise<boolean> {
  return invokeCommand("get_auto_start_enabled");
}

export async function setAutoStartEnabled(enabled: boolean): Promise<boolean> {
  return invokeCommand("set_auto_start_enabled", { enabled });
}
