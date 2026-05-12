//! Spellcheck language discovery.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::SpellcheckConfig;

/// Resolves selected or installed spellcheck languages.
pub(crate) fn resolve_languages(config: &SpellcheckConfig) -> Result<Vec<String>> {
    if !config.languages.is_empty() {
        return Ok(normalize_languages(config.languages.iter().cloned()));
    }

    installed_languages(config.custom_dictionary_dir.as_deref())
}

/// Lists installed Enchant dictionary languages.
pub(crate) fn installed_languages(custom_dictionary_dir: Option<&Path>) -> Result<Vec<String>> {
    let mut languages = enchant::Broker::new()
        .list_dicts()
        .into_iter()
        .map(|dictionary| dictionary.lang)
        .collect::<Vec<_>>();

    if let Some(directory) = custom_dictionary_dir {
        languages.extend(custom_dictionary_languages(directory)?);
    }

    Ok(normalize_languages(languages))
}

fn custom_dictionary_languages(directory: &Path) -> Result<Vec<String>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut languages = Vec::new();
    for entry in fs::read_dir(directory)
        .with_context(|| format!("failed to read dictionary dir {}", directory.display()))?
    {
        let entry = entry.context("failed to read dictionary entry")?;
        let path = entry.path();
        if is_dictionary_file(&path) {
            if let Some(stem) = path.file_stem() {
                languages.push(stem.to_string_lossy().into_owned());
            }
        }
    }

    languages.sort();
    Ok(languages)
}

fn is_dictionary_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("dic" | "aff")
    )
}

fn normalize_languages(languages: impl IntoIterator<Item = String>) -> Vec<String> {
    languages
        .into_iter()
        .map(|language| language.trim().to_string())
        .filter(|language| !language.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn language_normalization_sorts_and_deduplicates() {
        assert_eq!(
            normalize_languages(["en_US", "en_US", " de_DE "].map(String::from)),
            vec!["de_DE".to_string(), "en_US".to_string()]
        );
    }

    #[test]
    fn custom_dictionary_language_scan_reads_dic_and_aff() {
        let dir = std::env::temp_dir().join(format!("openwhatsapp-dicts-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("en_US.dic"), "").unwrap();
        std::fs::write(dir.join("de_DE.aff"), "").unwrap();
        std::fs::write(dir.join("notes.txt"), "").unwrap();

        assert_eq!(
            custom_dictionary_languages(&dir).unwrap(),
            vec!["de_DE".to_string(), "en_US".to_string()]
        );

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn dictionary_file_detection_uses_expected_extensions() {
        assert!(is_dictionary_file(&PathBuf::from("en_US.dic")));
        assert!(is_dictionary_file(&PathBuf::from("en_US.aff")));
        assert!(!is_dictionary_file(&PathBuf::from("en_US.txt")));
    }
}
