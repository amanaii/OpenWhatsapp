//! Desktop notification integration.

use gtk::gio::prelude::*;
use gtk::glib::variant::{StaticVariantType, ToVariant};
use uuid::Uuid;

use crate::ipc::{AppEvent, EventBus};

const APP_ICON_PATH: &str = "assets/icons/openwhatsapp.png";
const OPEN_NOTIFICATION_ACTION: &str = "open-notification";
const DETAILED_OPEN_NOTIFICATION_ACTION: &str = "app.open-notification";

/// Installs notification actions on the GTK application.
pub(crate) fn install_actions(application: &gtk::Application, events: EventBus) {
    if application
        .lookup_action(OPEN_NOTIFICATION_ACTION)
        .is_some()
    {
        return;
    }

    let parameter_type = String::static_variant_type();
    let action =
        gtk::gio::SimpleAction::new(OPEN_NOTIFICATION_ACTION, Some(parameter_type.as_ref()));
    action.connect_activate(move |_, parameter| {
        let Some(account_id) = parameter
            .and_then(|parameter| parameter.get::<String>())
            .and_then(|account_id| Uuid::parse_str(&account_id).ok())
        else {
            return;
        };

        let _subscriber_count = events.emit(AppEvent::WindowShow);
        let _subscriber_count = events.emit(AppEvent::AccountSwitched(account_id));
    });

    application.add_action(&action);
}

/// Sends a portal-compatible desktop notification.
pub(crate) fn send(application: &gtk::Application, account_id: Uuid, title: &str, body: &str) {
    let notification = gtk::gio::Notification::new(title);
    notification.set_body(Some(body));
    notification.set_icon(&notification_icon());
    notification.set_priority(gtk::gio::NotificationPriority::Normal);
    notification.set_default_action_and_target_value(
        DETAILED_OPEN_NOTIFICATION_ACTION,
        Some(&account_id.to_string().to_variant()),
    );

    application.send_notification(
        Some(&notification_id(account_id, title, body)),
        &notification,
    );
}

fn notification_icon() -> gtk::gio::FileIcon {
    let file = gtk::gio::File::for_path(APP_ICON_PATH);
    gtk::gio::FileIcon::new(&file)
}

fn notification_id(account_id: Uuid, title: &str, body: &str) -> String {
    format!(
        "openwhatsapp-{account_id}-{}",
        stable_text_hash(title, body)
    )
}

fn stable_text_hash(title: &str, body: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in title.bytes().chain([0]).chain(body.bytes()) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_id_contains_account_id() {
        let account_id = Uuid::new_v4();

        assert_eq!(
            notification_id(account_id, "A", "B"),
            format!("openwhatsapp-{account_id}-{}", stable_text_hash("A", "B"))
        );
    }
}
