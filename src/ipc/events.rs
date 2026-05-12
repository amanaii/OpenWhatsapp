//! App event definitions.

use std::path::PathBuf;

use uuid::Uuid;

use crate::config::Theme;

/// Cross-module application event.
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
#[non_exhaustive]
pub(crate) enum AppEvent {
    /// App bootstrap finished.
    AppReady,
    /// Show main window.
    WindowShow,
    /// Hide main window.
    WindowHide,
    /// Fullscreen state changed.
    FullscreenToggled(bool),
    /// Account was added.
    AccountAdded(Uuid),
    /// Account was removed.
    AccountRemoved(Uuid),
    /// Active account changed.
    AccountSwitched(Uuid),
    /// Account unread count changed.
    UnreadCountChanged {
        /// Account identifier.
        account_id: Uuid,
        /// Unread message count.
        count: u32,
    },
    /// Desktop notification was received.
    NotificationReceived {
        /// Account identifier.
        account_id: Uuid,
        /// Notification title.
        title: String,
        /// Notification body.
        body: String,
    },
    /// Customizations were reloaded.
    CustomizationReloaded {
        /// Optional account identifier; `None` means global.
        account_id: Option<Uuid>,
    },
    /// Download started.
    DownloadStarted {
        /// Account identifier.
        account_id: Uuid,
        /// Download destination path.
        path: PathBuf,
    },
    /// Theme preference changed.
    ThemeChanged(Theme),
    /// Config was saved.
    ConfigSaved,
    /// Settings dialog should open.
    SettingsOpen,
    /// New account flow was requested.
    NewAccountRequested,
    /// Switch to next account.
    NextAccount,
    /// Switch to previous account.
    PreviousAccount,
    /// Reload active webview.
    ReloadActiveWebView,
    /// Open active WebKit inspector.
    OpenInspector,
    /// App should quit.
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_event_clones_with_payload() {
        let account_id = Uuid::new_v4();
        let event = AppEvent::UnreadCountChanged {
            account_id,
            count: 3,
        };

        assert_eq!(event.clone(), event);
    }
}
