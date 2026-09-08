import type { Ref } from "vue";
import type { TranslateFn } from "../components/viewContexts";
import { useLazySettingsMount } from "./useLazySettingsMount";
import type { LocaleCode } from "./useLocale";
import { useSettingsPrivacy } from "./useSettingsPrivacy";
import { useSettingsViewContext } from "./useSettingsViewContext";
import type { ThemeMode } from "./useThemeMode";

type UseAppSettingsSectionOptions = {
  tx: TranslateFn;
  locale: Readonly<Ref<LocaleCode>>;
  onLocaleChange: (event: Event) => void;
  themeMode: Readonly<Ref<ThemeMode>>;
  toggleThemeMode: () => void;
  refreshData: () => Promise<void>;
  setErrorMessage: (error: unknown) => void;
};

export function useAppSettingsSection({
  tx,
  locale,
  onLocaleChange,
  themeMode,
  toggleThemeMode,
  refreshData,
  setErrorMessage,
}: UseAppSettingsSectionOptions) {
  const {
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
  } = useSettingsPrivacy({ tx, refreshData, setErrorMessage });
  const {
    privacyViewMounted,
    showSettingsViewAndRefresh,
    startSettingsWarmup,
    cleanupSettingsWarmup,
  } = useLazySettingsMount({ refreshSettingsData });
  const settingsCtx = useSettingsViewContext({
    tx,
    locale,
    onLocaleChange,
    themeMode,
    toggleThemeMode,
    autoStartEnabled,
    onAutoStartChange,
    privacy,
    handleSavePrivacySettings,
    privacyFeedbackType,
    privacyFeedback,
    whitelistInput,
    onWhitelistInput,
    handleAddWhitelist,
    whitelist,
    handleRemoveWhitelist,
  });

  return {
    cleanupSettingsWarmup,
    privacyViewMounted,
    refreshSettingsData,
    resetPrivacyFeedback,
    settingsCtx,
    showSettingsViewAndRefresh,
    startSettingsWarmup,
  };
}
