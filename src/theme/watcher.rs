//! System theme watcher.

use anyhow::{Context, Result};
use gio::prelude::*;

use crate::ipc::{AppEvent, EventBus};

const PORTAL_BUS_NAME: &str = "org.freedesktop.portal.Desktop";
const PORTAL_OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";
const PORTAL_SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";
const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";
const COLOR_SCHEME_KEY: &str = "color-scheme";
const DBUS_TIMEOUT_MS: i32 = 250;

/// Portal color scheme value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SystemColorScheme {
    /// No system preference.
    NoPreference,
    /// System prefers dark UI.
    PreferDark,
    /// System prefers light UI.
    PreferLight,
}

/// Keeps the portal settings watcher alive.
pub(crate) struct ThemeWatcher {
    proxy: gio::DBusProxy,
}

impl ThemeWatcher {
    /// Keeps the watcher captured by GTK signal closures.
    pub(crate) fn keep_alive(&self) {
        let _ = self.proxy.name();
    }
}

/// Starts watching portal color-scheme changes.
pub(crate) fn start(events: EventBus) -> Result<ThemeWatcher> {
    let proxy = portal_settings_proxy()?;

    if let Some(scheme) = read_current_scheme(&proxy)? {
        let effective_theme = super::apply_system_scheme(scheme)?;
        let _subscriber_count = events.emit(AppEvent::ThemeChanged(effective_theme));
    }

    let events_for_signal = events.clone();
    proxy.connect_g_signal(move |_, _, signal_name, parameters| {
        if signal_name != "SettingChanged" {
            return;
        }

        let Some(scheme) = parse_setting_changed(parameters) else {
            return;
        };

        match super::apply_system_scheme(scheme) {
            Ok(effective_theme) => {
                let _subscriber_count =
                    events_for_signal.emit(AppEvent::ThemeChanged(effective_theme));
            }
            Err(error) => tracing::error!(?error, "failed to apply system theme change"),
        }
    });

    Ok(ThemeWatcher { proxy })
}

fn portal_settings_proxy() -> Result<gio::DBusProxy> {
    gio::DBusProxy::for_bus_sync(
        gio::BusType::Session,
        gio::DBusProxyFlags::NONE,
        None::<&gio::DBusInterfaceInfo>,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        PORTAL_SETTINGS_INTERFACE,
        None::<&gio::Cancellable>,
    )
    .context("failed to connect to xdg-desktop-portal settings")
}

fn read_current_scheme(proxy: &gio::DBusProxy) -> Result<Option<SystemColorScheme>> {
    let parameters = (APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY).to_variant();
    let response = proxy
        .call_sync(
            "ReadOne",
            Some(&parameters),
            gio::DBusCallFlags::NONE,
            DBUS_TIMEOUT_MS,
            None::<&gio::Cancellable>,
        )
        .context("failed to read portal color-scheme")?;

    Ok(response
        .try_child_value(0)
        .and_then(|value| value.as_variant())
        .and_then(|value| value.get::<u32>())
        .map(SystemColorScheme::from))
}

fn parse_setting_changed(parameters: &gio::glib::Variant) -> Option<SystemColorScheme> {
    let namespace = parameters.try_child_get::<String>(0).ok().flatten()?;
    let key = parameters.try_child_get::<String>(1).ok().flatten()?;

    if namespace != APPEARANCE_NAMESPACE || key != COLOR_SCHEME_KEY {
        return None;
    }

    parameters
        .try_child_value(2)
        .and_then(|value| value.as_variant())
        .and_then(|value| value.get::<u32>())
        .map(SystemColorScheme::from)
}

impl From<u32> for SystemColorScheme {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::PreferDark,
            2 => Self::PreferLight,
            _ => Self::NoPreference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portal_values_map_to_color_scheme() {
        assert_eq!(SystemColorScheme::from(0), SystemColorScheme::NoPreference);
        assert_eq!(SystemColorScheme::from(1), SystemColorScheme::PreferDark);
        assert_eq!(SystemColorScheme::from(2), SystemColorScheme::PreferLight);
        assert_eq!(SystemColorScheme::from(99), SystemColorScheme::NoPreference);
    }

    #[test]
    fn setting_changed_parser_ignores_other_keys() {
        let parameters = ("org.example", COLOR_SCHEME_KEY, 1_u32.to_variant()).to_variant();

        assert_eq!(parse_setting_changed(&parameters), None);
    }

    #[test]
    fn setting_changed_parser_reads_color_scheme() {
        let parameters = (APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY, 1_u32.to_variant()).to_variant();

        assert_eq!(
            parse_setting_changed(&parameters),
            Some(SystemColorScheme::PreferDark)
        );
    }
}
