//! Tray menu construction.

use gtk3::prelude::*;

use crate::accounts::Account;
use crate::ipc::{AppEvent, EventBus};

/// Builds the tray context menu.
pub(crate) fn build(accounts: &[Account], events: EventBus) -> gtk3::Menu {
    let menu = gtk3::Menu::new();

    menu.append(&event_item("Show", events.clone(), AppEvent::WindowShow));
    menu.append(&event_item("Hide", events.clone(), AppEvent::WindowHide));
    menu.append(&gtk3::SeparatorMenuItem::new());
    menu.append(&accounts_item(accounts, events.clone()));
    menu.append(&gtk3::SeparatorMenuItem::new());
    menu.append(&event_item("Quit", events, AppEvent::Quit));

    menu
}

fn event_item(label: &str, events: EventBus, event: AppEvent) -> gtk3::MenuItem {
    let item = gtk3::MenuItem::with_label(label);
    item.connect_activate(move |_| {
        let _subscriber_count = events.emit(event.clone());
    });

    item
}

fn accounts_item(accounts: &[Account], events: EventBus) -> gtk3::MenuItem {
    let item = gtk3::MenuItem::with_label("Accounts");
    let submenu = gtk3::Menu::new();

    if accounts.is_empty() {
        let empty = gtk3::MenuItem::with_label(empty_accounts_label());
        empty.set_sensitive(false);
        submenu.append(&empty);
    } else {
        for account in accounts {
            let account_id = account.id;
            let account_item = gtk3::MenuItem::with_label(&account.display_name);
            let events = events.clone();
            account_item.connect_activate(move |_| {
                let _subscriber_count = events.emit(AppEvent::AccountSwitched(account_id));
            });
            submenu.append(&account_item);
        }
    }

    item.set_submenu(Some(&submenu));
    item
}

fn empty_accounts_label() -> &'static str {
    "No accounts"
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_accounts_menu_label_is_stable() {
        assert_eq!(super::empty_accounts_label(), "No accounts");
    }
}
