import type { ForegroundCaptureDiagnostic } from "../api";

type TranslateFn = (zh: string, en: string) => string;
type RuleMappedType = "LEARN" | "REST" | "IGNORE";

export function guardCaptureBlockReasonText(reason: string | null, tx: TranslateFn): string {
  if (!reason) {
    return "-";
  }
  if (reason === "baseline_only") {
    return tx("首次采样仅建立基线", "First sample only sets baseline");
  }
  if (reason === "curtain_enabled") {
    return tx("被窗帘模式拦截", "Blocked by Curtain mode");
  }
  if (reason === "incognito_window") {
    return tx("无痕/隐私窗口拦截", "Blocked by incognito/private window");
  }
  if (reason === "whitelist_blocked") {
    return tx("白名单策略降级", "Whitelisted-only policy fallback");
  }
  if (reason === "no_foreground_window") {
    return tx("未获取到前台窗口", "No foreground window detected");
  }
  if (reason === "elapsed_too_short") {
    return tx("采样间隔过短", "Sampling interval too short");
  }
  return reason;
}

export function guardCaptureRuleText(
  item: ForegroundCaptureDiagnostic,
  tx: TranslateFn,
  mappedTypeText: (mappedType: RuleMappedType) => string,
): string {
  if (!item.rule_saved) {
    return tx("未分类（规则未保存）", "Unclassified (rule not saved)");
  }
  return mappedTypeText(item.rule_mapped_type);
}

export function canSaveRuleFromDiagnostic(item: ForegroundCaptureDiagnostic): boolean {
  return item.observed_process_name !== "unknown.exe";
}
