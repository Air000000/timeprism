type TranslateFn = (zh: string, en: string) => string;

export function defaultGuardAutoCaptureFeedback(tx: TranslateFn): string {
  return tx("自动采样已开启", "Auto capture enabled");
}

export function defaultGuardCheckFeedback(tx: TranslateFn): string {
  return tx("尚未执行检测", "No check executed yet");
}

export function guardIdleResolveErrorFeedback(error: unknown, tx: TranslateFn): string {
  return tx(`空闲时段分类失败：${error}`, `Failed to classify idle segment: ${error}`);
}

export function guardRuleSavedFeedback(
  processName: string,
  mappedTypeLabel: string,
  tx: TranslateFn,
): string {
  return tx(
    `已保存规则：${processName} -> ${mappedTypeLabel}`,
    `Rule saved: ${processName} -> ${mappedTypeLabel}`,
  );
}

export function guardRuleUpdatedFeedback(
  processName: string,
  mappedTypeLabel: string,
  tx: TranslateFn,
): string {
  return tx(
    `已更新规则：${processName} -> ${mappedTypeLabel}`,
    `Rule updated: ${processName} -> ${mappedTypeLabel}`,
  );
}

export function guardRuleSaveErrorFeedback(error: unknown, tx: TranslateFn): string {
  return tx(`保存规则失败：${error}`, `Failed to save rule: ${error}`);
}