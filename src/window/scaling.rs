//! Window scaling helpers.

use anyhow::{Context, Result};
use gtk::prelude::*;

const BASE_FONT_SIZE_PX: f64 = 16.0;
const MIN_SCALE: f64 = 0.75;
const MAX_SCALE: f64 = 2.0;

pub(super) fn apply_interface_scale(window: &gtk::ApplicationWindow, scale: f64) -> Result<()> {
    let scale = clamp_scale(scale);
    let css = format!(
        ".openwhatsapp-scaled {{ font-size: {:.2}px; }}",
        font_size_px(scale)
    );
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);

    let display = gtk::gdk::Display::default().context("failed to locate default GTK display")?;
    gtk::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    window.add_css_class("openwhatsapp-scaled");

    Ok(())
}

fn clamp_scale(scale: f64) -> f64 {
    if scale.is_finite() {
        scale.clamp(MIN_SCALE, MAX_SCALE)
    } else {
        1.0
    }
}

fn font_size_px(scale: f64) -> f64 {
    BASE_FONT_SIZE_PX * clamp_scale(scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_is_clamped_to_supported_range() {
        assert_eq!(clamp_scale(0.1), MIN_SCALE);
        assert_eq!(clamp_scale(5.0), MAX_SCALE);
        assert_eq!(clamp_scale(f64::NAN), 1.0);
    }

    #[test]
    fn font_size_tracks_scale() {
        assert_eq!(font_size_px(1.5), 24.0);
    }
}
