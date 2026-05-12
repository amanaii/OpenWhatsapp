//! Account profile metadata.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use uuid::Uuid;

/// Persisted WhatsApp account profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Account {
    /// Stable account identifier.
    pub(crate) id: Uuid,
    /// Display name shown in account UI.
    pub(crate) display_name: String,
    /// Optional avatar image path.
    pub(crate) avatar_path: Option<PathBuf>,
    /// Creation timestamp as Unix seconds.
    pub(crate) created_at: i64,
    /// Optional custom download folder.
    pub(crate) custom_download_dir: Option<PathBuf>,
    /// Whether account customizations override global settings.
    pub(crate) customization_override: bool,
}

impl Account {
    /// Creates a new account profile with a generated UUID.
    #[allow(dead_code)]
    pub(crate) fn new(display_name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            display_name: display_name.into(),
            avatar_path: None,
            created_at: unix_timestamp_now()?,
            custom_download_dir: None,
            customization_override: false,
        })
    }
}

fn unix_timestamp_now() -> Result<i64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?;

    i64::try_from(duration.as_secs()).context("Unix timestamp exceeds i64")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_account_gets_uuid_and_timestamp() {
        let account = Account::new("Work").unwrap();

        assert_eq!(account.display_name, "Work");
        assert!(account.created_at > 0);
    }
}
