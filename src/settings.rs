use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use crate::state::StoredSettings;

const ORG_QUALIFIER: &str = "com";
const ORG_NAME: &str = "stuart";
const APP_NAME: &str = "MReader";

pub fn load_settings() -> StoredSettings {
    let Some(path) = settings_path() else {
        return StoredSettings::default();
    };

    let Ok(data) = fs::read(path) else {
        return StoredSettings::default();
    };

    serde_json::from_slice(&data).unwrap_or_default()
}

pub fn save_settings(settings: &StoredSettings) -> std::io::Result<()> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_vec_pretty(settings).unwrap_or_default();
    fs::write(path, data)
}

fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from(ORG_QUALIFIER, ORG_NAME, APP_NAME)
        .map(|dirs| dirs.config_dir().join("settings.json"))
}
