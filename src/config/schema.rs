//! Versioned config schema.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Current config schema version.
pub(crate) const CURRENT_CONFIG_VERSION: u32 = 1;

/// App theme preference.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Theme {
    /// Follow system theme.
    #[default]
    Auto,
    /// Force light theme.
    Light,
    /// Force dark theme.
    Dark,
}

/// Root application configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    /// Schema version used for migrations.
    pub(crate) version: u32,
    /// Window size and position.
    pub(crate) window: WindowConfig,
    /// Appearance settings.
    pub(crate) appearance: AppearanceConfig,
    /// Tray and workflow settings.
    pub(crate) usability: UsabilityConfig,
    /// Spellcheck settings.
    pub(crate) spellcheck: SpellcheckConfig,
    /// Download settings.
    pub(crate) downloads: DownloadConfig,
    /// Performance settings.
    pub(crate) performance: PerformanceConfig,
}

impl AppConfig {
    /// Migrates this config to the current schema version.
    pub(crate) fn migrate(mut self) -> Self {
        if self.version < CURRENT_CONFIG_VERSION {
            self.version = CURRENT_CONFIG_VERSION;
        }

        self
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CURRENT_CONFIG_VERSION,
            window: WindowConfig::default(),
            appearance: AppearanceConfig::default(),
            usability: UsabilityConfig::default(),
            spellcheck: SpellcheckConfig::default(),
            downloads: DownloadConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Window geometry configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct WindowConfig {
    /// Window width in physical pixels.
    pub(crate) width: i32,
    /// Window height in physical pixels.
    pub(crate) height: i32,
    /// Last saved window x coordinate.
    pub(crate) x: Option<i32>,
    /// Last saved window y coordinate.
    pub(crate) y: Option<i32>,
    /// Whether the window starts maximized.
    pub(crate) maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 800,
            x: None,
            y: None,
            maximized: false,
        }
    }
}

/// Appearance configuration.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct AppearanceConfig {
    /// Theme preference.
    pub(crate) theme: Theme,
    /// Whether client-side decorations are enabled.
    pub(crate) custom_decorations: bool,
    /// Interface scale multiplier.
    pub(crate) interface_scale: f64,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: Theme::Auto,
            custom_decorations: true,
            interface_scale: 1.0,
        }
    }
}

/// Usability configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct UsabilityConfig {
    /// Whether tray integration is enabled.
    pub(crate) tray_enabled: bool,
    /// Whether closing the window keeps the app in background.
    pub(crate) background_process: bool,
    /// Optional custom tray icon path.
    pub(crate) tray_icon_path: Option<PathBuf>,
    /// Whether native file dialogs are disabled.
    pub(crate) disable_native_file_dialog: bool,
}

impl Default for UsabilityConfig {
    fn default() -> Self {
        Self {
            tray_enabled: true,
            background_process: true,
            tray_icon_path: None,
            disable_native_file_dialog: false,
        }
    }
}

/// Spellcheck configuration.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct SpellcheckConfig {
    /// Whether spellcheck is enabled.
    pub(crate) enabled: bool,
    /// Selected spellcheck language tags.
    pub(crate) languages: Vec<String>,
    /// Optional custom dictionary folder.
    pub(crate) custom_dictionary_dir: Option<PathBuf>,
}

/// Download configuration.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct DownloadConfig {
    /// Optional custom download directory.
    pub(crate) directory: Option<PathBuf>,
    /// Whether native file dialogs are disabled for downloads.
    pub(crate) disable_native_file_dialog: bool,
}

/// Performance configuration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(default)]
pub(crate) struct PerformanceConfig {
    /// Whether hardware acceleration is enabled.
    pub(crate) hardware_acceleration: bool,
    /// Cache size limit in MiB.
    pub(crate) cache_size_mb: u32,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            hardware_acceleration: true,
            cache_size_mb: 256,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_current_version() {
        assert_eq!(AppConfig::default().version, CURRENT_CONFIG_VERSION);
    }

    #[test]
    fn old_config_migrates_to_current_version() {
        let config = AppConfig {
            version: 0,
            ..AppConfig::default()
        };

        assert_eq!(config.migrate().version, CURRENT_CONFIG_VERSION);
    }
}
