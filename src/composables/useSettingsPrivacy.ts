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

type TranslateFn = (zh: string, en: string) => string;
type FeedbackTone = "info" | "ok" | "warn" | "error";

type UseSettingsPrivacyOptions = {
  tx: TranslateFn;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

function defaultPrivacySettings(): PrivacySettings {
  return {
    curtain_enabled: false,
    browser_title_mode: "BLUR",
    whitelist_only_enabled: false,
  };
}

export function useSettingsPrivacy({ tx, refreshData, setErrorMessage }: UseSettingsPrivacyOptions) {
  const loadingSettings = ref(false);
  const settingsLoadedAt = ref(0);
  const privacy = ref<PrivacySettings>(defaultPrivacySettings());
  const autoStartEnabled = ref(false);
  const whitelist = ref<string[]>([]);
  const whitelistInput = ref("code.exe");
  const privacyFeedback = ref("未保存隐私设置");
  const privacyFeedbackType = ref<FeedbackTone>("info");

  function browserModeText(mode: PrivacySettings["browser_title_mode"]): string {
    if (mode === "FULL") {
      return tx("完整标题", "Full title");
    }
    if (mode === "BLUR") {
      return tx("模糊标题", "Blurred title");
    }
    return tx("不采集标题", "No title capture");
  }

  function resetPrivacyFeedback() {
    privacyFeedback.value = tx("未保存隐私设置", "Privacy settings not saved");
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
      await updatePrivacySettings({
        curtain_enabled: privacy.value.curtain_enabled,
        browser_title_mode: privacy.value.browser_title_mode,
        whitelist_only_enabled: privacy.value.whitelist_only_enabled,
      });
      privacyFeedbackType.value = "ok";
      privacyFeedback.value = tx(
        `隐私设置已保存（浏览器模式：${browserModeText(privacy.value.browser_title_mode)}）。`,
        `Privacy settings saved (browser mode: ${browserModeText(privacy.value.browser_title_mode)}).`,
      );
      await refreshPrivacy();
      await refreshData();
    } catch (e) {
      setErrorMessage(e);
      privacyFeedbackType.value = "error";
      privacyFeedback.value = tx(`保存失败：${e}`, `Save failed: ${e}`);
    }
  }

  async function handleAddWhitelist() {
    const processName = whitelistInput.value.trim().toLowerCase();
    if (!processName) {
      return;
    }

    try {
      await setWhitelistItem({
        process_name: processName,
        enabled: true,
      });
      whitelistInput.value = "";
      privacyFeedbackType.value = "ok";
      privacyFeedback.value = tx(`已加入白名单：${processName}`, `Added to whitelist: ${processName}`);
      await refreshPrivacy();
    } catch (e) {
      setErrorMessage(e);
      privacyFeedbackType.value = "error";
      privacyFeedback.value = tx(`添加失败：${e}`, `Add failed: ${e}`);
    }
  }

  async function handleRemoveWhitelist(processName: string) {
    try {
      await setWhitelistItem({
        process_name: processName,
        enabled: false,
      });
      privacyFeedbackType.value = "warn";
      privacyFeedback.value = tx(`已移除白名单：${processName}`, `Removed from whitelist: ${processName}`);
      await refreshPrivacy();
    } catch (e) {
      setErrorMessage(e);
      privacyFeedbackType.value = "error";
      privacyFeedback.value = tx(`移除失败：${e}`, `Remove failed: ${e}`);
    }
  }

  function onAutoStartChange(event: Event) {
    const input = event.target as HTMLInputElement;
    autoStartEnabled.value = input.checked;
  }

  function onWhitelistInput(event: Event) {
    const input = event.target as HTMLInputElement;
    whitelistInput.value = input.value;
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
