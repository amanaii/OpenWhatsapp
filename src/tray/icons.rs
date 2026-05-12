//! Tray icon selection.

use std::path::{Path, PathBuf};

use libappindicator::{AppIndicator, AppIndicatorStatus};

/// Default icon name looked up in the icon theme.
pub(crate) const DEFAULT_ICON_NAME: &str = "openwhatsapp";
/// Unread icon name looked up in the icon theme.
pub(crate) const UNREAD_ICON_NAME: &str = "openwhatsapp-unread";

/// Tray icon source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TrayIcon {
    icon_name: String,
    theme_path: Option<PathBuf>,
}

impl TrayIcon {
    /// Builds a tray icon source from an optional custom icon file.
    pub(crate) fn new(custom_icon_path: Option<&Path>) -> Self {
        match custom_icon_path.and_then(split_icon_file) {
            Some((theme_path, icon_name)) => Self {
                icon_name,
                theme_path: Some(theme_path),
            },
            None => Self {
                icon_name: DEFAULT_ICON_NAME.to_string(),
                theme_path: None,
            },
        }
    }

    /// Creates an AppIndicator using this icon source.
    pub(crate) fn create_indicator(&self, title: &str) -> AppIndicator {
        match self.theme_path.as_ref() {
            Some(theme_path) => {
                let theme_path = theme_path.to_string_lossy();
                AppIndicator::with_path(title, &self.icon_name, theme_path.as_ref())
            }
            None => AppIndicator::new(title, &self.icon_name),
        }
    }
}

/// Applies unread icon state to an indicator.
pub(crate) fn apply_unread_state(indicator: &mut AppIndicator, unread_count: u32) {
    if unread_count == 0 {
        indicator.set_icon_full(DEFAULT_ICON_NAME, "OpenWhatsapp");
        indicator.set_label("", "");
        indicator.set_status(AppIndicatorStatus::Active);
    } else {
        indicator.set_icon_full(UNREAD_ICON_NAME, "OpenWhatsapp unread");
        indicator.set_label(&badge_text(unread_count), "999+");
        indicator.set_status(AppIndicatorStatus::Attention);
    }
}

fn split_icon_file(path: &Path) -> Option<(PathBuf, String)> {
    let theme_path = path.parent()?.to_path_buf();
    let icon_name = path.file_stem()?.to_string_lossy().into_owned();

    Some((theme_path, icon_name))
}

fn badge_text(unread_count: u32) -> String {
    match unread_count {
        0 => String::new(),
        1..=999 => unread_count.to_string(),
        _ => "999+".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_icon_path_splits_theme_dir_and_name() {
        let icon = TrayIcon::new(Some(Path::new("/tmp/icons/openwhatsapp.png")));

        assert_eq!(icon.icon_name, "openwhatsapp");
        assert_eq!(icon.theme_path, Some(PathBuf::from("/tmp/icons")));
    }

    #[test]
    fn badge_text_caps_large_counts() {
        assert_eq!(badge_text(0), "");
        assert_eq!(badge_text(7), "7");
        assert_eq!(badge_text(1_000), "999+");
    }
}
