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

  function syncGoalFromSlider() {
    let total = Number.isFinite(learnGoalSliderMinutes.value)
      ? Math.round(learnGoalSliderMinutes.value / 15) * 15
      : 120;
    total = Math.min(1440, Math.max(0, total));
    learnGoalSliderMinutes.value = total;
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
        await setHeatmapGoalSecondsSetting(getHeatmapGoalSeconds());
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
      learnGoalSliderMinutes.value = Math.min(1440, Math.max(0, minutes));
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
