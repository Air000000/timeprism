import type { PrivacySettings } from "../api";

type TranslateFn = (zh: string, en: string) => string;

export function browserModeText(mode: PrivacySettings["browser_title_mode"], tx: TranslateFn): string {
  if (mode === "FULL") {
    return tx("完整标题", "Full title");
  }
  if (mode === "BLUR") {
    return tx("模糊标题", "Blurred title");
  }
  return tx("不采集标题", "No title capture");
}

export function defaultPrivacyFeedback(tx: TranslateFn): string {
  return tx("未保存隐私设置", "Privacy settings not saved");
}

export function privacySettingsSavedFeedback(
  mode: PrivacySettings["browser_title_mode"],
  tx: TranslateFn,
): string {
  return tx(
    `隐私设置已保存（浏览器模式：${browserModeText(mode, tx)}）。`,
    `Privacy settings saved (browser mode: ${browserModeText(mode, tx)}).`,
  );
}

export function privacySaveErrorFeedback(error: unknown, tx: TranslateFn): string {
  return tx(`保存失败：${error}`, `Save failed: ${error}`);
}

export function whitelistAddedFeedback(processName: string, tx: TranslateFn): string {
  return tx(`已加入白名单：${processName}`, `Added to whitelist: ${processName}`);
}

export function whitelistRemovedFeedback(processName: string, tx: TranslateFn): string {
  return tx(`已移除白名单：${processName}`, `Removed from whitelist: ${processName}`);
}

export function whitelistAddErrorFeedback(error: unknown, tx: TranslateFn): string {
  return tx(`添加失败：${error}`, `Add failed: ${error}`);
}

export function whitelistRemoveErrorFeedback(error: unknown, tx: TranslateFn): string {
  return tx(`移除失败：${error}`, `Remove failed: ${error}`);
}