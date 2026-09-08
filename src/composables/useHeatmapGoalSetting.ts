import { ref } from "vue";
import {
  getHeatmapGoalSecondsSetting,
  setHeatmapGoalSecondsSetting,
} from "../api";
import {
  heatmapGoalMinutesToSeconds,
  heatmapGoalSecondsToSliderMinutes,
  normalizeHeatmapGoalMinutes,
} from "../lib/heatmapGoalSetting";

type HeatmapGoalSettingOptions = {
  setErrorMessage: (error: unknown) => void;
};

export function useHeatmapGoalSetting({
  setErrorMessage,
}: HeatmapGoalSettingOptions) {
  const learnGoalSliderMinutes = ref(120);
  let heatmapGoalSaveTimer: number | null = null;

  function normalizeGoalFromSlider() {
    const total = normalizeHeatmapGoalMinutes(learnGoalSliderMinutes.value);
    learnGoalSliderMinutes.value = total;
    return total;
  }

  function syncGoalFromSlider() {
    normalizeGoalFromSlider();
    schedulePersistHeatmapGoal();
  }

  function getHeatmapGoalSeconds(): number {
    syncGoalFromSlider();
    return heatmapGoalMinutesToSeconds(learnGoalSliderMinutes.value);
  }

  function schedulePersistHeatmapGoal() {
    if (heatmapGoalSaveTimer !== null) {
      window.clearTimeout(heatmapGoalSaveTimer);
      heatmapGoalSaveTimer = null;
    }

    heatmapGoalSaveTimer = window.setTimeout(async () => {
      try {
        await setHeatmapGoalSecondsSetting(
          heatmapGoalMinutesToSeconds(normalizeGoalFromSlider()),
        );
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
      learnGoalSliderMinutes.value = heatmapGoalSecondsToSliderMinutes(savedGoalSeconds);
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
