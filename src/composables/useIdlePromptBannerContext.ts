import { computed, type Ref } from "vue";
import type { IdlePrompt } from "../api";
import type {
  IdlePromptBannerContext,
  TranslateFn,
} from "../components/viewContexts";

type IdlePromptBannerContextOptions = {
  tx: TranslateFn;
  currentIdlePrompt: Readonly<Ref<IdlePrompt | null>>;
  formatIdlePromptSpan: IdlePromptBannerContext["formatIdlePromptSpan"];
  idleRememberChoice: Readonly<Ref<boolean>>;
  onIdleRememberChoiceChange: IdlePromptBannerContext["onIdleRememberChoiceChange"];
  idleActionLoading: Readonly<Ref<boolean>>;
  handleResolveIdle: IdlePromptBannerContext["handleResolveIdle"];
};

export function useIdlePromptBannerContext({
  tx,
  currentIdlePrompt,
  formatIdlePromptSpan,
  idleRememberChoice,
  onIdleRememberChoiceChange,
  idleActionLoading,
  handleResolveIdle,
}: IdlePromptBannerContextOptions) {
  return computed<IdlePromptBannerContext | null>(() => {
    if (!currentIdlePrompt.value) {
      return null;
    }

    return {
      tx,
      currentIdlePrompt: currentIdlePrompt.value,
      formatIdlePromptSpan,
      idleRememberChoice: idleRememberChoice.value,
      onIdleRememberChoiceChange,
      idleActionLoading: idleActionLoading.value,
      handleResolveIdle,
    };
  });
}
