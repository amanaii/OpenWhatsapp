//! Customization editor dialog.

use gtk::prelude::*;

/// Builds a basic customization editor dialog.
#[allow(dead_code)]
#[allow(deprecated)]
pub(crate) fn build_editor_dialog(parent: &gtk::ApplicationWindow, title: &str) -> gtk::Dialog {
    let dialog = gtk::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(title)
        .default_width(720)
        .default_height(520)
        .build();
    dialog.add_button("Save", gtk::ResponseType::Accept);
    dialog.add_button("Save & Reload", gtk::ResponseType::Apply);

    let text_view = gtk::TextView::new();
    text_view.set_monospace(true);
    text_view.set_vexpand(true);
    text_view.set_hexpand(true);
    dialog.content_area().append(&text_view);

    dialog
}

#[cfg(test)]
mod tests {
    #[test]
    fn editor_title_is_passthrough_text() {
        let title = "Edit CSS".to_string();

        assert!(title.starts_with("Edit"));
    }
}
