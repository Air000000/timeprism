import { invoke } from "@tauri-apps/api/core";
import { LogicalPosition } from "@tauri-apps/api/dpi";
import { Menu } from "@tauri-apps/api/menu";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { getTodaySummary, type TodaySummary } from "./api";
import {
	LOCALE_STORAGE_KEY,
	getStoredOrBrowserLocale,
	translateForLocale,
	type LocaleCode,
} from "./lib/locale";
import {
	dockEdgeForState,
	isDockedState,
	petDockStateFromSettleState,
	type DockEdge,
	type PetDockState,
	type PetWindowSettleResult,
} from "./lib/petDock";
import { queryPetElements, renderPetShell } from "./lib/petDom";
import {
	PET_CHARACTER_DEFAULT_SRC,
	PET_CHARACTER_DOCKED_LEFT_SRC,
	PET_CHARACTER_DOCKED_RIGHT_SRC,
	PET_CHARACTER_PRIMARY_SRC,
	getPetCharacterSrc,
	savePetCharacterSrc,
} from "./lib/petCharacter";
import {
	availablePromptDescriptors,
	pickActivePromptDescriptor,
	pruneExpiredPromptSnoozes,
	type IdlePromptLite,
	type PendingRuleProcessLite,
	type PromptDescriptor,
	type ReminderLite,
} from "./lib/petPrompts";
import {
	hidePromptBubble,
	renderPromptBubble,
	type PromptBubbleState,
} from "./lib/petPromptDom";
import { cleanPetProcessName } from "./lib/petProcessName";
import { formatPetReminderDueText } from "./lib/petReminderText";
import { formatSeconds } from "./lib/time";
import "./pet.css";

const petWindow = getCurrentWindow();
const EDGE_SNAP_THRESHOLD = 72;

function getLocale(): LocaleCode {
	return getStoredOrBrowserLocale();
}

function tx(zh: string, en: string): string {
	return translateForLocale(getLocale(), zh, en);
}

let petState: PetDockState = "free";
let isDragging = false;
let expanded = false;
let hideTimer: number | null = null;
let moodResetTimer: number | null = null;
let dragPointerId: number | null = null;
let snapRetryTimer: number | null = null;
let currentPanelMode: "summary" | "heatmap" | "stack" = "summary";

const app = document.querySelector<HTMLDivElement>("#pet-app");
if (!app) {
	throw new Error("pet app root not found");
}

renderPetShell(app, tx, PET_CHARACTER_PRIMARY_SRC);
const {
	mood,
	learn,
	rest,
	learnToken,
	restToken,
	characterImage,
	dragArea,
	shell,
	promptBubble,
} = queryPetElements();

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

function applyPetCharacter() {
	characterImage.src = getPetCharacterSrc();
	characterImage.onerror = () => {
		if (characterImage.src.endsWith(PET_CHARACTER_DEFAULT_SRC)) {
			return;
		}
		characterImage.src = PET_CHARACTER_DEFAULT_SRC;
	};
}

function applyDockedAppearance() {
	const docked = isDockedState(petState);
	const dockEdge = dockEdgeForState(petState);
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
	savePetCharacterSrc(src);
	applyDockedAppearance();
}

(window as Window & { setTimePrismPetCharacter?: (src: string) => void }).setTimePrismPetCharacter = setPetCharacterSrc;

const promptSnoozeUntilByKey = new Map<string, number>();
const promptBubbleState: PromptBubbleState = { currentPromptKey: "" };

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

function reportActionError(shortText: string, detail: unknown) {
	setTransientMood(shortText);
	console.error(`[pet] ${shortText}`, detail);
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
		reportActionError(tx("右键菜单打开失败", "Context menu failed"), e);
	}
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
				detail: `${reminder.content} · ${formatPetReminderDueText(
					reminder,
					tx,
					getLocale(),
				)}`,
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
				detail: `${cleanPetProcessName(process, tx)} · ${formatSeconds(
					Math.max(0, pending.total_seconds),
				)}`,
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
		pruneExpiredPromptSnoozes(promptSnoozeUntilByKey, nowMs);
		const active = pickActivePromptDescriptor(
			availablePromptDescriptors(descriptors, promptSnoozeUntilByKey, nowMs),
			promptBubbleState.currentPromptKey,
		);

		if (!active) {
			hidePromptBubble(promptBubble, promptBubbleState);
			return;
		}

		renderPromptBubble({
			promptBubble,
			state: promptBubbleState,
			promptKey: active.key,
			title: active.title,
			detail: active.detail,
			actions: active.actions,
			onActionError: (e) => reportActionError(tx("提示操作失败", "Action failed"), e),
		});
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
	setPetState(petDockStateFromSettleState(result.state));
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
	if (!isDockedState(petState)) {
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
	if (!isDockedState(petState)) {
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


