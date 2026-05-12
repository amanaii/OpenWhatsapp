//! Theme integration.

use anyhow::{Context, Result};

use crate::config::Theme;
use crate::ipc::EventBus;

pub(crate) mod watcher;

/// Applies the configured theme preference to GTK.
pub(crate) fn apply_preference(preference: &Theme) -> Result<()> {
    let settings = gtk::Settings::default().context("failed to locate default GTK settings")?;
    settings.set_gtk_application_prefer_dark_theme(prefer_dark_for(preference));

    Ok(())
}

/// Starts the system theme watcher when auto theme is enabled.
pub(crate) fn watch_system_changes(
    preference: &Theme,
    events: EventBus,
) -> Result<Option<watcher::ThemeWatcher>> {
    if preference != &Theme::Auto {
        return Ok(None);
    }

    watcher::start(events).map(Some)
}

fn apply_system_scheme(scheme: watcher::SystemColorScheme) -> Result<Theme> {
    let effective_theme = theme_for_system_scheme(scheme);
    apply_preference(&effective_theme)?;

    Ok(effective_theme)
}

fn prefer_dark_for(preference: &Theme) -> bool {
    matches!(preference, Theme::Dark)
}

fn theme_for_system_scheme(scheme: watcher::SystemColorScheme) -> Theme {
    match scheme {
        watcher::SystemColorScheme::PreferDark => Theme::Dark,
        watcher::SystemColorScheme::PreferLight => Theme::Light,
        watcher::SystemColorScheme::NoPreference => Theme::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_preference_enables_dark_theme() {
        assert!(prefer_dark_for(&Theme::Dark));
        assert!(!prefer_dark_for(&Theme::Light));
        assert!(!prefer_dark_for(&Theme::Auto));
    }

    #[test]
    fn system_scheme_maps_to_effective_theme() {
        assert_eq!(
            theme_for_system_scheme(watcher::SystemColorScheme::PreferDark),
            Theme::Dark
        );
        assert_eq!(
            theme_for_system_scheme(watcher::SystemColorScheme::PreferLight),
            Theme::Light
        );
        assert_eq!(
            theme_for_system_scheme(watcher::SystemColorScheme::NoPreference),
            Theme::Auto
        );
    }
}
