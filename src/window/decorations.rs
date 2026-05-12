//! Window decoration helpers.

use gtk::prelude::*;

pub(super) fn apply_custom_decorations(window: &gtk::ApplicationWindow, enabled: bool) {
    if enabled {
        let header = gtk::HeaderBar::new();
        header.set_show_title_buttons(true);
        window.set_titlebar(Some(&header));
    } else {
        window.set_titlebar(Option::<&gtk::Widget>::None);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn custom_decorations_default_to_enabled_in_config() {
        assert!(
            crate::config::AppConfig::default()
                .appearance
                .custom_decorations
        );
    }
}
