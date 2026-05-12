//! Drag and drop integration.

use gtk::gio::prelude::*;
use gtk::prelude::*;
use webkit6::prelude::*;

const ACCEPTED_MIME_TYPES: &[&str] = &[
    "text/uri-list",
    "text/plain",
    "application/octet-stream",
    "image/png",
    "image/jpeg",
    "image/gif",
    "video/mp4",
    "audio/mpeg",
    "application/pdf",
];
const READ_MIME_TYPES: &[&str] = &["text/uri-list", "text/plain"];
const MAX_DROP_BYTES: usize = 1024 * 1024;

/// Attaches file drag-and-drop handling to a WebKit webview.
pub(crate) fn attach_to_webview(webview: &webkit6::WebView) {
    let formats = gtk::gdk::ContentFormats::new(ACCEPTED_MIME_TYPES);
    let target = gtk::DropTargetAsync::new(Some(formats), gtk::gdk::DragAction::COPY);
    let webview_for_drop = webview.clone();

    target.connect_accept(|_, _| true);
    target.connect_drop(move |_, drop, _, _| {
        let drop = drop.clone();
        let webview = webview_for_drop.clone();

        gtk::glib::MainContext::default().spawn_local(async move {
            if let Err(error) = handle_drop(&webview, &drop).await {
                tracing::error!(?error, "failed to handle file drop");
                drop.finish(gtk::gdk::DragAction::empty());
                return;
            }

            drop.finish(gtk::gdk::DragAction::COPY);
        });

        true
    });

    webview.add_controller(target);
}

async fn handle_drop(webview: &webkit6::WebView, drop: &gtk::gdk::Drop) -> anyhow::Result<()> {
    let (stream, _mime_type) = drop
        .read_future(READ_MIME_TYPES, gtk::glib::Priority::DEFAULT)
        .await?;
    let bytes = stream
        .read_bytes_future(MAX_DROP_BYTES, gtk::glib::Priority::DEFAULT)
        .await?;
    let text = String::from_utf8_lossy(bytes.as_ref());
    let uris = parse_uri_list(&text);

    if uris.is_empty() {
        return Ok(());
    }

    inject_attach_event(webview, &uris);

    Ok(())
}

fn inject_attach_event(webview: &webkit6::WebView, uris: &[String]) {
    let script = format!(
        "window.dispatchEvent(new CustomEvent('openwhatsapp-files-dropped', {{ detail: {{ uris: [{}] }} }}));",
        uris.iter()
            .map(|uri| format!("\"{}\"", escape_js_string(uri)))
            .collect::<Vec<_>>()
            .join(",")
    );

    webview.evaluate_javascript(
        &script,
        None,
        None,
        None::<&webkit6::gio::Cancellable>,
        |result| {
            if let Err(error) = result {
                tracing::error!(?error, "failed to inject drop event");
            }
        },
    );
}

fn parse_uri_list(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
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
    fn uri_list_parser_ignores_comments_and_blanks() {
        assert_eq!(
            parse_uri_list("# comment\n\nfile:///tmp/a.png\nfile:///tmp/b.pdf\n"),
            vec![
                "file:///tmp/a.png".to_string(),
                "file:///tmp/b.pdf".to_string()
            ]
        );
    }

    #[test]
    fn js_string_escape_handles_quotes_and_newlines() {
        assert_eq!(escape_js_string("a\"b\nc"), "a\\\"b\\nc");
    }
}
