import { computed, ref } from "vue";
import type { HomeRhythmBar, HomeViewContext } from "../components/viewContexts";

type RhythmSegment = "learn" | "rest";

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

function rhythmToneText(ctx: HomeViewContext, share: number, segment: RhythmSegment): string {
  if (segment === "learn") {
    if (share >= 60) {
      return ctx.tx("这一天明显进入了学习主线。", "Learning clearly led this day.");
    }
    if (share >= 35) {
      return ctx.tx("这一天有一段扎实的学习投入。", "There was a solid learning block here.");
    }
    return ctx.tx("这一天学习有推进，但还留着余量。", "Learning moved forward, but there was still room.");
  }

  if (share >= 60) {
    return ctx.tx("这一天更偏向恢复和放松。", "This day leaned more toward recovery and rest.");
  }
  if (share >= 35) {
    return ctx.tx("这一天有一段比较明确的休息时间。", "There was a clearly defined rest window here.");
  }
  return ctx.tx("这段休息比较轻，像一次短缓冲。", "This rest block was light, more like a short reset.");
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
    const palette = segment === "learn"
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
    rhythmTooltip.value = {
      visible: true,
      x: pos.x,
      y: pos.y,
      ...palette,
      dayLabel: bar.label,
      segmentLabel: segment === "learn" ? ctx.tx("学习", "Learn") : ctx.tx("休息", "Rest"),
      durationText: compactRhythmDuration(segmentSeconds),
      shareText: `${share}%`,
      toneText: rhythmToneText(ctx, share, segment),
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
