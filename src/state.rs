use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use time::OffsetDateTime;

use crate::markdown::render_markdown;

pub const MAX_RECENT_FILES: usize = 8;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecentFileRecord {
    pub path: String,
    pub display_name: String,
    pub last_opened: i64,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReaderTheme {
    GitHub,
    DocC,
    Basic,
    GitHubDark,
    DocCDark,
    BasicDark,
}

impl ReaderTheme {
    pub const ALL: [ReaderTheme; 6] = [
        ReaderTheme::GitHub,
        ReaderTheme::DocC,
        ReaderTheme::Basic,
        ReaderTheme::GitHubDark,
        ReaderTheme::DocCDark,
        ReaderTheme::BasicDark,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            ReaderTheme::GitHub => "GitHub",
            ReaderTheme::DocC => "DocC",
            ReaderTheme::Basic => "Basic",
            ReaderTheme::GitHubDark => "GitHub Dark",
            ReaderTheme::DocCDark => "DocC Dark",
            ReaderTheme::BasicDark => "Basic Dark",
        }
    }

    pub fn value(self) -> &'static str {
        match self {
            ReaderTheme::GitHub => "github",
            ReaderTheme::DocC => "docc",
            ReaderTheme::Basic => "basic",
            ReaderTheme::GitHubDark => "github-dark",
            ReaderTheme::DocCDark => "docc-dark",
            ReaderTheme::BasicDark => "basic-dark",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            ReaderTheme::GitHub => "theme-github",
            ReaderTheme::DocC => "theme-docc",
            ReaderTheme::Basic => "theme-basic",
            ReaderTheme::GitHubDark => "theme-github-dark",
            ReaderTheme::DocCDark => "theme-docc-dark",
            ReaderTheme::BasicDark => "theme-basic-dark",
        }
    }

    pub fn from_value(value: &str) -> Option<Self> {
        match value {
            "github" => Some(ReaderTheme::GitHub),
            "docc" => Some(ReaderTheme::DocC),
            "basic" => Some(ReaderTheme::Basic),
            "github-dark" => Some(ReaderTheme::GitHubDark),
            "docc-dark" => Some(ReaderTheme::DocCDark),
            "basic-dark" => Some(ReaderTheme::BasicDark),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoredSettings {
    pub selected_theme: ReaderTheme,
    pub auto_reload: bool,
    pub recent_files: Vec<RecentFileRecord>,
}

impl Default for StoredSettings {
    fn default() -> Self {
        Self {
            selected_theme: ReaderTheme::GitHub,
            auto_reload: true,
            recent_files: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub markdown_text: String,
    pub rendered_html: String,
    pub document_name: Option<String>,
    pub current_path: Option<PathBuf>,
    pub error_message: Option<String>,
    pub auto_reload: bool,
    pub selected_theme: ReaderTheme,
    pub recent_files: Vec<RecentFileRecord>,
    pub last_modified: Option<SystemTime>,
}

impl AppState {
    pub fn from_settings(settings: StoredSettings) -> Self {
        Self {
            markdown_text: "# MReader\n\nOpen a Markdown file to preview it.".to_string(),
            rendered_html: render_markdown("# MReader\n\nOpen a Markdown file to preview it."),
            document_name: None,
            current_path: None,
            error_message: None,
            auto_reload: settings.auto_reload,
            selected_theme: settings.selected_theme,
            recent_files: settings.recent_files,
            last_modified: None,
        }
    }

    pub fn to_settings(&self) -> StoredSettings {
        StoredSettings {
            selected_theme: self.selected_theme,
            auto_reload: self.auto_reload,
            recent_files: self.recent_files.clone(),
        }
    }

    pub fn load_path(&mut self, path: PathBuf) {
        match read_markdown(&path) {
            Ok((content, modified)) => {
                self.markdown_text = content.clone();
                self.rendered_html = render_markdown(&content);
                self.document_name = path.file_name().map(|name| name.to_string_lossy().to_string());
                self.current_path = Some(path.clone());
                self.error_message = None;
                self.last_modified = Some(modified);
                self.add_recent_file(&path);
            }
            Err(message) => {
                self.error_message = Some(message);
            }
        }
    }

    pub fn open_recent(&mut self, record: &RecentFileRecord) {
        let path = PathBuf::from(&record.path);
        if !path.exists() {
            self.error_message = Some(format!("Unable to access {}.", record.display_name));
            self.remove_recent_by_path(&record.path);
            return;
        }
        self.load_path(path);
    }

    pub fn clear_recent_files(&mut self) {
        self.recent_files.clear();
    }

    pub fn file_modified_time(path: &Path) -> Result<SystemTime, String> {
        let metadata = fs::metadata(path)
            .map_err(|err| format!("Failed to read metadata: {}", err))?;
        metadata
            .modified()
            .map_err(|err| format!("Failed to read modification time: {}", err))
    }

    fn add_recent_file(&mut self, path: &Path) {
        let display_name = path.file_name().map(|name| name.to_string_lossy().to_string());
        let display_name = display_name.unwrap_or_else(|| "Untitled".to_string());
        self.recent_files.retain(|record| record.path != path.to_string_lossy());
        let record = RecentFileRecord {
            path: path.to_string_lossy().to_string(),
            display_name,
            last_opened: OffsetDateTime::now_utc().unix_timestamp(),
        };
        self.recent_files.insert(0, record);
        if self.recent_files.len() > MAX_RECENT_FILES {
            self.recent_files.truncate(MAX_RECENT_FILES);
        }
    }

    fn remove_recent_by_path(&mut self, path: &str) {
        self.recent_files.retain(|record| record.path != path);
    }
}

fn read_markdown(path: &Path) -> Result<(String, SystemTime), String> {
    let data = fs::read(path).map_err(|err| format!("Failed to read file: {}", err))?;
    let content = String::from_utf8_lossy(&data).to_string();
    let modified = AppState::file_modified_time(path).unwrap_or_else(|_| SystemTime::now());
    Ok((content, modified))
}
