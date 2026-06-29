import { ref } from "vue";
import {
  getAutoStartEnabled,
  getPrivacySettings,
  listWhitelist,
  setAutoStartEnabled,
  setWhitelistItem,
  type PrivacySettings,
  updatePrivacySettings,
} from "../api";
import {
  defaultPrivacyFeedback,
  privacySaveErrorFeedback,
  privacySettingsSavedFeedback,
  whitelistAddedFeedback,
  whitelistAddErrorFeedback,
  whitelistRemovedFeedback,
  whitelistRemoveErrorFeedback,
} from "../lib/settingsPrivacyFeedback";
import {
  buildPrivacySettingsUpdateInput,
  buildWhitelistItemInput,
  defaultPrivacySettings,
  normalizeWhitelistProcessName,
  settingsCheckedFromEvent,
  settingsTextValueFromEvent,
} from "../lib/settingsPrivacy";

type TranslateFn = (zh: string, en: string) => string;
type FeedbackTone = "info" | "ok" | "warn" | "error";

type UseSettingsPrivacyOptions = {
  tx: TranslateFn;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

export function useSettingsPrivacy({ tx, refreshData, setErrorMessage }: UseSettingsPrivacyOptions) {
  const loadingSettings = ref(false);
  const settingsLoadedAt = ref(0);
  const privacy = ref<PrivacySettings>(defaultPrivacySettings());
  const autoStartEnabled = ref(false);
  const whitelist = ref<string[]>([]);
  const whitelistInput = ref("code.exe");
  const privacyFeedback = ref(defaultPrivacyFeedback(tx));
  const privacyFeedbackType = ref<FeedbackTone>("info");

  function resetPrivacyFeedback() {
    privacyFeedback.value = defaultPrivacyFeedback(tx);
  }

  async function refreshPrivacy() {
    try {
      const [settings, list] = await Promise.all([getPrivacySettings(), listWhitelist()]);
      privacy.value = settings;
      whitelist.value = list;
    } catch (e) {
      setErrorMessage(e);
    }
  }

  async function refreshAutoStartSetting() {
    try {
      autoStartEnabled.value = await getAutoStartEnabled();
    } catch (e) {
      setErrorMessage(e);
    }
  }

  async function refreshSettingsData() {
    const now = Date.now();
    if (settingsLoadedAt.value > 0 && now - settingsLoadedAt.value < 60_000) {
      return;
    }
    if (loadingSettings.value) {
      return;
    }
    loadingSettings.value = true;
    try {
      await Promise.all([refreshPrivacy(), refreshAutoStartSetting()]);
      settingsLoadedAt.value = Date.now();
    } finally {
      loadingSettings.value = false;
    }
  }

  async function handleSavePrivacySettings() {
    try {
      await setAutoStartEnabled(autoStartEnabled.value);
      await updatePrivacySettings(buildPrivacySettingsUpdateInput(privacy.value));
      privacyFeedbackType.value = "ok";
      privacyFeedback.value = privacySettingsSavedFeedback(privacy.value.browser_title_mode, tx);
      await refreshPrivacy();
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      privacyFeedbackType.value = "error";
      privacyFeedback.value = privacySaveErrorFeedback(e, tx);
    }
  }

  async function runWhitelistMutation(
    task: () => Promise<void>,
    successType: FeedbackTone,
    buildSuccessFeedback: () => string,
    buildErrorFeedback: (error: unknown) => string,
    onSuccess?: () => void,
  ) {
    try {
      await task();
      onSuccess?.();
      privacyFeedbackType.value = successType;
      privacyFeedback.value = buildSuccessFeedback();
      await refreshPrivacy();
    } catch (e) {
      setErrorMessage(e);
      privacyFeedbackType.value = "error";
      privacyFeedback.value = buildErrorFeedback(e);
    }
  }

  async function handleAddWhitelist() {
    const processName = normalizeWhitelistProcessName(whitelistInput.value);
    if (!processName) {
      return;
    }

    await runWhitelistMutation(
      () => setWhitelistItem(buildWhitelistItemInput(processName, true)),
      "ok",
      () => whitelistAddedFeedback(processName, tx),
      (error) => whitelistAddErrorFeedback(error, tx),
      () => {
        whitelistInput.value = "";
      },
    );
  }

  async function handleRemoveWhitelist(processName: string) {
    await runWhitelistMutation(
      () => setWhitelistItem(buildWhitelistItemInput(processName, false)),
      "warn",
      () => whitelistRemovedFeedback(processName, tx),
      (error) => whitelistRemoveErrorFeedback(error, tx),
    );
  }

  function onAutoStartChange(event: Event) {
    autoStartEnabled.value = settingsCheckedFromEvent(event);
  }

  function onWhitelistInput(event: Event) {
    whitelistInput.value = settingsTextValueFromEvent(event);
  }

  return {
    privacy,
    autoStartEnabled,
    whitelist,
    whitelistInput,
    privacyFeedback,
    privacyFeedbackType,
    refreshSettingsData,
    resetPrivacyFeedback,
    handleSavePrivacySettings,
    handleAddWhitelist,
    handleRemoveWhitelist,
    onAutoStartChange,
    onWhitelistInput,
  };
}
