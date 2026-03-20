use dioxus::desktop::use_muda_event_handler;
use dioxus::prelude::dioxus_core::Task;
use dioxus::prelude::*;
use std::path::PathBuf;
use std::time::Duration;

use crate::FileEventReceiver;

use crate::menu;
use crate::settings::{load_settings, save_settings};
use crate::state::{AppState, ReaderTheme, StoredSettings};
use crate::watcher::FileWatcher;

const MAIN_CSS: &str = include_str!("../assets/main.css");

pub fn app() -> Element {
    let mut state = use_signal(|| AppState::from_settings(load_settings()));
    let file_tick = use_signal_sync(|| 0u64);
    let watcher = use_signal(FileWatcher::new);
    let debounce_task = use_signal(|| None as Option<Task>);

    // Background loop: receives file paths from Finder double-click (Apple Events)
    // and CLI args. Runs once for the lifetime of the component.
    let file_rx = use_context::<FileEventReceiver>();
    use_future(move || {
        let mut state = state.clone();
        let rx = file_rx.0.clone();
        async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let path = rx.try_lock().ok().and_then(|g| g.try_recv().ok());
                if let Some(path) = path {
                    let settings = {
                        let mut snapshot = state.write();
                        snapshot.load_path(path);
                        menu::sync_recent_menu(&snapshot.recent_files);
                        snapshot.to_settings()
                    };
                    persist_settings(settings);
                }
            }
        }
    });

    use_effect({
        let mut watcher = watcher.clone();
        let file_tick = file_tick;
        let mut state = state.clone();
        move || {
            let (auto_reload, path) = {
                let snapshot = state.read();
                (snapshot.auto_reload, snapshot.current_path.clone())
            };

            let mut watcher = watcher.write();
            if !auto_reload || path.is_none() {
                watcher.stop();
                return;
            }

            let path = path.expect("path checked");
            if let Err(message) = watcher.watch(&path, file_tick) {
                let mut snapshot = state.write();
                snapshot.error_message = Some(message);
            }
        }
    });

    use_effect({
        let file_tick = file_tick;
        let mut debounce_task = debounce_task.clone();
        let state = state.clone();
        move || {
            let _ = *file_tick.read();
            if let Some(task) = debounce_task.write().take() {
                task.cancel();
            }

            let mut state = state.clone();
            let task = spawn(async move {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let (auto_reload, path, last_modified) = {
                    let snapshot = state.read();
                    (
                        snapshot.auto_reload,
                        snapshot.current_path.clone(),
                        snapshot.last_modified,
                    )
                };

                if !auto_reload {
                    return;
                }

                let Some(path) = path else { return; };

                match AppState::file_modified_time(&path) {
                    Ok(modified) => {
                        // Reload even when mtimes are equal to avoid missing rapid saves
                        // on filesystems with coarse timestamp resolution.
                        let should_reload = last_modified
                            .map(|previous| modified >= previous)
                            .unwrap_or(true);
                        if should_reload {
                            let settings = {
                                let mut snapshot = state.write();
                                snapshot.load_path(path);
                                menu::sync_recent_menu(&snapshot.recent_files);
                                snapshot.to_settings()
                            };
                            persist_settings(settings);
                        }
                    }
                    Err(message) => {
                        let mut snapshot = state.write();
                        snapshot.error_message = Some(message);
                    }
                }
            });

            *debounce_task.write() = Some(task);
        }
    });

    {
        let mut state = state.clone();
        use_muda_event_handler(move |event| {
            let id = event.id().as_ref();
            if id == menu::MENU_OPEN {
                let mut state = state.clone();
                dioxus::prelude::spawn(async move {
                    if let Some(path) = open_markdown_dialog().await {
                        let settings = {
                            let mut snapshot = state.write();
                            snapshot.load_path(path);
                            menu::sync_recent_menu(&snapshot.recent_files);
                            snapshot.to_settings()
                        };
                        persist_settings(settings);
                    }
                });
                return;
            }

            if id == menu::MENU_CLEAR_RECENT {
                let settings = {
                    let mut snapshot = state.write();
                    snapshot.clear_recent_files();
                    menu::sync_recent_menu(&snapshot.recent_files);
                    snapshot.to_settings()
                };
                persist_settings(settings);
                return;
            }

            if id == menu::MENU_AUTO_RELOAD {
                let settings = {
                    let mut snapshot = state.write();
                    snapshot.auto_reload = !snapshot.auto_reload;
                    menu::sync_auto_reload(snapshot.auto_reload);
                    snapshot.to_settings()
                };
                persist_settings(settings);
                return;
            }

            if let Some(theme) = menu::theme_from_id(id) {
                let settings = {
                    let mut snapshot = state.write();
                    snapshot.selected_theme = theme;
                    menu::sync_theme(theme);
                    snapshot.to_settings()
                };
                persist_settings(settings);
                return;
            }

            if let Some(index) = menu::recent_index_from_id(id) {
                let settings = {
                    let mut snapshot = state.write();
                    if let Some(record) = snapshot.recent_files.get(index).cloned() {
                        snapshot.open_recent(&record);
                    }
                    menu::sync_recent_menu(&snapshot.recent_files);
                    snapshot.to_settings()
                };
                persist_settings(settings);
            }
        });
    }

    let snapshot = state.read().clone();
    let theme_class = snapshot.selected_theme.css_class();
    let recent_items = snapshot
        .recent_files
        .iter()
        .cloned()
        .map(|record| {
            let mut state = state.clone();
            rsx!(button {
                class: "recent-item",
                onclick: move |_| {
                    let settings = {
                        let mut snapshot = state.write();
                        snapshot.open_recent(&record);
                        menu::sync_recent_menu(&snapshot.recent_files);
                        snapshot.to_settings()
                    };
                    persist_settings(settings);
                },
                span { class: "recent-name", "{record.display_name}" }
                span { class: "recent-path", "{record.path}" }
            })
        });

    rsx! {
        style { "{MAIN_CSS}" }
        div {
            class: format!("app {}", theme_class),
            div {
                class: "header",
                div {
                    class: "title-block",
                    h1 { "{snapshot.document_name.clone().unwrap_or_else(|| \"No Document Open\".to_string())}" }
                    p { class: "subtitle", "Read-only Markdown preview" }
                }
                div {
                    class: "controls",
                    label {
                        class: "toggle",
                        input {
                            r#type: "checkbox",
                            checked: snapshot.auto_reload,
                            onclick: move |_| {
                                let settings = {
                                    let mut snapshot = state.write();
                                    snapshot.auto_reload = !snapshot.auto_reload;
                                    menu::sync_auto_reload(snapshot.auto_reload);
                                    snapshot.to_settings()
                                };
                                persist_settings(settings);
                            }
                        }
                        span { "Auto-Reload" }
                    }
                    select {
                        class: "theme-select",
                        value: snapshot.selected_theme.value(),
                        onchange: move |evt| {
                            if let Some(theme) = ReaderTheme::from_value(&evt.value()) {
                                let settings = {
                                    let mut snapshot = state.write();
                                    snapshot.selected_theme = theme;
                                    menu::sync_theme(theme);
                                    snapshot.to_settings()
                                };
                                persist_settings(settings);
                            }
                        },
                        {ReaderTheme::ALL.into_iter().map(|theme| {
                            rsx!(option {
                                value: theme.value(),
                                "{theme.display_name()}"
                            })
                        })}
                    }
                    button {
                        class: "primary",
                        onclick: move |_| {
                            let mut state = state.clone();
                            async move {
                                if let Some(path) = open_markdown_dialog().await {
                                    let settings = {
                                        let mut snapshot = state.write();
                                        snapshot.load_path(path);
                                        menu::sync_recent_menu(&snapshot.recent_files);
                                        snapshot.to_settings()
                                    };
                                    persist_settings(settings);
                                }
                            }
                        },
                        "Open Markdown"
                    }
                }
            }
            if let Some(error) = snapshot.error_message.as_ref() {
                div { class: "error", "{error}" }
            }
            div {
                class: "content",
                if snapshot.document_name.is_none() {
                    div {
                        class: "empty-state",
                        h2 { "No Document Open" }
                        p { class: "muted", "Open a Markdown file or choose one from your recent list." }
                        button {
                            class: "primary",
                            onclick: move |_| {
                                let mut state = state.clone();
                                async move {
                                    if let Some(path) = open_markdown_dialog().await {
                                        let settings = {
                                            let mut snapshot = state.write();
                                            snapshot.load_path(path);
                                            menu::sync_recent_menu(&snapshot.recent_files);
                                            snapshot.to_settings()
                                        };
                                        persist_settings(settings);
                                    }
                                }
                            },
                            "Open Markdown"
                        }
                        if snapshot.recent_files.is_empty() {
                            p { class: "muted", "No recent files yet." }
                        } else {
                            div {
                                class: "recent-list",
                                div { class: "recent-header", "Recent Files" }
                                {recent_items}
                                button {
                                    class: "clear-recent",
                                    onclick: move |_| {
                                        let settings = {
                                            let mut snapshot = state.write();
                                            snapshot.clear_recent_files();
                                            menu::sync_recent_menu(&snapshot.recent_files);
                                            snapshot.to_settings()
                                        };
                                        persist_settings(settings);
                                    },
                                    "Clear Recent Files"
                                }
                            }
                        }
                    }
                } else {
                    div {
                        class: "markdown-body",
                        dangerous_inner_html: "{snapshot.rendered_html}",
                    }
                }
            }
        }
    }
}

async fn open_markdown_dialog() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown"])
        .pick_file()
        .await
        .map(|handle| handle.path().to_path_buf())
}

fn persist_settings(settings: StoredSettings) {
    if let Err(err) = save_settings(&settings) {
        eprintln!("Failed to save settings: {err}");
    }
}
