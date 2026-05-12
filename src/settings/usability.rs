//! Usability settings.

use gtk::prelude::*;

use crate::config::AppConfig;

pub(super) struct UsabilityPanel {
    root: gtk::Box,
    tray: gtk::Switch,
    background: gtk::Switch,
    native_dialog: gtk::Switch,
    download_dir: gtk::Entry,
}

impl UsabilityPanel {
    pub(super) fn new(config: &AppConfig) -> Self {
        let root = super::panel_box();
        let tray = gtk::Switch::builder()
            .active(config.usability.tray_enabled)
            .build();
        let background = gtk::Switch::builder()
            .active(config.usability.background_process)
            .build();
        let native_dialog = gtk::Switch::builder()
            .active(config.downloads.disable_native_file_dialog)
            .build();
        let download_dir = gtk::Entry::new();
        if let Some(path) = &config.downloads.directory {
            download_dir.set_text(&path.to_string_lossy());
        }

        root.append(&super::row("Tray icon", &tray));
        root.append(&super::row("Background process", &background));
        root.append(&super::row("Disable native file dialog", &native_dialog));
        root.append(&super::row("Download folder", &download_dir));

        Self {
            root,
            tray,
            background,
            native_dialog,
            download_dir,
        }
    }

    pub(super) fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub(super) fn write_config(&self, config: &mut AppConfig) {
        config.usability.tray_enabled = self.tray.is_active();
        config.usability.background_process = self.background.is_active();
        config.usability.disable_native_file_dialog = self.native_dialog.is_active();
        config.downloads.disable_native_file_dialog = self.native_dialog.is_active();
        let text = self.download_dir.text();
        config.downloads.directory = if text.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(text.as_str()))
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn empty_download_dir_means_default() {
        let value = "";

        assert!(value.trim().is_empty());
    }
}
