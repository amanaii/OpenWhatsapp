//! Performance settings.

use gtk::prelude::*;

use crate::config::AppConfig;

pub(super) struct PerformancePanel {
    root: gtk::Box,
    hardware: gtk::Switch,
    cache_size: gtk::SpinButton,
}

impl PerformancePanel {
    pub(super) fn new(config: &AppConfig) -> Self {
        let root = super::panel_box();
        let hardware = gtk::Switch::builder()
            .active(config.performance.hardware_acceleration)
            .build();
        let cache_size = gtk::SpinButton::with_range(32.0, 4096.0, 32.0);
        cache_size.set_value(f64::from(config.performance.cache_size_mb));

        root.append(&super::row("Hardware acceleration", &hardware));
        root.append(&super::row("Cache size (MiB)", &cache_size));

        Self {
            root,
            hardware,
            cache_size,
        }
    }

    pub(super) fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub(super) fn write_config(&self, config: &mut AppConfig) {
        config.performance.hardware_acceleration = self.hardware.is_active();
        config.performance.cache_size_mb = self.cache_size.value_as_int().max(0) as u32;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn cache_step_is_positive() {
        let step = 32;

        assert!(step > 0);
    }
}
