//! Spellcheck context menu integration.

use webkit6::gio::prelude::*;
use webkit6::prelude::*;

/// Attaches a spellcheck language submenu to a webview context menu.
pub(crate) fn attach_language_menu(webview: &webkit6::WebView, languages: Vec<String>) {
    if languages.is_empty() {
        return;
    }

    webview.connect_context_menu(move |webview, menu, _hit_test| {
        let submenu = build_language_submenu(webview, &languages);
        menu.append(&webkit6::ContextMenuItem::new_separator());
        menu.append(&webkit6::ContextMenuItem::with_submenu(
            "Spellcheck Language",
            &submenu,
        ));

        false
    });
}

fn build_language_submenu(
    webview: &webkit6::WebView,
    languages: &[String],
) -> webkit6::ContextMenu {
    let submenu = webkit6::ContextMenu::new();
    let webview_weak = webview.downgrade();

    for language in languages {
        let action = webkit6::gio::SimpleAction::new(&action_name(language), None);
        let language_for_action = language.clone();
        let webview_weak = webview_weak.clone();

        action.connect_activate(move |_, _| {
            if let Some(webview) = webview_weak.upgrade() {
                set_webview_language(&webview, &language_for_action);
            }
        });

        submenu.append(&webkit6::ContextMenuItem::from_gaction(
            &action, language, None,
        ));
    }

    submenu
}

fn set_webview_language(webview: &webkit6::WebView, language: &str) {
    if let Some(context) = webview.context() {
        context.set_spell_checking_enabled(true);
        context.set_spell_checking_languages(&[language]);
    }
}

fn action_name(language: &str) -> String {
    let normalized = language
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();

    format!("spellcheck-{normalized}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_name_is_simple_action_safe() {
        assert_eq!(action_name("en_US"), "spellcheck-en-us");
    }
}
