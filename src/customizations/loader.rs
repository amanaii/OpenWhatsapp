//! Customization file loading.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use uuid::Uuid;

use super::store::CustomizationStore;
use super::{userstyle, CustomizationEntry, CustomizationKind};

/// Loaded customization content.
pub(crate) struct LoadedCustomization {
    /// Content kind.
    pub(crate) kind: CustomizationKind,
    /// File content ready to inject.
    pub(crate) content: String,
}

/// Loads enabled customizations for an account.
pub(crate) fn load_enabled(
    store: &CustomizationStore,
    account_id: Option<Uuid>,
) -> Result<Vec<LoadedCustomization>> {
    store
        .enabled_for_account(account_id)?
        .into_iter()
        .map(load_entry)
        .collect()
}

fn load_entry(entry: CustomizationEntry) -> Result<LoadedCustomization> {
    let content = fs::read_to_string(&entry.path)
        .with_context(|| format!("failed to read customization {}", entry.path.display()))?;
    let content = match entry.kind {
        CustomizationKind::Css if is_userstyle(&entry.path) => {
            userstyle::extract_whatsapp_css(&content).unwrap_or(content)
        }
        _ => content,
    };

    Ok(LoadedCustomization {
        kind: entry.kind,
        content,
    })
}

fn is_userstyle(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".user.css"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn userstyle_detection_uses_suffix() {
        assert!(is_userstyle(&PathBuf::from("a.user.css")));
        assert!(!is_userstyle(&PathBuf::from("a.css")));
    }
}
