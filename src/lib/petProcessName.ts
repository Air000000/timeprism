type TranslateFn = (zh: string, en: string) => string;

export function cleanPetProcessName(name: string, tx: TranslateFn): string {
  return name
    .replace(/^__idle_learn__\.exe$/i, tx("离开时段（学习）", "Away Segment (Learn)"))
    .replace(/^__idle_rest__\.exe$/i, tx("离开时段（休息）", "Away Segment (Break)"))
    .replace(/^__idle__\.exe$/i, tx("离开时段", "Away Segment"))
    .replace(/\.exe$/i, "")
    .trim();
}
