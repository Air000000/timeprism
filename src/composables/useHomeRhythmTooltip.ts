import { computed, ref } from "vue";
import type { HomeRhythmBar, HomeViewContext } from "../components/viewContexts";
import {
  compactRhythmDuration,
  rhythmSegmentLabel,
  rhythmSegmentPalette,
  rhythmToneText,
  type RhythmSegment,
} from "../lib/homeRhythmTooltip";

export { compactRhythmDuration } from "../lib/homeRhythmTooltip";

type RhythmTooltipState = {
  visible: boolean;
  x: number;
  y: number;
  bgColor: string;
  borderColor: string;
  accentColor: string;
  dayLabel: string;
  segmentLabel: string;
  durationText: string;
  shareText: string;
  toneText: string;
  contextText: string;
};

function placeRhythmTooltip(event: MouseEvent) {
  const width = 248;
  const height = 126;
  const gap = 16;
  const maxX = Math.max(12, window.innerWidth - width - 12);
  const maxY = Math.max(12, window.innerHeight - height - 12);
  return {
    x: Math.min(maxX, event.clientX + gap),
    y: Math.min(maxY, event.clientY - 12),
  };
}

export function useHomeRhythmTooltip(getCtx: () => HomeViewContext) {
  const rhythmTooltip = ref<RhythmTooltipState | null>(null);

  const homeRhythmSummary = computed(() => {
    const ctx = getCtx();
    const bars = ctx.homeMonthRhythmBars ?? [];
    const totalSeconds = bars.reduce((sum, bar) => sum + (bar.totalSeconds ?? 0), 0);
    const activeDays = bars.filter((bar) => (bar.totalSeconds ?? 0) > 0).length;
    const averageSeconds = bars.length > 0 ? Math.round(totalSeconds / bars.length) : 0;
    const bestBar = bars.reduce<HomeRhythmBar | null>((best, bar) => (
      (bar.totalSeconds ?? 0) > (best?.totalSeconds ?? -1) ? bar : best
    ), null);
    return {
      totalText: compactRhythmDuration(totalSeconds),
      averageText: compactRhythmDuration(averageSeconds),
      activeDays,
      bestText: bestBar && (bestBar.totalSeconds ?? 0) > 0
        ? ctx.tx(
          `${bestBar.label}号 · ${compactRhythmDuration(bestBar.totalSeconds)}`,
          `${bestBar.label} · ${compactRhythmDuration(bestBar.totalSeconds)}`,
        )
        : ctx.tx("暂无记录", "No data"),
    };
  });

  function handleRhythmEnter(
    event: MouseEvent,
    bar: HomeRhythmBar,
    segment: RhythmSegment,
  ) {
    const ctx = getCtx();
    const segmentSeconds = segment === "learn" ? (bar.learnSeconds ?? 0) : (bar.restSeconds ?? 0);
    const share = (bar.computerTotalSeconds ?? 0) > 0
      ? Math.round((segmentSeconds / bar.computerTotalSeconds) * 100)
      : 0;
    const pos = placeRhythmTooltip(event);
    const palette = rhythmSegmentPalette(segment);
    rhythmTooltip.value = {
      visible: true,
      x: pos.x,
      y: pos.y,
      ...palette,
      dayLabel: bar.label,
      segmentLabel: rhythmSegmentLabel(segment, ctx.tx),
      durationText: compactRhythmDuration(segmentSeconds),
      shareText: `${share}%`,
      toneText: rhythmToneText(ctx.tx, share, segment),
      contextText: ctx.tx(
        `第 ${bar.label} 天电脑总使用 ${compactRhythmDuration(bar.computerTotalSeconds ?? 0)}`,
        `Day ${bar.label} total computer use ${compactRhythmDuration(bar.computerTotalSeconds ?? 0)}`,
      ),
    };
  }

  function handleRhythmMove(event: MouseEvent) {
    if (!rhythmTooltip.value) {
      return;
    }
    const pos = placeRhythmTooltip(event);
    rhythmTooltip.value.x = pos.x;
    rhythmTooltip.value.y = pos.y;
  }

  function handleRhythmLeave() {
    rhythmTooltip.value = null;
  }

  return {
    compactRhythmDuration,
    handleRhythmEnter,
    handleRhythmLeave,
    handleRhythmMove,
    homeRhythmSummary,
    rhythmTooltip,
  };
}
