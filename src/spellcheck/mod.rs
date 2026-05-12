//! Spellcheck integration.

use anyhow::Result;
use webkit6::prelude::*;

use crate::config::SpellcheckConfig;

pub(crate) mod context_menu;
pub(crate) mod languages;

/// Applies spellcheck settings to a WebKit webview.
pub(crate) fn apply_to_webview(
    webview: &webkit6::WebView,
    config: &SpellcheckConfig,
) -> Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let languages = languages::resolve_languages(config)?;
    let language_refs = languages.iter().map(String::as_str).collect::<Vec<_>>();

    if let Some(context) = webview.context() {
        context.set_spell_checking_enabled(config.enabled && !language_refs.is_empty());
        context.set_spell_checking_languages(&language_refs);
    }

    context_menu::attach_language_menu(webview, languages);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_default_config_stays_disabled() {
        let config = SpellcheckConfig::default();

        assert!(!config.enabled);
    }
}
