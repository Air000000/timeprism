import { computed, type Ref } from "vue";
import type {
  SettingsViewContext,
  TranslateFn,
} from "../components/viewContexts";
import type { LocaleCode } from "./useLocale";
import type { ThemeMode } from "./useThemeMode";
import type { PrivacySettings } from "../api";

type SettingsViewContextOptions = {
  tx: TranslateFn;
  locale: Readonly<Ref<LocaleCode>>;
  onLocaleChange: SettingsViewContext["onLocaleChange"];
  themeMode: Readonly<Ref<ThemeMode>>;
  toggleThemeMode: SettingsViewContext["toggleThemeMode"];
  autoStartEnabled: Readonly<Ref<boolean>>;
  onAutoStartChange: SettingsViewContext["onAutoStartChange"];
  privacy: Readonly<Ref<PrivacySettings>>;
  handleSavePrivacySettings: SettingsViewContext["handleSavePrivacySettings"];
  privacyFeedbackType: Readonly<Ref<SettingsViewContext["privacyFeedbackType"]>>;
  privacyFeedback: Readonly<Ref<string>>;
  whitelistInput: Readonly<Ref<string>>;
  onWhitelistInput: SettingsViewContext["onWhitelistInput"];
  handleAddWhitelist: SettingsViewContext["handleAddWhitelist"];
  whitelist: Readonly<Ref<string[]>>;
  handleRemoveWhitelist: SettingsViewContext["handleRemoveWhitelist"];
};

export function useSettingsViewContext({
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
}: SettingsViewContextOptions) {
  return computed<SettingsViewContext>(() => ({
    tx,
    locale: locale.value,
    onLocaleChange,
    themeMode: themeMode.value,
    toggleThemeMode,
    autoStartEnabled: autoStartEnabled.value,
    onAutoStartChange,
    privacy: privacy.value,
    handleSavePrivacySettings,
    privacyFeedbackType: privacyFeedbackType.value,
    privacyFeedback: privacyFeedback.value,
    whitelistInput: whitelistInput.value,
    onWhitelistInput,
    handleAddWhitelist,
    whitelist: whitelist.value,
    handleRemoveWhitelist,
  }));
}
