//! Temporary download folder handling.

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::utils::paths;

/// Creates and returns the runtime temp folder.
pub(crate) fn prepare() -> Result<PathBuf> {
    let mut last_error = None;

    for directory in temp_candidates()? {
        match fs::create_dir_all(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) => last_error = Some((directory, error)),
        }
    }

    let (directory, error) = last_error.context("no temp download directory candidates")?;
    Err(error)
        .with_context(|| format!("failed to create temp download dir {}", directory.display()))
}

/// Removes the runtime temp folder if it exists.
pub(crate) fn cleanup() -> Result<()> {
    for directory in temp_candidates()? {
        if directory.exists() {
            fs::remove_dir_all(&directory).with_context(|| {
                format!("failed to clean temp download dir {}", directory.display())
            })?;
        }
    }

    Ok(())
}

fn temp_candidates() -> Result<Vec<PathBuf>> {
    let primary = paths::temp_download_dir()?;
    let fallback = std::env::temp_dir().join("openwhatsapp").join("tmp");

    if primary == fallback {
        Ok(vec![primary])
    } else {
        Ok(vec![primary, fallback])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_returns_tmp_directory() {
        let path = paths::temp_download_dir().unwrap();

        assert_eq!(path.file_name().unwrap(), "tmp");
    }

    #[test]
    fn temp_candidates_include_fallback() {
        assert!(temp_candidates()
            .unwrap()
            .iter()
            .any(|path| path.starts_with(std::env::temp_dir())));
    }
}
