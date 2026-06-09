type TranslateFn = (zh: string, en: string) => string;

export type RhythmSegment = "learn" | "rest";

export type RhythmTooltipPalette = {
  bgColor: string;
  borderColor: string;
  accentColor: string;
};

export function compactRhythmDuration(seconds: number): string {
  const safe = Math.max(0, Math.floor(seconds));
  if (safe >= 3600) {
    const hours = safe / 3600;
    return `${hours >= 10 ? hours.toFixed(0) : hours.toFixed(1)}h`;
  }
  if (safe >= 60) {
    return `${Math.round(safe / 60)}m`;
  }
  return `${safe}s`;
}

export function rhythmSegmentLabel(segment: RhythmSegment, tx: TranslateFn): string {
  return segment === "learn" ? tx("学习", "Learn") : tx("休息", "Rest");
}

export function rhythmSegmentPalette(segment: RhythmSegment): RhythmTooltipPalette {
  return segment === "learn"
    ? {
      bgColor: "rgba(233, 246, 236, 0.96)",
      borderColor: "rgba(123, 181, 137, 0.42)",
      accentColor: "#2f7a52",
    }
    : {
      bgColor: "rgba(236, 242, 252, 0.96)",
      borderColor: "rgba(150, 176, 223, 0.42)",
      accentColor: "#476d9f",
    };
}

export function rhythmToneText(
  tx: TranslateFn,
  share: number,
  segment: RhythmSegment,
): string {
  if (segment === "learn") {
    if (share >= 60) {
      return tx("这一天明显进入了学习主线。", "Learning clearly led this day.");
    }
    if (share >= 35) {
      return tx("这一天有一段扎实的学习投入。", "There was a solid learning block here.");
    }
    return tx("这一天学习有推进，但还留着余量。", "Learning moved forward, but there was still room.");
  }

  if (share >= 60) {
    return tx("这一天更偏向恢复和放松。", "This day leaned more toward recovery and rest.");
  }
  if (share >= 35) {
    return tx("这一天有一段比较明确的休息时间。", "There was a clearly defined rest window here.");
  }
  return tx("这段休息比较轻，像一次短缓冲。", "This rest block was light, more like a short reset.");
}
