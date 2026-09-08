import type { PromptAction } from "./petPrompts";

export type PromptBubbleState = {
  currentPromptKey: string;
};

type RenderPromptBubbleOptions = {
  promptBubble: HTMLElement;
  state: PromptBubbleState;
  promptKey: string;
  title: string;
  detail: string;
  actions: PromptAction[];
  onActionError: (error: unknown) => void;
};

export function hidePromptBubble(promptBubble: HTMLElement, state: PromptBubbleState) {
  state.currentPromptKey = "";
  promptBubble.classList.remove("visible", "down");
  promptBubble.replaceChildren();
}

export function renderPromptBubble({
  promptBubble,
  state,
  promptKey,
  title,
  detail,
  actions,
  onActionError,
}: RenderPromptBubbleOptions) {
  if (state.currentPromptKey === promptKey && promptBubble.classList.contains("visible")) {
    return;
  }

  state.currentPromptKey = promptKey;
  promptBubble.replaceChildren();

  const titleEl = document.createElement("div");
  titleEl.className = "prompt-title";
  titleEl.textContent = title;

  const detailEl = document.createElement("div");
  detailEl.className = "prompt-detail";
  detailEl.textContent = detail;

  const actionWrap = document.createElement("div");
  actionWrap.className = "prompt-actions";

  const buttons: HTMLButtonElement[] = [];
  for (const action of actions) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "prompt-action";
    button.textContent = action.label;
    button.addEventListener("click", async () => {
      if (button.disabled) {
        return;
      }
      for (const btn of buttons) {
        btn.disabled = true;
      }
      try {
        await action.run();
      } catch (e) {
        onActionError(e);
      } finally {
        for (const btn of buttons) {
          btn.disabled = false;
        }
      }
    });
    buttons.push(button);
    actionWrap.appendChild(button);
  }

  promptBubble.append(titleEl, detailEl, actionWrap);
  promptBubble.classList.remove("down");
  promptBubble.classList.add("visible");
}
