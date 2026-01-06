use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

use std::io::{self, ErrorKind};

use crate::state::{MAX_RECENT_FILES, StoredSettings};

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

    let settings: StoredSettings = serde_json::from_slice(&data).unwrap_or_default();

    normalize_settings(settings)
}

pub fn save_settings(settings: &StoredSettings) -> std::io::Result<()> {
    let Some(path) = settings_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data =
        serde_json::to_vec_pretty(settings).map_err(|err| io::Error::new(ErrorKind::Other, err))?;
    fs::write(path, data)
}

fn settings_path() -> Option<PathBuf> {
    ProjectDirs::from(ORG_QUALIFIER, ORG_NAME, APP_NAME)
        .map(|dirs| dirs.config_dir().join("settings.json"))
}

fn normalize_settings(mut settings: StoredSettings) -> StoredSettings {
    settings
        .recent_files
        .sort_by(|a, b| b.last_opened.cmp(&a.last_opened));
    settings.recent_files.truncate(MAX_RECENT_FILES);
    settings
}
