use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub(crate) struct PrivacySettings {
    pub(crate) curtain_enabled: bool,
    pub(crate) browser_title_mode: String,
    pub(crate) whitelist_only_enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct UpdatePrivacySettingsInput {
    pub(crate) curtain_enabled: bool,
    pub(crate) browser_title_mode: String,
    pub(crate) whitelist_only_enabled: bool,
}

#[derive(Deserialize)]
pub(crate) struct SetWhitelistItemInput {
    pub(crate) process_name: String,
    pub(crate) enabled: bool,
}
