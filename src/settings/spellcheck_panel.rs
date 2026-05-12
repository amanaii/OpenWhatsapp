//! Spellcheck settings.

use gtk::prelude::*;

use crate::config::AppConfig;

pub(super) struct SpellcheckPanel {
    root: gtk::Box,
    enabled: gtk::Switch,
    languages: gtk::Entry,
    dictionary_dir: gtk::Entry,
}

impl SpellcheckPanel {
    pub(super) fn new(config: &AppConfig) -> Self {
        let root = super::panel_box();
        let enabled = gtk::Switch::builder()
            .active(config.spellcheck.enabled)
            .build();
        let languages = gtk::Entry::new();
        languages.set_text(&config.spellcheck.languages.join(","));
        let dictionary_dir = gtk::Entry::new();
        if let Some(path) = &config.spellcheck.custom_dictionary_dir {
            dictionary_dir.set_text(&path.to_string_lossy());
        }

        root.append(&super::row("Enable spellcheck", &enabled));
        root.append(&super::row("Languages", &languages));
        root.append(&super::row("Custom dictionary folder", &dictionary_dir));

        Self {
            root,
            enabled,
            languages,
            dictionary_dir,
        }
    }

    pub(super) fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    pub(super) fn write_config(&self, config: &mut AppConfig) {
        config.spellcheck.enabled = self.enabled.is_active();
        config.spellcheck.languages = self
            .languages
            .text()
            .split(',')
            .map(str::trim)
            .filter(|language| !language.is_empty())
            .map(ToOwned::to_owned)
            .collect();
        let dictionary_dir = self.dictionary_dir.text();
        config.spellcheck.custom_dictionary_dir = if dictionary_dir.trim().is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(dictionary_dir.as_str()))
        };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn comma_language_list_can_split() {
        let languages = "en_US,de_DE";

        assert_eq!(languages.split(',').count(), 2);
    }
}
