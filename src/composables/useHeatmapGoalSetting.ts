import { ref } from "vue";
import {
  getHeatmapGoalSecondsSetting,
  setHeatmapGoalSecondsSetting,
} from "../api";

type HeatmapGoalSettingOptions = {
  setErrorMessage: (error: unknown) => void;
};

export function useHeatmapGoalSetting({
  setErrorMessage,
}: HeatmapGoalSettingOptions) {
  const learnGoalSliderMinutes = ref(120);
  let heatmapGoalSaveTimer: number | null = null;

  function normalizeGoalMinutes(minutes: number) {
    const rounded = Number.isFinite(minutes)
      ? Math.round(minutes / 15) * 15
      : 120;
    return Math.min(1440, Math.max(0, rounded));
  }

  function normalizeGoalFromSlider() {
    const total = normalizeGoalMinutes(learnGoalSliderMinutes.value);
    learnGoalSliderMinutes.value = total;
    return total;
  }

  function syncGoalFromSlider() {
    normalizeGoalFromSlider();
    schedulePersistHeatmapGoal();
  }

  function getHeatmapGoalSeconds(): number {
    syncGoalFromSlider();
    return learnGoalSliderMinutes.value * 60;
  }

  function schedulePersistHeatmapGoal() {
    if (heatmapGoalSaveTimer !== null) {
      window.clearTimeout(heatmapGoalSaveTimer);
      heatmapGoalSaveTimer = null;
    }

    heatmapGoalSaveTimer = window.setTimeout(async () => {
      try {
        await setHeatmapGoalSecondsSetting(normalizeGoalFromSlider() * 60);
      } catch (e) {
        setErrorMessage(e);
      } finally {
        heatmapGoalSaveTimer = null;
      }
    }, 260);
  }

  async function loadHeatmapGoalSecondsSetting() {
    try {
      const savedGoalSeconds = await getHeatmapGoalSecondsSetting();
      const minutes = Math.round(savedGoalSeconds / 60 / 15) * 15;
      learnGoalSliderMinutes.value = normalizeGoalMinutes(minutes);
    } catch (e) {
      setErrorMessage(e);
    }
  }

  function cleanupHeatmapGoalSetting() {
    if (heatmapGoalSaveTimer !== null) {
      window.clearTimeout(heatmapGoalSaveTimer);
      heatmapGoalSaveTimer = null;
    }
  }

  return {
    learnGoalSliderMinutes,
    getHeatmapGoalSeconds,
    loadHeatmapGoalSecondsSetting,
    cleanupHeatmapGoalSetting,
  };
}
