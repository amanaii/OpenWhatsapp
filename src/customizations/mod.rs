//! User customization pipeline.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use uuid::Uuid;
use webkit6::prelude::*;

use crate::utils::paths;

pub(crate) mod editor;
pub(crate) mod loader;
pub(crate) mod store;
pub(crate) mod url_loader;
pub(crate) mod userstyle;

pub(crate) use store::CustomizationStore;

/// Customization content kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CustomizationKind {
    /// CSS stylesheet.
    Css,
    /// JavaScript script.
    Js,
}

/// Customization source kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CustomizationSource {
    /// Local file source.
    File,
    /// Remote HTTPS URL source.
    Url,
}

/// Persisted customization entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CustomizationEntry {
    /// Stable customization ID.
    pub(crate) id: Uuid,
    /// Optional account override target.
    pub(crate) account_id: Option<Uuid>,
    /// Local cached file path.
    pub(crate) path: PathBuf,
    /// Content kind.
    pub(crate) kind: CustomizationKind,
    /// Whether this entry is enabled.
    pub(crate) enabled: bool,
    /// Source kind.
    pub(crate) source: CustomizationSource,
    /// Optional remote URL.
    pub(crate) url: Option<String>,
    /// Optional cached ETag.
    pub(crate) etag: Option<String>,
}

/// Ensures base customization folders exist.
pub(crate) fn ensure_base_dirs() -> Result<()> {
    for directory in [
        paths::global_css_dir()?,
        paths::global_js_dir()?,
        paths::extensions_dir()?,
    ] {
        std::fs::create_dir_all(&directory).with_context(|| {
            format!("failed to create customization dir {}", directory.display())
        })?;
    }

    Ok(())
}

/// Attaches customization injection after each page load.
pub(crate) fn attach_to_webview(
    webview: &webkit6::WebView,
    account_id: Option<Uuid>,
    store: Arc<CustomizationStore>,
) {
    webview.connect_load_changed(move |webview, event| {
        if event != webkit6::LoadEvent::Finished {
            return;
        }

        if let Err(error) = inject_enabled(webview, account_id, &store) {
            tracing::error!(?error, "failed to inject customizations");
        }
    });
}

fn inject_enabled(
    webview: &webkit6::WebView,
    account_id: Option<Uuid>,
    store: &CustomizationStore,
) -> Result<()> {
    for loaded in loader::load_enabled(store, account_id)? {
        match loaded.kind {
            CustomizationKind::Css => inject_css(webview, &loaded.content),
            CustomizationKind::Js => inject_js(webview, &loaded.content),
        }
    }

    Ok(())
}

fn inject_css(webview: &webkit6::WebView, css: &str) {
    let script = format!(
        "(() => {{ const style = document.createElement('style'); style.dataset.openwhatsapp = 'true'; style.textContent = \"{}\"; document.documentElement.appendChild(style); }})();",
        escape_js_string(css)
    );
    inject_js(webview, &script);
}

fn inject_js(webview: &webkit6::WebView, script: &str) {
    webview.evaluate_javascript(
        script,
        None,
        None,
        None::<&webkit6::gio::Cancellable>,
        |result| {
            if let Err(error) = result {
                tracing::error!(?error, "customization injection failed");
            }
        },
    );
}

fn escape_js_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_escape_handles_quotes() {
        assert_eq!(escape_js_string("a\"b"), "a\\\"b");
    }
}
