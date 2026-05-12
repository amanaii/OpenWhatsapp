//! Account grid UI.

use gtk::prelude::*;

use crate::ipc::{AppEvent, EventBus};

use super::profile::Account;

const GRID_WIDTH: i32 = 240;
const AVATAR_SIZE: i32 = 42;
const MAX_BADGE_COUNT: u32 = 999;

/// FlowBox-backed account grid.
pub(crate) struct AccountGrid {
    root: gtk::ScrolledWindow,
}

impl AccountGrid {
    /// Builds an account grid that emits account switch events on click.
    pub(crate) fn new(accounts: &[Account], events: EventBus) -> Self {
        let flow = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .max_children_per_line(1)
            .min_children_per_line(1)
            .row_spacing(8)
            .column_spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        for account in accounts {
            flow.append(&account_button(account, events.clone()));
        }

        let root = gtk::ScrolledWindow::builder()
            .child(&flow)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_width(GRID_WIDTH)
            .max_content_width(GRID_WIDTH)
            .vexpand(true)
            .build();

        Self { root }
    }

    /// Returns the root GTK widget for the grid.
    pub(crate) fn widget(&self) -> &gtk::ScrolledWindow {
        &self.root
    }
}

fn account_button(account: &Account, events: EventBus) -> gtk::Button {
    let button = gtk::Button::builder().hexpand(true).build();
    button.set_child(Some(&account_card(account, 0)));

    let account_id = account.id;
    button.connect_clicked(move |_| {
        let _subscriber_count = events.emit(AppEvent::AccountSwitched(account_id));
    });

    button
}

fn account_card(account: &Account, unread_count: u32) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    card.set_hexpand(true);
    card.set_margin_top(6);
    card.set_margin_bottom(6);
    card.set_margin_start(6);
    card.set_margin_end(6);

    card.append(&avatar_widget(account));
    card.append(&name_label(&account.display_name));
    card.append(&unread_badge(unread_count));

    card
}

fn avatar_widget(account: &Account) -> gtk::Widget {
    if let Some(path) = &account.avatar_path {
        if path.exists() {
            let picture = gtk::Picture::for_filename(path);
            picture.set_size_request(AVATAR_SIZE, AVATAR_SIZE);
            return picture.upcast();
        }
    }

    let fallback = gtk::Label::new(Some(&initial_for_name(&account.display_name)));
    fallback.set_size_request(AVATAR_SIZE, AVATAR_SIZE);
    fallback.add_css_class("account-avatar-fallback");
    fallback.upcast()
}

fn name_label(display_name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(display_name));
    label.set_hexpand(true);
    label.set_xalign(0.0);
    label.set_ellipsize(gtk::pango::EllipsizeMode::End);
    label
}

fn unread_badge(count: u32) -> gtk::Label {
    let label = gtk::Label::new(Some(&badge_text(count)));
    label.add_css_class("account-unread-badge");
    label.set_visible(count > 0);
    label
}

fn initial_for_name(display_name: &str) -> String {
    display_name
        .trim()
        .chars()
        .next()
        .map(|character| character.to_uppercase().collect())
        .unwrap_or_else(|| "?".to_string())
}

fn badge_text(count: u32) -> String {
    match count {
        0 => String::new(),
        1..=MAX_BADGE_COUNT => count.to_string(),
        _ => format!("{MAX_BADGE_COUNT}+"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_uses_first_display_name_character() {
        assert_eq!(initial_for_name(" work"), "W");
        assert_eq!(initial_for_name(""), "?");
    }

    #[test]
    fn badge_text_caps_large_counts() {
        assert_eq!(badge_text(0), "");
        assert_eq!(badge_text(9), "9");
        assert_eq!(badge_text(1_000), "999+");
    }
}
