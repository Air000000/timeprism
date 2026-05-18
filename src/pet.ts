import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { Menu } from "@tauri-apps/api/menu";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { getTodaySummary, type TodaySummary } from "./api";
import "./pet.css";

const petWindow = getCurrentWindow();
const EDGE_SNAP_THRESHOLD = 72;
const LOCALE_STORAGE_KEY = "timeprism-locale";
const PET_CHARACTER_STORAGE_KEY = "timeprism.pet.character";
const PET_CHARACTER_PRIMARY_SRC = "/慕沛灵Q版桌宠形象.png";
const PET_CHARACTER_LEGACY_PRIMARY_SRC = "/pet-character.png";
const PET_CHARACTER_DEFAULT_SRC = "/pet-character-default.svg";
const PET_CHARACTER_DOCKED_LEFT_SRC = "/左侧.png";
const PET_CHARACTER_DOCKED_RIGHT_SRC = "/右侧.png";

type LocaleCode = "zh-CN" | "en-US";

function getLocale(): LocaleCode {
	try {
		const stored = window.localStorage.getItem(LOCALE_STORAGE_KEY);
		if (stored === "zh-CN" || stored === "en-US") {
			return stored;
		}
	} catch {
		// Ignore storage read failures.
	}
	const browserLang = typeof navigator !== "undefined" ? navigator.language.toLowerCase() : "zh-cn";
	return browserLang.startsWith("zh") ? "zh-CN" : "en-US";
}

function tx(zh: string, en: string): string {
	return getLocale() === "zh-CN" ? zh : en;
}

function cleanPetProcessName(name: string): string {
	return name
		.replace(/^__idle_learn__\.exe$/i, tx("离开时段（学习）", "Away Segment (Learn)"))
		.replace(/^__idle_rest__\.exe$/i, tx("离开时段（休息）", "Away Segment (Break)"))
		.replace(/^__idle__\.exe$/i, tx("离开时段", "Away Segment"))
		.replace(/\.exe$/i, "")
		.trim();
}

type DockEdge = "left" | "right";
type PetDockState = "free" | "dragging" | "docked_left" | "docked_right";

type PetWindowSettleResult = {
	x: number;
	y: number;
	width: number;
	height: number;
	state: "free" | "dock_left" | "dock_right";
};

let petState: PetDockState = "free";
let isDragging = false;
let expanded = false;
let hideTimer: number | null = null;
let moodResetTimer: number | null = null;
let dragPointerId: number | null = null;
let snapRetryTimer: number | null = null;
let currentPanelMode: "summary" | "heatmap" | "stack" = "summary";

function formatSeconds(totalSeconds: number): string {
	const safe = Math.max(0, Math.floor(totalSeconds));
	const h = Math.floor(safe / 3600)
		.toString()
		.padStart(2, "0");
	const m = Math.floor((safe % 3600) / 60)
		.toString()
		.padStart(2, "0");
	const s = Math.floor(safe % 60)
		.toString()
		.padStart(2, "0");
	return `${h}:${m}:${s}`;
}

const app = document.querySelector<HTMLDivElement>("#pet-app");
if (!app) {
	throw new Error("pet app root not found");
}

app.innerHTML = `
	<section class="pet-shell">
		<div id="petMood" class="pet-hidden-mood">${tx("自动记录中", "Auto tracking")}</div>

		<div class="pet-body">
			<div class="pet-main">
				<div id="petDragArea" class="pet-portrait-wrap">
					<img id="petCharacterImage" class="pet-portrait" src="${PET_CHARACTER_PRIMARY_SRC}" alt="${tx("TimePrism 桌宠角色", "TimePrism Pet Character")}" />
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

const moodEl = document.querySelector<HTMLDivElement>("#petMood");
const learnEl = document.querySelector<HTMLDivElement>("#learnValue");
const restEl = document.querySelector<HTMLDivElement>("#restValue");
const learnTokenEl = document.querySelector<HTMLSpanElement>("#learnToken");
const restTokenEl = document.querySelector<HTMLSpanElement>("#restToken");
const petCharacterImage = document.querySelector<HTMLImageElement>("#petCharacterImage");
const petDragAreaEl = document.querySelector<HTMLElement>("#petDragArea");
const petShell = document.querySelector<HTMLElement>(".pet-shell");
const promptBubbleEl = document.querySelector<HTMLElement>("#petPromptBubble");

if (
	!moodEl
	|| !learnEl
	|| !restEl
	|| !learnTokenEl
	|| !restTokenEl
	|| !petCharacterImage
	|| !petDragAreaEl
	|| !petShell
	|| !promptBubbleEl
) {
	throw new Error("pet controls not found");
}

const mood = moodEl;
const learn = learnEl;
const rest = restEl;
const learnToken = learnTokenEl;
const restToken = restTokenEl;
const characterImage = petCharacterImage;
const dragArea = petDragAreaEl;
const shell = petShell;
const promptBubble = promptBubbleEl;

function applyLocalizedStaticText() {
	learnToken.textContent = tx("学", "L");
	restToken.textContent = tx("休", "B");

	const currentMood = mood.textContent || "";
	if (
		currentMood === "自动记录中"
		|| currentMood === "Auto tracking"
	) {
		mood.textContent = tx("自动记录中", "Auto tracking");
	}
}

function getPetCharacterSrc(): string {
	try {
		const saved = window.localStorage.getItem(PET_CHARACTER_STORAGE_KEY)?.trim();
		if (saved) {
			if (saved === PET_CHARACTER_LEGACY_PRIMARY_SRC) {
				return PET_CHARACTER_PRIMARY_SRC;
			}
			return saved;
		}
	} catch {
		// Ignore storage access failures in restricted environments.
	}
	return PET_CHARACTER_PRIMARY_SRC;
}

function applyPetCharacter() {
	characterImage.src = getPetCharacterSrc();
	characterImage.onerror = () => {
		if (characterImage.src.endsWith(PET_CHARACTER_DEFAULT_SRC)) {
			return;
		}
		characterImage.src = PET_CHARACTER_DEFAULT_SRC;
	};
}

function isDockedState(state = petState): boolean {
	return state === "docked_left" || state === "docked_right";
}

function currentDockEdge(): DockEdge | null {
	if (petState === "docked_left") {
		return "left";
	}
	if (petState === "docked_right") {
		return "right";
	}
	return null;
}

function applyDockedAppearance() {
	const docked = isDockedState();
	const dockEdge = currentDockEdge();
	shell.classList.toggle("edge-hidden", docked);
	shell.classList.toggle("dock-left", petState === "docked_left");
	shell.classList.toggle("dock-right", petState === "docked_right");
	if (!docked || !dockEdge) {
		applyPetCharacter();
		return;
	}

	characterImage.src = dockEdge === "left" ? PET_CHARACTER_DOCKED_LEFT_SRC : PET_CHARACTER_DOCKED_RIGHT_SRC;
	characterImage.onerror = () => {
		characterImage.onerror = null;
		applyPetCharacter();
	};
}

function setPetState(next: PetDockState) {
	petState = next;
	applyDockedAppearance();
}

function setPetCharacterSrc(src: string) {
	const normalized = src.trim();
	try {
		if (!normalized) {
			window.localStorage.removeItem(PET_CHARACTER_STORAGE_KEY);
		} else {
			window.localStorage.setItem(PET_CHARACTER_STORAGE_KEY, normalized);
		}
	} catch {
		// Ignore storage failures and still try to apply to current session.
	}
	applyDockedAppearance();
}

(window as Window & { setTimePrismPetCharacter?: (src: string) => void }).setTimePrismPetCharacter = setPetCharacterSrc;

type IdlePromptLite = {
	id: number;
	start_timestamp: number;
	end_timestamp: number;
	duration_ms: number;
};

type PendingRuleProcessLite = {
	process_name: string;
	total_seconds: number;
};

type ReminderLite = {
	id: number;
	content: string;
	repeat_rule: "NONE" | "DAILY" | "WEEKLY";
	remind_at: number | null;
	daily_time_minutes: number | null;
	weekly_days: number[] | null;
	next_due_timestamp: number;
	done: boolean;
};

type PromptAction = {
	label: string;
	run: () => Promise<void>;
};

type PromptDescriptor = {
	key: string;
	title: string;
	detail: string;
	actions: PromptAction[];
};

const promptSnoozeUntilByKey = new Map<string, number>();
let currentPromptKey = "";

async function setPanelMode(next: "summary" | "heatmap" | "stack") {
	currentPanelMode = next;

	try {
		if (next === "summary") {
			await invoke("hide_pet_panel");
			return;
		}

		await invoke("show_pet_panel", { mode: next });
		await invoke("sync_pet_panel_position");
	} catch (e) {
		reportActionError("图表浮窗切换失败", e);
	}
}

function setMood(text: string) {
	mood.textContent = text;
}

function clearMoodResetTimer() {
	if (moodResetTimer !== null) {
		window.clearTimeout(moodResetTimer);
		moodResetTimer = null;
	}
}

function setTransientMood(text: string, timeoutMs = 2200) {
	setMood(text);
	clearMoodResetTimer();
	moodResetTimer = window.setTimeout(() => {
		setMood(tx("自动记录中", "Auto tracking"));
		moodResetTimer = null;
	}, timeoutMs);
}

function formatReminderDueText(item: ReminderLite): string {
	if (item.repeat_rule === "DAILY") {
		const minutes = Math.max(0, Math.min(1439, item.daily_time_minutes ?? 9 * 60));
		const hour = Math.floor(minutes / 60)
			.toString()
			.padStart(2, "0");
		const minute = (minutes % 60).toString().padStart(2, "0");
		return `${tx("每日", "Daily")} ${hour}:${minute}`;
	}

	if (item.repeat_rule === "WEEKLY") {
		const minutes = Math.max(0, Math.min(1439, item.daily_time_minutes ?? 9 * 60));
		const hour = Math.floor(minutes / 60)
			.toString()
			.padStart(2, "0");
		const minute = (minutes % 60).toString().padStart(2, "0");
		const labels = (item.weekly_days ?? [])
			.map((day) => [tx("日", "Sun"), tx("一", "Mon"), tx("二", "Tue"), tx("三", "Wed"), tx("四", "Thu"), tx("五", "Fri"), tx("六", "Sat")][day] ?? `${day}`)
			.join(" ");
		return `${tx("每周", "Weekly")} ${labels} ${hour}:${minute}`.trim();
	}

	const date = new Date(item.next_due_timestamp * 1000);
	return date.toLocaleString(getLocale(), {
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
	});
}

function reportActionError(shortText: string, detail: unknown) {
	setTransientMood(shortText);
	console.error(`[pet] ${shortText}`, detail);
}

function hidePromptBubble() {
	currentPromptKey = "";
	promptBubble.classList.remove("visible", "down");
	promptBubble.replaceChildren();
}

function closeContextMenu() {
	// Native context menu is ephemeral and managed by the OS.
}

async function openContextMenu(clientX: number, clientY: number) {
	try {
		const menu = await Menu.new({
			items: [
				{
					id: "show-calendar",
					text: tx("学习日历", "Learning Calendar"),
					action: () => {
						void setPanelMode(currentPanelMode === "heatmap" ? "summary" : "heatmap");
					},
				},
				{
					id: "show-breakdown",
					text: tx("周活跃", "Weekly Activity"),
					action: () => {
						void setPanelMode(currentPanelMode === "stack" ? "summary" : "stack");
					},
				},
				{
					id: "show-main",
					text: tx("打开主界面", "Open Main"),
					action: () => {
						void invoke("show_main_window").catch((e) => reportActionError(tx("打开主面板失败", "Open main window failed"), e));
					},
				},
				{
					id: "hide-pet",
					text: tx("隐藏桌宠", "Hide Pet"),
					action: () => {
						void invoke("hide_pet_window").catch((e) => reportActionError(tx("隐藏失败", "Hide failed"), e));
					},
				},
				{
					id: "close-pet",
					text: tx("关闭桌宠", "Close Pet"),
					action: () => {
						void invoke("close_pet_window").catch((e) => reportActionError(tx("关闭失败", "Close failed"), e));
					},
				},
			],
		});

		await menu.popup(new LogicalPosition(clientX, clientY), petWindow);
	} catch (e) {
		reportActionError(tx("鍙抽敭鑿滃崟鎵撳紑澶辫触", "Context menu failed"), e);
	}
}

function renderPromptBubble(
	promptKey: string,
	title: string,
	detail: string,
	actions: PromptAction[],
) {
	if (currentPromptKey === promptKey && promptBubble.classList.contains("visible")) {
		return;
	}

	currentPromptKey = promptKey;
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
				reportActionError(tx("提示操作失败", "Action failed"), e);
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

async function refreshPromptBubble() {
	try {
		const [dueReminders, idleItems, pendingItems] = await Promise.all([
			invoke<ReminderLite[]>("list_due_reminders", { limit: 4 }),
			invoke<IdlePromptLite[]>("list_pending_idle_prompts", { limit: 3 }),
			invoke<PendingRuleProcessLite[]>("list_pending_rule_processes", { limit: 3 }),
		]);

		const descriptors: PromptDescriptor[] = [];

		for (const reminder of dueReminders) {
			const key = `reminder-${reminder.id}`;
			descriptors.push({
				key,
				title: tx("日程提醒", "Reminder Due"),
				detail: `${reminder.content} · ${formatReminderDueText(reminder)}`,
				actions: [
					{
						label: tx("完成", "Done"),
						run: async () => {
							await invoke("set_reminder_done", {
								input: { id: reminder.id, done: true },
							});
							setTransientMood(tx("提醒已完成", "Reminder done"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("稍后10分钟", "Snooze 10m"),
						run: async () => {
							await invoke("snooze_reminder", {
								id: reminder.id,
								snoozeSeconds: 600,
							});
							setTransientMood(tx("稍后提醒成功", "Reminder snoozed"));
							await refreshPromptBubble();
						},
					},
				],
			});
		}

		for (const idle of idleItems) {
			const idleSeconds = Math.max(0, Math.floor(idle.duration_ms / 1000));
			const key = `idle-${idle.id}`;
			descriptors.push({
				key,
				title: tx("离开时段待确认", "Idle Segment Confirmation"),
				detail: tx(
					`持续 ${formatSeconds(idleSeconds)}，请尽快归类`,
					`${formatSeconds(idleSeconds)} idle time, please classify`,
				),
				actions: [
					{
						label: tx("学习", "Learn"),
						run: async () => {
							await invoke("resolve_idle_prompt", {
								input: { prompt_id: idle.id, decision: "LEARN", remember_this_session: false },
							});
							setTransientMood(tx("已标记为学习", "Marked as Learn"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("休息", "Break"),
						run: async () => {
							await invoke("resolve_idle_prompt", {
								input: { prompt_id: idle.id, decision: "REST", remember_this_session: false },
							});
							setTransientMood(tx("已标记为休息", "Marked as Break"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("离开", "Away"),
						run: async () => {
							await invoke("resolve_idle_prompt", {
								input: { prompt_id: idle.id, decision: "IDLE", remember_this_session: false },
							});
							setTransientMood(tx("已标记为离开", "Marked as Away"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("稍后提醒", "Remind later"),
						run: async () => {
							await invoke("resolve_idle_prompt", {
								input: { prompt_id: idle.id, decision: "SKIP", remember_this_session: false },
							});
							promptSnoozeUntilByKey.set(key, Date.now() + 60_000);
							await refreshPromptBubble();
						},
					},
				],
			});
		}

		for (const pending of pendingItems) {
			const process = pending.process_name;
			const key = `rule-${process}`;
			descriptors.push({
				key,
				title: tx("新软件待判定", "New App Needs Classification"),
				detail: `${cleanPetProcessName(process)} 路 ${formatSeconds(Math.max(0, pending.total_seconds))}`,
				actions: [
					{
						label: tx("学习", "Learn"),
						run: async () => {
							await invoke("save_app_rule", {
								input: { process_name: process, mapped_type: "LEARN", privacy_level: "NORMAL" },
							});
							setTransientMood(tx("已设为学习", "Set to Learn"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("休息", "Break"),
						run: async () => {
							await invoke("save_app_rule", {
								input: { process_name: process, mapped_type: "REST", privacy_level: "NORMAL" },
							});
							setTransientMood(tx("已设为休息", "Set to Break"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("未分类", "Unclassified"),
						run: async () => {
							await invoke("save_app_rule", {
								input: { process_name: process, mapped_type: "IGNORE", privacy_level: "NORMAL" },
							});
							setTransientMood(tx("已设为未分类", "Set to Unclassified"));
							await refreshPromptBubble();
						},
					},
					{
						label: tx("稍后提醒", "Remind later"),
						run: async () => {
							promptSnoozeUntilByKey.set(key, Date.now() + 15 * 60_000);
							await refreshPromptBubble();
						},
					},
				],
			});
		}

		const nowMs = Date.now();
		for (const [key, until] of promptSnoozeUntilByKey.entries()) {
			if (until <= nowMs) {
				promptSnoozeUntilByKey.delete(key);
			}
		}

		const available = descriptors.filter((item) => {
			const until = promptSnoozeUntilByKey.get(item.key) ?? 0;
			return until <= nowMs;
		});

		if (available.length === 0) {
			hidePromptBubble();
			return;
		}

		const active = available.find((item) => item.key === currentPromptKey) ?? available[0];
		renderPromptBubble(active.key, active.title, active.detail, active.actions);
		return;
	} catch (e) {
		console.error("[pet] refresh prompt bubble failed", e);
	}
}

async function getWorkAreaRect() {
	const monitor = await currentMonitor();
	if (!monitor) {
		return null;
	}

	return {
		x: monitor.workArea.position.x,
		y: monitor.workArea.position.y,
		width: monitor.workArea.size.width,
		height: monitor.workArea.size.height,
	};
}

function clearHideTimer() {
	if (hideTimer !== null) {
		window.clearTimeout(hideTimer);
		hideTimer = null;
	}
}

function clearSnapRetryTimer() {
	if (snapRetryTimer !== null) {
		window.clearTimeout(snapRetryTimer);
		snapRetryTimer = null;
	}
}

async function setExpanded(next: boolean) {
	if (expanded === next) {
		return;
	}

	expanded = next;
	shell.classList.toggle("expanded", next);
}

async function settlePetWindow(mode: "free" | "dock_left" | "dock_right") {
	const result = await invoke<PetWindowSettleResult>("settle_pet_window", {
		input: { mode },
	});
	if (result.state === "dock_left") {
		setPetState("docked_left");
	} else if (result.state === "dock_right") {
		setPetState("docked_right");
	} else {
		setPetState("free");
	}
	return result;
}

async function detectDockEdge(): Promise<DockEdge | null> {
	const rect = await getWorkAreaRect();
	if (!rect) {
		return null;
	}

	const pos = await petWindow.outerPosition();
	const size = await petWindow.outerSize();
	const leftGap = pos.x - rect.x;
	const rightGap = rect.x + rect.width - (pos.x + size.width);

	const candidates: Array<{ edge: DockEdge; metric: number }> = [];
	if (leftGap <= EDGE_SNAP_THRESHOLD) {
		candidates.push({ edge: "left", metric: Math.abs(leftGap) });
	}
	if (rightGap <= EDGE_SNAP_THRESHOLD) {
		candidates.push({ edge: "right", metric: Math.abs(rightGap) });
	}

	if (candidates.length === 0) {
		return null;
	}

	candidates.sort((a, b) => a.metric - b.metric);
	return candidates[0].edge;
}

async function settleAfterDrag() {
	if (isDragging) {
		return;
	}
	const edge = await detectDockEdge();
	const mode = edge === "left" ? "dock_left" : edge === "right" ? "dock_right" : "free";
	await settlePetWindow(mode);
}

function scheduleSettleRetry(attempt = 0) {
	clearSnapRetryTimer();
	snapRetryTimer = window.setTimeout(async () => {
		snapRetryTimer = null;
		if (isDragging) {
			return;
		}
		await settleAfterDrag();
		if (attempt < 4) {
			scheduleSettleRetry(attempt + 1);
		}
	}, 120 + attempt * 60);
}

async function refreshSummary() {
	try {
		const summary: TodaySummary = await getTodaySummary();
		learn.textContent = formatSeconds(summary.learn_seconds);
		rest.textContent = formatSeconds(summary.rest_seconds);
		setMood(tx("自动记录中", "Auto tracking"));
	} catch {
		setMood(tx("状态同步失败，请打开设置查看详情", "Sync failed, open settings for details"));
	}
}

dragArea.addEventListener("contextmenu", (event) => {
	event.preventDefault();
	void openContextMenu(event.clientX, event.clientY);
});

dragArea.addEventListener("pointerdown", (event) => {
	const target = event.target as HTMLElement;
	if (target.closest("button")) {
		return;
	}
	if (event.pointerType === "mouse" && event.button !== 0) {
		return;
	}

	clearHideTimer();
	clearSnapRetryTimer();
	closeContextMenu();
	dragPointerId = event.pointerId;
	isDragging = true;
	dragArea.setPointerCapture?.(event.pointerId);
	void (async () => {
		try {
			await invoke("begin_pet_drag");
		} catch (e) {
			reportActionError(tx("拖拽启动失败", "Drag start failed"), e);
			await finishDrag();
		}
	})();
});

window.addEventListener("pointermove", (event) => {
	if (!isDragging || dragPointerId === null || event.pointerId !== dragPointerId) {
		return;
	}
});

async function finishDrag() {
	if (!isDragging) {
		return;
	}

	isDragging = false;
	if (dragPointerId !== null) {
		dragArea.releasePointerCapture?.(dragPointerId);
	}
	dragPointerId = null;

	hideTimer = window.setTimeout(() => {
		void settleAfterDrag();
	}, 140);
	scheduleSettleRetry();
}

window.addEventListener("pointerup", () => {
	void finishDrag();
});

window.addEventListener("pointercancel", () => {
	void finishDrag();
});

dragArea.addEventListener("lostpointercapture", () => {
	void finishDrag();
});

window.addEventListener("blur", () => {
	closeContextMenu();
	void finishDrag();
});

window.addEventListener("pointerdown", (event) => {
	const target = event.target as Node | null;
	if (!target) {
		closeContextMenu();
		return;
	}
	if (dragArea.contains(target)) {
		return;
	}
	closeContextMenu();
});

window.addEventListener("keydown", (event) => {
	if (event.key === "Escape") {
		closeContextMenu();
	}
});

const handleHoverEnter = () => {
	if (isDragging) {
		return;
	}
	if (!isDockedState()) {
		return;
	}
	clearHideTimer();
	void setExpanded(true);
};

const handleHoverLeave = () => {
	if (isDragging) {
		return;
	}
	clearHideTimer();
	if (!isDockedState()) {
		return;
	}
	if (currentPanelMode !== "summary") {
		return;
	}
	hideTimer = window.setTimeout(() => {
		void setExpanded(false);
	}, 260);
};

dragArea.addEventListener("mouseenter", handleHoverEnter);
dragArea.addEventListener("mouseleave", handleHoverLeave);

void refreshSummary();
void refreshPromptBubble();
void setPanelMode("summary");
setPetState("free");
applyLocalizedStaticText();
void settlePetWindow("free").then(() => setExpanded(true));

const timer = window.setInterval(() => {
	applyLocalizedStaticText();
	void refreshSummary();
	void refreshPromptBubble();
}, 5000);

window.addEventListener("storage", (event) => {
	if (event.key === LOCALE_STORAGE_KEY) {
		applyLocalizedStaticText();
	}
});

window.addEventListener("beforeunload", () => {
	window.clearInterval(timer);
	clearHideTimer();
	clearMoodResetTimer();
	closeContextMenu();
});


