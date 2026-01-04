mod app;
mod markdown;
mod menu;
mod settings;
mod state;
mod watcher;

fn main() {
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
        .launch(app::app);
}
