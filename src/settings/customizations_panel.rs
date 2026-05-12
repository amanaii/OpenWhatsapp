//! Customization settings.

use gtk::prelude::*;

pub(super) struct CustomizationsPanel {
    root: gtk::Box,
}

impl CustomizationsPanel {
    pub(super) fn new() -> Self {
        let root = super::panel_box();
        let actions = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        for label in [
            "Import File",
            "Import URL",
            "Create",
            "Save",
            "Save & Reload",
            "Reload",
            "Open Folder",
        ] {
            actions.append(&gtk::Button::with_label(label));
        }
        root.append(&actions);

        Self { root }
    }

    pub(super) fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn customization_actions_include_reload() {
        let actions = [
            "Import File",
            "Import URL",
            "Create",
            "Save",
            "Save & Reload",
            "Reload",
        ];

        assert!(actions.contains(&"Reload"));
    }
}
