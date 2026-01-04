use std::path::{Path, PathBuf};

use dioxus::prelude::{Signal, SyncStorage, WritableExt};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    watched_path: Option<PathBuf>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_path: None,
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut watcher) = self.watcher.take() {
            if let Some(path) = self.watched_path.take() {
                let _ = watcher.unwatch(&path);
            }
        } else {
            self.watched_path = None;
        }
    }

    pub fn watch(
        &mut self,
        path: &Path,
        mut file_tick: Signal<u64, SyncStorage>,
    ) -> Result<(), String> {
        if self
            .watched_path
            .as_ref()
            .map(|watched| watched == path)
            .unwrap_or(false)
        {
            return Ok(());
        }

        self.stop();

        let mut watcher = notify::recommended_watcher(move |result: Result<Event, notify::Error>| {
            if let Ok(event) = result {
                if should_trigger(&event.kind) {
                    let mut value = file_tick.write();
                    *value = value.wrapping_add(1);
                }
            }
        })
        .map_err(|err| err.to_string())?;

        watcher
            .configure(Config::default())
            .map_err(|err| err.to_string())?;

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|err| err.to_string())?;

        self.watcher = Some(watcher);
        self.watched_path = Some(path.to_path_buf());

        Ok(())
    }
}

fn should_trigger(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Modify(_)
            | EventKind::Create(_)
            | EventKind::Remove(_)
            | EventKind::Any
    )
}
