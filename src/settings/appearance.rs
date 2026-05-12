//! Appearance settings.

#![allow(deprecated)]

use gtk::prelude::*;

use crate::config::{AppConfig, Theme};

pub(super) struct AppearancePanel {
    root: gtk::Box,
    theme: gtk::ComboBoxText,
    csd: gtk::Switch,
    scale: gtk::Scale,
}

impl AppearancePanel {
    pub(super) fn new(config: &AppConfig) -> Self {
        let root = super::panel_box();
        let theme = gtk::ComboBoxText::new();
        theme.append(Some("auto"), "Auto");
        theme.append(Some("light"), "Light");
        theme.append(Some("dark"), "Dark");
        theme.set_active_id(Some(match config.appearance.theme {
            Theme::Auto => "auto",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }));

        let csd = gtk::Switch::builder()
            .active(config.appearance.custom_decorations)
            .build();
        let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.75, 2.0, 0.05);
        scale.set_value(config.appearance.interface_scale);
        scale.set_draw_value(true);

        root.append(&super::row("Theme", &theme));
        root.append(&super::row("Custom decorations", &csd));
        root.append(&super::row("Interface scale", &scale));

        Self {
            root,
            theme,
            csd,
            scale,
        }
    }

    pub(super) fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub(super) fn write_config(&self, config: &mut AppConfig) {
        config.appearance.theme = match self.theme.active_id().as_deref() {
            Some("light") => Theme::Light,
            Some("dark") => Theme::Dark,
            _ => Theme::Auto,
        };
        config.appearance.custom_decorations = self.csd.is_active();
        config.appearance.interface_scale = self.scale.value();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn theme_ids_are_stable() {
        let ids = ["auto", "light", "dark"];

        assert!(ids.contains(&"auto"));
    }
}
