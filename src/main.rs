mod app;
#[cfg(target_os = "macos")]
mod file_events;
mod markdown;
mod menu;
mod settings;
mod state;
mod watcher;

use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};

/// Newtype wrapping the mpsc receiver for Dioxus context injection.
/// Arc<Mutex<_>> satisfies the Clone + Send + Sync + 'static bounds
/// required by LaunchBuilder::with_context.
#[derive(Clone)]
pub struct FileEventReceiver(pub Arc<Mutex<mpsc::Receiver<PathBuf>>>);

fn main() {
    let (tx, rx) = mpsc::channel::<PathBuf>();

    // Register Apple Events handler BEFORE launch() — macOS may deliver
    // the 'odoc' event synchronously during startup.
    #[cfg(target_os = "macos")]
    file_events::register_open_document_handler(tx.clone());

    // CLI / script fallback: `m_reader /path/to/file.md`
    if let Some(path) = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .filter(|p| p.exists())
    {
        let _ = tx.send(path);
    }

    drop(tx);

    let settings = settings::load_settings();
    let menu = menu::build_menu(&settings);
    let window = dioxus::desktop::WindowBuilder::new()
        .with_title("M Reader")
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(720.0, 520.0));
    let config = dioxus::desktop::Config::new()
        .with_menu(menu)
        .with_window(window);

    dioxus::LaunchBuilder::new()
        .with_cfg(config)
        .with_context(FileEventReceiver(Arc::new(Mutex::new(rx))))
        .launch(app::app);
}
