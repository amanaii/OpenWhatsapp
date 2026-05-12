//! Keyboard shortcut bindings.

use gtk::prelude::*;
use uuid::Uuid;

use crate::accounts::Account;
use crate::ipc::{AppEvent, EventBus};

/// Installs global keyboard shortcuts on the main window.
pub(crate) fn install(window: &gtk::ApplicationWindow, accounts: &[Account], events: EventBus) {
    let controller = gtk::EventControllerKey::new();
    let window_weak = window.downgrade();
    let account_ids = account_ids_by_index(accounts);

    controller.connect_key_pressed(move |_, key, _, state| {
        let Some(action) = shortcut_action(key, state, &account_ids) else {
            return gtk::glib::Propagation::Proceed;
        };

        if let ShortcutAction::ToggleFullscreen = action {
            if let Some(window) = window_weak.upgrade() {
                let fullscreen = toggle_fullscreen(&window);
                let _subscriber_count = events.emit(AppEvent::FullscreenToggled(fullscreen));
            }

            return gtk::glib::Propagation::Stop;
        }

        if let Some(event) = action.into_event() {
            let _subscriber_count = events.emit(event);
        }

        gtk::glib::Propagation::Stop
    });

    window.add_controller(controller);
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ShortcutAction {
    ToggleFullscreen,
    OpenSettings,
    NewAccount,
    NextAccount,
    PreviousAccount,
    JumpToAccount(Uuid),
    ReloadActiveWebView,
    OpenInspector,
    Quit,
}

impl ShortcutAction {
    fn into_event(self) -> Option<AppEvent> {
        match self {
            Self::ToggleFullscreen => None,
            Self::OpenSettings => Some(AppEvent::SettingsOpen),
            Self::NewAccount => Some(AppEvent::NewAccountRequested),
            Self::NextAccount => Some(AppEvent::NextAccount),
            Self::PreviousAccount => Some(AppEvent::PreviousAccount),
            Self::JumpToAccount(account_id) => Some(AppEvent::AccountSwitched(account_id)),
            Self::ReloadActiveWebView => Some(AppEvent::ReloadActiveWebView),
            Self::OpenInspector => Some(AppEvent::OpenInspector),
            Self::Quit => Some(AppEvent::Quit),
        }
    }
}

fn shortcut_action(
    key: gtk::gdk::Key,
    state: gtk::gdk::ModifierType,
    account_ids: &[Uuid],
) -> Option<ShortcutAction> {
    let ctrl = state.contains(gtk::gdk::ModifierType::CONTROL_MASK);
    let shift = state.contains(gtk::gdk::ModifierType::SHIFT_MASK);

    if !ctrl && !shift && key == gtk::gdk::Key::F11 {
        return Some(ShortcutAction::ToggleFullscreen);
    }

    if !ctrl {
        return None;
    }

    if is_tab_key(key) {
        return if shift {
            Some(ShortcutAction::PreviousAccount)
        } else {
            Some(ShortcutAction::NextAccount)
        };
    }

    match normalized_key_char(key) {
        Some(',') if !shift => Some(ShortcutAction::OpenSettings),
        Some('n') if !shift => Some(ShortcutAction::NewAccount),
        Some('q') if !shift => Some(ShortcutAction::Quit),
        Some('r') if shift => Some(ShortcutAction::ReloadActiveWebView),
        Some('i') if shift => Some(ShortcutAction::OpenInspector),
        Some(digit @ '1'..='9') if !shift => {
            account_id_for_digit(digit, account_ids).map(ShortcutAction::JumpToAccount)
        }
        _ => None,
    }
}

fn toggle_fullscreen(window: &gtk::ApplicationWindow) -> bool {
    if window.is_fullscreen() {
        window.unfullscreen();
        false
    } else {
        window.fullscreen();
        true
    }
}

fn is_tab_key(key: gtk::gdk::Key) -> bool {
    key == gtk::gdk::Key::Tab || key == gtk::gdk::Key::ISO_Left_Tab
}

fn normalized_key_char(key: gtk::gdk::Key) -> Option<char> {
    key.to_unicode()
        .map(|character| character.to_ascii_lowercase())
}

fn account_ids_by_index(accounts: &[Account]) -> Vec<Uuid> {
    accounts.iter().map(|account| account.id).collect()
}

fn account_id_for_digit(digit: char, account_ids: &[Uuid]) -> Option<Uuid> {
    digit
        .to_digit(10)
        .and_then(|digit| usize::try_from(digit).ok())
        .and_then(|digit| digit.checked_sub(1))
        .and_then(|index| account_ids.get(index).copied())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digit_shortcut_maps_to_account_index() {
        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        let accounts = vec![first, second];

        assert_eq!(account_id_for_digit('1', &accounts), Some(first));
        assert_eq!(account_id_for_digit('2', &accounts), Some(second));
        assert_eq!(account_id_for_digit('9', &accounts), None);
    }

    #[test]
    fn action_turns_into_expected_event() {
        assert_eq!(ShortcutAction::Quit.into_event(), Some(AppEvent::Quit));
        assert_eq!(
            ShortcutAction::ReloadActiveWebView.into_event(),
            Some(AppEvent::ReloadActiveWebView)
        );
    }
}
