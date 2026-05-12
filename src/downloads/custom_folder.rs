//! Custom download folder handling.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::accounts::Account;
use crate::config::DownloadConfig;

/// Returns a concrete destination path for a WebKit download.
pub(crate) fn destination_for(
    config: &DownloadConfig,
    account: &Account,
    suggested_filename: &str,
) -> Result<PathBuf> {
    let directory = resolve_download_dir(config, account)?;
    fs::create_dir_all(&directory)
        .with_context(|| format!("failed to create download dir {}", directory.display()))?;

    Ok(unique_destination(
        &directory,
        &safe_filename(suggested_filename),
    ))
}

fn resolve_download_dir(config: &DownloadConfig, account: &Account) -> Result<PathBuf> {
    if let Some(directory) = &account.custom_download_dir {
        return Ok(directory.clone());
    }

    if let Some(directory) = &config.directory {
        return Ok(directory.clone());
    }

    dirs::download_dir().context("failed to locate XDG downloads directory")
}

fn safe_filename(suggested_filename: &str) -> String {
    let filename = Path::new(suggested_filename)
        .file_name()
        .map(|filename| filename.to_string_lossy().into_owned())
        .unwrap_or_else(|| "download".to_string());

    if filename.trim().is_empty() {
        "download".to_string()
    } else {
        filename
    }
}

fn unique_destination(directory: &Path, filename: &str) -> PathBuf {
    let first = directory.join(filename);
    if !first.exists() {
        return first;
    }

    let path = Path::new(filename);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .filter(|stem| !stem.is_empty())
        .unwrap_or_else(|| "download".to_string());
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy());

    for index in 1..10_000 {
        let candidate_name = match &extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = directory.join(candidate_name);

        if !candidate.exists() {
            return candidate;
        }
    }

    directory.join(format!("{stem} (copy)"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn safe_filename_strips_parent_components() {
        assert_eq!(safe_filename("../secret.txt"), "secret.txt");
        assert_eq!(safe_filename(""), "download");
    }

    #[test]
    fn unique_destination_adds_suffix() {
        let directory =
            std::env::temp_dir().join(format!("openwhatsapp-downloads-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("file.txt"), "").unwrap();

        assert_eq!(
            unique_destination(&directory, "file.txt"),
            directory.join("file (1).txt")
        );

        std::fs::remove_dir_all(directory).unwrap();
    }
}
