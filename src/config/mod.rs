//! Config loading and migration.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

pub(crate) mod schema;

pub(crate) use schema::{AppConfig, DownloadConfig, SpellcheckConfig, Theme};

/// Loads config from disk, creating a default file when missing.
pub(crate) fn load_or_create(path: &Path) -> Result<AppConfig> {
    if path.exists() {
        load(path)
    } else {
        let config = AppConfig::default();
        save(path, &config)?;
        Ok(config)
    }
}

/// Loads config from a TOML file and migrates it to the current schema.
pub(crate) fn load(path: &Path) -> Result<AppConfig> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file {}", path.display()))?;
    let config = toml::from_str::<AppConfig>(&content)
        .with_context(|| format!("failed to parse config file {}", path.display()))?;

    Ok(config.migrate())
}

/// Saves config to a TOML file.
pub(crate) fn save(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create config dir {}", parent.display()))?;
    }

    let content = toml::to_string_pretty(config).context("failed to serialize config")?;
    fs::write(path, content)
        .with_context(|| format!("failed to write config file {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::AppearanceConfig;
    use uuid::Uuid;

    fn temp_config_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("openwhatsapp-test-{}.toml", Uuid::new_v4()))
    }

    #[test]
    fn config_round_trips_through_toml() {
        let path = temp_config_path();
        let config = AppConfig {
            appearance: AppearanceConfig {
                theme: Theme::Dark,
                custom_decorations: false,
                interface_scale: 1.5,
            },
            ..AppConfig::default()
        };

        save(&path, &config).unwrap();
        let loaded = load(&path).unwrap();

        assert_eq!(loaded, config);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn load_or_create_writes_default_config() {
        let path = temp_config_path();

        let config = load_or_create(&path).unwrap();

        assert_eq!(config, AppConfig::default());
        assert!(path.exists());
        std::fs::remove_file(path).unwrap();
    }
}
