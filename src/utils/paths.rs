//! Filesystem path helpers.

use std::path::PathBuf;

use anyhow::{Context, Result};
use uuid::Uuid;

const APP_DIR_NAME: &str = "openwhatsapp";

/// Returns the OpenWhatsapp config directory.
pub(crate) fn config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .map(|path| path.join(APP_DIR_NAME))
        .context("failed to locate XDG config directory")
}

/// Returns the OpenWhatsapp config file path.
pub(crate) fn config_file() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

/// Returns the OpenWhatsapp SQLite database file path.
pub(crate) fn database_file() -> Result<PathBuf> {
    Ok(data_dir()?.join("openwhatsapp.sqlite3"))
}

/// Returns the OpenWhatsapp data directory.
pub(crate) fn data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .map(|path| path.join(APP_DIR_NAME))
        .context("failed to locate XDG data directory")
}

/// Returns the OpenWhatsapp runtime directory.
pub(crate) fn runtime_dir() -> Result<PathBuf> {
    dirs::runtime_dir()
        .or_else(|| Some(std::env::temp_dir()))
        .map(|path| path.join(APP_DIR_NAME))
        .context("failed to locate runtime directory")
}

/// Returns the OpenWhatsapp temporary download directory.
pub(crate) fn temp_download_dir() -> Result<PathBuf> {
    Ok(runtime_dir()?.join("tmp"))
}

/// Returns the per-account WebKit data directory.
pub(crate) fn account_web_data_dir(account_id: Uuid) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("web")
        .join(account_id.to_string())
        .join("data"))
}

/// Returns the per-account WebKit cache directory.
pub(crate) fn account_web_cache_dir(account_id: Uuid) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("web")
        .join(account_id.to_string())
        .join("cache"))
}

/// Returns global customization CSS directory.
pub(crate) fn global_css_dir() -> Result<PathBuf> {
    Ok(data_dir()?
        .join("customizations")
        .join("global")
        .join("css"))
}

/// Returns global customization JS directory.
pub(crate) fn global_js_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("customizations").join("global").join("js"))
}

/// Returns account customization CSS directory.
#[allow(dead_code)]
pub(crate) fn account_css_dir(account_id: Uuid) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("customizations")
        .join("accounts")
        .join(account_id.to_string())
        .join("css"))
}

/// Returns account customization JS directory.
#[allow(dead_code)]
pub(crate) fn account_js_dir(account_id: Uuid) -> Result<PathBuf> {
    Ok(data_dir()?
        .join("customizations")
        .join("accounts")
        .join(account_id.to_string())
        .join("js"))
}

/// Returns reserved customization extensions directory.
pub(crate) fn extensions_dir() -> Result<PathBuf> {
    Ok(data_dir()?.join("customizations").join("extensions"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_paths_end_with_openwhatsapp() {
        assert_eq!(config_dir().unwrap().file_name().unwrap(), APP_DIR_NAME);
        assert_eq!(data_dir().unwrap().file_name().unwrap(), APP_DIR_NAME);
        assert_eq!(runtime_dir().unwrap().file_name().unwrap(), APP_DIR_NAME);
    }

    #[test]
    fn config_file_uses_toml_extension() {
        assert_eq!(config_file().unwrap().file_name().unwrap(), "config.toml");
    }

    #[test]
    fn database_file_uses_sqlite_extension() {
        assert_eq!(
            database_file().unwrap().file_name().unwrap(),
            "openwhatsapp.sqlite3"
        );
    }

    #[test]
    fn temp_download_dir_lives_below_runtime_dir() {
        assert!(temp_download_dir()
            .unwrap()
            .starts_with(runtime_dir().unwrap()));
    }

    #[test]
    fn account_web_dirs_include_account_id() {
        let account_id = Uuid::new_v4();
        let account_id_text = account_id.to_string();

        assert!(account_web_data_dir(account_id)
            .unwrap()
            .to_string_lossy()
            .contains(&account_id_text));
        assert!(account_web_cache_dir(account_id)
            .unwrap()
            .to_string_lossy()
            .contains(&account_id_text));
    }

    #[test]
    fn customization_dirs_follow_expected_layout() {
        let account_id = Uuid::new_v4();
        let account_id_text = account_id.to_string();

        assert!(global_css_dir()
            .unwrap()
            .to_string_lossy()
            .ends_with("customizations/global/css"));
        assert!(global_js_dir()
            .unwrap()
            .to_string_lossy()
            .ends_with("customizations/global/js"));
        assert!(account_css_dir(account_id)
            .unwrap()
            .to_string_lossy()
            .contains(&account_id_text));
        assert!(account_js_dir(account_id)
            .unwrap()
            .to_string_lossy()
            .contains(&account_id_text));
        assert!(extensions_dir()
            .unwrap()
            .to_string_lossy()
            .ends_with("customizations/extensions"));
    }
}
