use std::cell::RefCell;

use dioxus::desktop::muda::{
    accelerator::{Accelerator, Code, CMD_OR_CTRL},
    CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu,
};

use crate::state::{RecentFileRecord, ReaderTheme, StoredSettings};

pub const MENU_OPEN: &str = "mreader.open";
pub const MENU_AUTO_RELOAD: &str = "mreader.auto_reload";
pub const MENU_CLEAR_RECENT: &str = "mreader.clear_recent";
pub const MENU_THEME_GITHUB: &str = "mreader.theme.github";
pub const MENU_THEME_DOCC: &str = "mreader.theme.docc";
pub const MENU_THEME_BASIC: &str = "mreader.theme.basic";
pub const MENU_THEME_GITHUB_DARK: &str = "mreader.theme.github.dark";
pub const MENU_THEME_DOCC_DARK: &str = "mreader.theme.docc.dark";
pub const MENU_THEME_BASIC_DARK: &str = "mreader.theme.basic.dark";
pub const MENU_RECENT_PREFIX: &str = "mreader.recent.";
const MENU_RECENT_EMPTY: &str = "mreader.recent.empty";

thread_local! {
    static MENU_BINDINGS: RefCell<Option<MenuBindings>> = RefCell::new(None);
}

struct MenuBindings {
    recent_menu: Submenu,
    auto_reload_item: CheckMenuItem,
    theme_items: ThemeItems,
}

struct ThemeItems {
    github: CheckMenuItem,
    docc: CheckMenuItem,
    basic: CheckMenuItem,
    github_dark: CheckMenuItem,
    docc_dark: CheckMenuItem,
    basic_dark: CheckMenuItem,
}

pub fn build_menu(settings: &StoredSettings) -> Menu {
    let (menu, bindings) = MenuBindings::new(settings);
    MENU_BINDINGS.with(|cell| {
        *cell.borrow_mut() = Some(bindings);
    });
    sync_recent_menu(&settings.recent_files);
    sync_auto_reload(settings.auto_reload);
    sync_theme(settings.selected_theme);
    menu
}

pub fn sync_recent_menu(recent: &[RecentFileRecord]) {
    with_bindings(|bindings| bindings.sync_recent(recent));
}

pub fn sync_auto_reload(enabled: bool) {
    with_bindings(|bindings| bindings.auto_reload_item.set_checked(enabled));
}

pub fn sync_theme(theme: ReaderTheme) {
    with_bindings(|bindings| bindings.sync_theme(theme));
}

pub fn theme_from_id(id: &str) -> Option<ReaderTheme> {
    match id {
        MENU_THEME_GITHUB => Some(ReaderTheme::GitHub),
        MENU_THEME_DOCC => Some(ReaderTheme::DocC),
        MENU_THEME_BASIC => Some(ReaderTheme::Basic),
        MENU_THEME_GITHUB_DARK => Some(ReaderTheme::GitHubDark),
        MENU_THEME_DOCC_DARK => Some(ReaderTheme::DocCDark),
        MENU_THEME_BASIC_DARK => Some(ReaderTheme::BasicDark),
        _ => None,
    }
}

pub fn recent_index_from_id(id: &str) -> Option<usize> {
    id.strip_prefix(MENU_RECENT_PREFIX)
        .and_then(|suffix| suffix.parse::<usize>().ok())
}

fn with_bindings(f: impl FnOnce(&MenuBindings)) {
    MENU_BINDINGS.with(|cell| {
        if let Some(bindings) = cell.borrow().as_ref() {
            f(bindings);
        }
    });
}

impl MenuBindings {
    fn new(settings: &StoredSettings) -> (Menu, Self) {
        let menu = Menu::new();

        let file_menu = Submenu::new("File", true);
        let open_item = MenuItem::with_id(
            MENU_OPEN,
            "Open...",
            true,
            Some(Accelerator::new(Some(CMD_OR_CTRL), Code::KeyO)),
        );
        let recent_menu = Submenu::new("Open Recent", true);

        let _ = file_menu.append_items(&[
            &open_item,
            &recent_menu,
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ]);

        let auto_reload_item = CheckMenuItem::with_id(
            MENU_AUTO_RELOAD,
            "Auto-Reload on Changes",
            true,
            settings.auto_reload,
            None,
        );
        let theme_menu = Submenu::new("Theme", true);
        let theme_items = ThemeItems::new(settings.selected_theme);
        let _ = theme_menu.append_items(&[
            &theme_items.github,
            &theme_items.docc,
            &theme_items.basic,
            &PredefinedMenuItem::separator(),
            &theme_items.github_dark,
            &theme_items.docc_dark,
            &theme_items.basic_dark,
        ]);

        let reader_menu = Submenu::new("Reader", true);
        let _ = reader_menu.append_items(&[
            &auto_reload_item,
            &PredefinedMenuItem::separator(),
            &theme_menu,
        ]);

        let edit_menu = Submenu::new("Edit", true);
        let _ = edit_menu.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ]);

        let window_menu = Submenu::new("Window", true);
        let _ = window_menu.append_items(&[
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(None),
        ]);

        #[cfg(target_os = "macos")]
        {
            window_menu.set_as_windows_menu_for_nsapp();
        }

        let _ = menu.append_items(&[&file_menu, &reader_menu, &edit_menu, &window_menu]);

        let bindings = MenuBindings {
            recent_menu,
            auto_reload_item,
            theme_items,
        };

        (menu, bindings)
    }

    fn sync_theme(&self, theme: ReaderTheme) {
        self.theme_items.github.set_checked(theme == ReaderTheme::GitHub);
        self.theme_items.docc.set_checked(theme == ReaderTheme::DocC);
        self.theme_items.basic.set_checked(theme == ReaderTheme::Basic);
        self.theme_items
            .github_dark
            .set_checked(theme == ReaderTheme::GitHubDark);
        self.theme_items
            .docc_dark
            .set_checked(theme == ReaderTheme::DocCDark);
        self.theme_items
            .basic_dark
            .set_checked(theme == ReaderTheme::BasicDark);
    }

    fn sync_recent(&self, recent: &[RecentFileRecord]) {
        while self.recent_menu.remove_at(0).is_some() {}

        if recent.is_empty() {
            self.recent_menu.set_enabled(false);
            let empty_item = MenuItem::with_id(MENU_RECENT_EMPTY, "No Recent Files", false, None);
            let _ = self.recent_menu.append(&empty_item);
            return;
        }

        self.recent_menu.set_enabled(true);
        for (index, record) in recent.iter().enumerate() {
            let item = MenuItem::with_id(
                format!("{MENU_RECENT_PREFIX}{index}"),
                record.display_name.clone(),
                true,
                None,
            );
            let _ = self.recent_menu.append(&item);
        }

        let _ = self.recent_menu.append(&PredefinedMenuItem::separator());
        let clear_item = MenuItem::with_id(MENU_CLEAR_RECENT, "Clear Menu", true, None);
        let _ = self.recent_menu.append(&clear_item);
    }
}

impl ThemeItems {
    fn new(selected: ReaderTheme) -> Self {
        let github = CheckMenuItem::with_id(
            MENU_THEME_GITHUB,
            "GitHub",
            true,
            selected == ReaderTheme::GitHub,
            None,
        );
        let docc = CheckMenuItem::with_id(
            MENU_THEME_DOCC,
            "DocC",
            true,
            selected == ReaderTheme::DocC,
            None,
        );
        let basic = CheckMenuItem::with_id(
            MENU_THEME_BASIC,
            "Basic",
            true,
            selected == ReaderTheme::Basic,
            None,
        );
        let github_dark = CheckMenuItem::with_id(
            MENU_THEME_GITHUB_DARK,
            "GitHub Dark",
            true,
            selected == ReaderTheme::GitHubDark,
            None,
        );
        let docc_dark = CheckMenuItem::with_id(
            MENU_THEME_DOCC_DARK,
            "DocC Dark",
            true,
            selected == ReaderTheme::DocCDark,
            None,
        );
        let basic_dark = CheckMenuItem::with_id(
            MENU_THEME_BASIC_DARK,
            "Basic Dark",
            true,
            selected == ReaderTheme::BasicDark,
            None,
        );
        Self {
            github,
            docc,
            basic,
            github_dark,
            docc_dark,
            basic_dark,
        }
    }
}
