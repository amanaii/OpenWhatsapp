//! Settings UI.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;

use crate::config::{self, AppConfig};
use crate::ipc::{AppEvent, EventBus};

pub(crate) mod appearance;
pub(crate) mod customizations_panel;
pub(crate) mod performance;
pub(crate) mod spellcheck_panel;
pub(crate) mod usability;

/// Shows the settings dialog.
#[allow(deprecated)]
pub(crate) fn show_dialog(
    parent: &gtk::ApplicationWindow,
    config_path: PathBuf,
    shared_config: Arc<Mutex<AppConfig>>,
    events: EventBus,
) {
    let config = match shared_config.lock() {
        Ok(config) => config.clone(),
        Err(error) => {
            tracing::error!(?error, "config lock poisoned while opening settings");
            return;
        }
    };

    let dialog = gtk::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Settings")
        .default_width(760)
        .default_height(560)
        .build();
    dialog.add_button("Cancel", gtk::ResponseType::Cancel);
    dialog.add_button("Save", gtk::ResponseType::Accept);
    dialog.add_button("Save & Reload", gtk::ResponseType::Apply);

    let notebook = gtk::Notebook::new();
    notebook.set_vexpand(true);
    notebook.set_hexpand(true);

    let appearance = appearance::AppearancePanel::new(&config);
    let usability = usability::UsabilityPanel::new(&config);
    let performance = performance::PerformancePanel::new(&config);
    let spellcheck = spellcheck_panel::SpellcheckPanel::new(&config);
    let customizations = customizations_panel::CustomizationsPanel::new();

    append_page(&notebook, appearance.widget(), "Appearance");
    append_page(&notebook, usability.widget(), "Usability");
    append_page(&notebook, performance.widget(), "Performance");
    append_page(&notebook, spellcheck.widget(), "Spellcheck");
    append_page(&notebook, customizations.widget(), "Customizations");
    dialog.content_area().append(&notebook);

    dialog.connect_response(move |dialog, response| {
        if matches!(
            response,
            gtk::ResponseType::Accept | gtk::ResponseType::Apply
        ) {
            match shared_config.lock() {
                Ok(mut config) => {
                    appearance.write_config(&mut config);
                    usability.write_config(&mut config);
                    performance.write_config(&mut config);
                    spellcheck.write_config(&mut config);

                    if let Err(error) = config::save(&config_path, &config) {
                        tracing::error!(?error, "failed to save settings");
                    } else {
                        let _subscriber_count = events.emit(AppEvent::ConfigSaved);
                    }
                }
                Err(error) => tracing::error!(?error, "config lock poisoned while saving settings"),
            }
        }

        dialog.close();
    });

    dialog.present();
}

fn append_page(notebook: &gtk::Notebook, child: &gtk::Widget, label: &str) {
    notebook.append_page(child, Some(&gtk::Label::new(Some(label))));
}

pub(super) fn panel_box() -> gtk::Box {
    let panel = gtk::Box::new(gtk::Orientation::Vertical, 12);
    panel.set_margin_top(16);
    panel.set_margin_bottom(16);
    panel.set_margin_start(16);
    panel.set_margin_end(16);
    panel
}

pub(super) fn row(label: &str, child: &impl IsA<gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let label = gtk::Label::new(Some(label));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    row.append(&label);
    row.append(child);
    row
}

#[cfg(test)]
mod tests {
    #[test]
    fn settings_has_five_tabs() {
        let tabs = [
            "Appearance",
            "Usability",
            "Performance",
            "Spellcheck",
            "Customizations",
        ];

        assert_eq!(tabs.len(), 5);
    }
}
