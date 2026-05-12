//! Window shell.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use gtk::prelude::*;

use crate::config::{self, AppConfig};
use crate::ipc::EventBus;

pub(crate) mod decorations;
pub(crate) mod scaling;

/// Applies initial window configuration and event handlers.
pub(crate) fn configure(
    window: &gtk::ApplicationWindow,
    config: &AppConfig,
    _events: EventBus,
) -> Result<()> {
    decorations::apply_custom_decorations(window, config.appearance.custom_decorations);
    scaling::apply_interface_scale(window, config.appearance.interface_scale)?;
    restore_geometry(window, config);

    Ok(())
}

/// Installs geometry saving when the window closes.
pub(crate) fn install_geometry_persistence(
    window: &gtk::ApplicationWindow,
    config_path: PathBuf,
    shared_config: Arc<Mutex<AppConfig>>,
) {
    let window_weak = window.downgrade();

    window.connect_close_request(move |_| {
        let Some(window) = window_weak.upgrade() else {
            return gtk::glib::Propagation::Proceed;
        };

        if let Ok(mut config) = shared_config.lock() {
            capture_geometry(&window, &mut config);

            if let Err(error) = config::save(&config_path, &config) {
                tracing::error!(?error, "failed to save window geometry");
            }
        } else {
            tracing::error!("config lock poisoned while saving window geometry");
        }

        gtk::glib::Propagation::Proceed
    });
}

fn restore_geometry(window: &gtk::ApplicationWindow, config: &AppConfig) {
    window.set_default_size(config.window.width, config.window.height);

    if config.window.maximized {
        window.maximize();
    }
}

fn capture_geometry(window: &gtk::ApplicationWindow, config: &mut AppConfig) {
    let width = window.width();
    let height = window.height();

    if width > 0 {
        config.window.width = width;
    }

    if height > 0 {
        config.window.height = height;
    }

    config.window.maximized = window.is_maximized();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_accepts_default_config_values() {
        let config = AppConfig::default();

        assert!(config.window.width > 0);
        assert!(config.window.height > 0);
    }
}
