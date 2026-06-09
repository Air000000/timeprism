import { nextTick, type Ref } from "vue";
import type { HistorySubViewKey, MainViewKey } from "./useAppNavigation";

type InsightsSectionNavigationOptions = {
  currentMainView: Ref<MainViewKey>;
  selectHistoryView: (next: HistorySubViewKey) => void;
};

export function useInsightsSectionNavigation({
  currentMainView,
  selectHistoryView,
}: InsightsSectionNavigationOptions) {
  let sectionFlashTimer: number | null = null;

  function flashInsightsSection(target: HTMLElement | null) {
    if (!target) {
      return;
    }
    document.getElementById("insights-stack-anchor")?.classList.remove("section-flash");
    target.classList.add("section-flash");

    if (sectionFlashTimer !== null) {
      window.clearTimeout(sectionFlashTimer);
      sectionFlashTimer = null;
    }

    sectionFlashTimer = window.setTimeout(() => {
      target.classList.remove("section-flash");
      sectionFlashTimer = null;
    }, 1000);
  }

  async function scrollToInsightsSection(section: string) {
    currentMainView.value = "insights";
    void section;
    selectHistoryView("topApps");
    await nextTick();
    const target = document.getElementById("insights-stack-anchor");
    flashInsightsSection(target);
  }

  function cleanupInsightsSectionNavigation() {
    if (sectionFlashTimer !== null) {
      window.clearTimeout(sectionFlashTimer);
      sectionFlashTimer = null;
    }
  }

  return {
    scrollToInsightsSection,
    cleanupInsightsSectionNavigation,
  };
}
