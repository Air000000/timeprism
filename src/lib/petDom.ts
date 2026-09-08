type TranslateFn = (zh: string, en: string) => string;

export type PetElements = {
  mood: HTMLDivElement;
  learn: HTMLDivElement;
  rest: HTMLDivElement;
  learnToken: HTMLSpanElement;
  restToken: HTMLSpanElement;
  characterImage: HTMLImageElement;
  dragArea: HTMLElement;
  shell: HTMLElement;
  promptBubble: HTMLElement;
};

export function renderPetShell(app: HTMLElement, tx: TranslateFn, primaryCharacterSrc: string) {
  app.innerHTML = `
	<section class="pet-shell">
		<div id="petMood" class="pet-hidden-mood">${tx("自动记录中", "Auto tracking")}</div>

		<div class="pet-body">
			<div class="pet-main">
				<div id="petDragArea" class="pet-portrait-wrap">
					<img id="petCharacterImage" class="pet-portrait" src="${primaryCharacterSrc}" alt="${tx("TimePrism 桌宠角色", "TimePrism Pet Character")}" />
				</div>
				<div class="today-lines">
					<div class="line-row single-row">
						<span id="learnToken" class="line-label">${tx("学", "L")}</span>
						<strong id="learnValue" class="line-value">00:00:00</strong>
						<span id="restToken" class="line-label">${tx("休", "B")}</span>
						<strong id="restValue" class="line-value">00:00:00</strong>
					</div>
				</div>
			</div>

		</div>
	</section>
	<section id="petPromptBubble" class="pet-prompt-bubble"></section>
`;
}

export function queryPetElements(root: ParentNode = document): PetElements {
  const mood = root.querySelector<HTMLDivElement>("#petMood");
  const learn = root.querySelector<HTMLDivElement>("#learnValue");
  const rest = root.querySelector<HTMLDivElement>("#restValue");
  const learnToken = root.querySelector<HTMLSpanElement>("#learnToken");
  const restToken = root.querySelector<HTMLSpanElement>("#restToken");
  const characterImage = root.querySelector<HTMLImageElement>("#petCharacterImage");
  const dragArea = root.querySelector<HTMLElement>("#petDragArea");
  const shell = root.querySelector<HTMLElement>(".pet-shell");
  const promptBubble = root.querySelector<HTMLElement>("#petPromptBubble");

  if (
    !mood
    || !learn
    || !rest
    || !learnToken
    || !restToken
    || !characterImage
    || !dragArea
    || !shell
    || !promptBubble
  ) {
    throw new Error("pet controls not found");
  }

  return {
    mood,
    learn,
    rest,
    learnToken,
    restToken,
    characterImage,
    dragArea,
    shell,
    promptBubble,
  };
}

type PetLocalizedStaticText = {
  learnToken: string;
  restToken: string;
  autoTracking: string;
  autoTrackingValues: readonly string[];
};

export function applyPetLocalizedStaticText(
  elements: Pick<PetElements, "mood" | "learnToken" | "restToken">,
  text: PetLocalizedStaticText,
) {
  elements.learnToken.textContent = text.learnToken;
  elements.restToken.textContent = text.restToken;

  const currentMood = elements.mood.textContent || "";
  if (text.autoTrackingValues.includes(currentMood)) {
    elements.mood.textContent = text.autoTracking;
  }
}

export function renderPetSummary(
  elements: Pick<PetElements, "learn" | "rest" | "mood">,
  learnText: string,
  restText: string,
  moodText: string,
) {
  elements.learn.textContent = learnText;
  elements.rest.textContent = restText;
  elements.mood.textContent = moodText;
}
