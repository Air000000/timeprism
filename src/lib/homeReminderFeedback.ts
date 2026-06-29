type TranslateFn = (zh: string, en: string) => string;

export function reminderContentEmptyFeedback(tx: TranslateFn): string {
  return tx("提醒内容不能为空", "Reminder content cannot be empty");
}

export function reminderSavedFeedback(created: boolean, tx: TranslateFn): string {
  return created
    ? tx("提醒已创建", "Reminder created")
    : tx("提醒已更新", "Reminder updated");
}

export function reminderDeletedFeedback(tx: TranslateFn): string {
  return tx("提醒已删除", "Reminder deleted");
}

export function reminderDoneFeedback(done: boolean, tx: TranslateFn): string {
  return done
    ? tx("提醒已完成", "Reminder marked done")
    : tx("提醒已恢复", "Reminder restored");
}

export function reminderSnoozedFeedback(tx: TranslateFn): string {
  return tx("提醒已稍后 10 分钟", "Reminder snoozed 10m");
}