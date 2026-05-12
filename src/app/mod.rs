//! Application bootstrap.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use gtk::prelude::*;

use crate::accounts::{Account, AccountGrid};
use crate::config::AppConfig;
use crate::customizations::CustomizationStore;
use crate::ipc::{AppEvent, EventBus};
use crate::notifications;
use crate::shortcuts;
use crate::theme;
use crate::tray;
use crate::webview::AccountWebViews;
use crate::window;

const APPLICATION_ID: &str = "io.github.openwhatsapp.OpenWhatsapp";
const APP_TITLE: &str = "OpenWhatsapp";
const APP_ICON_NAME: &str = "openwhatsapp";
const APP_ICON_DIR: &str = "assets/icons";

/// Runs the GTK application.
pub(crate) fn run(
    config_path: PathBuf,
    config: AppConfig,
    accounts: Vec<Account>,
    customization_store: Arc<CustomizationStore>,
    events: EventBus,
) {
    let application = gtk::Application::new(
        Some(application_id()),
        gtk::gio::ApplicationFlags::FLAGS_NONE,
    );
    let shared_config = Arc::new(Mutex::new(config));

    let shared_config_for_activate = Arc::clone(&shared_config);
    application.connect_activate(move |application| {
        build_main_window(
            application,
            &config_path,
            &accounts,
            Arc::clone(&customization_store),
            Arc::clone(&shared_config_for_activate),
            &events,
        );
    });

    let _exit_code = application.run();
}

fn build_main_window(
    application: &gtk::Application,
    config_path: &std::path::Path,
    accounts: &[Account],
    customization_store: Arc<CustomizationStore>,
    shared_config: Arc<Mutex<AppConfig>>,
    events: &EventBus,
) {
    install_app_icon();
    let config = snapshot_config(&shared_config);
    if let Err(error) = theme::apply_preference(&config.appearance.theme) {
        tracing::error!(?error, "failed to apply theme preference");
    }

    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.set_hexpand(true);
    root.set_vexpand(true);

    if !accounts.is_empty() {
        let account_grid = AccountGrid::new(accounts, events.clone());
        root.append(account_grid.widget());
    }

    let (webview_stack, webview_widgets) = match AccountWebViews::new(
        accounts,
        &config.spellcheck,
        &config.downloads,
        customization_store,
        events.clone(),
    ) {
        Ok(webviews) => {
            let stack = webviews.widget().clone();
            let widgets = webviews.webviews().to_vec();
            root.append(webviews.widget());
            (stack, widgets)
        }
        Err(error) => {
            tracing::error!(?error, "failed to build account webviews");
            return;
        }
    };

    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title(APP_TITLE)
        .icon_name(APP_ICON_NAME)
        .default_width(config.window.width)
        .default_height(config.window.height)
        .child(&root)
        .build();

    if let Err(error) = window::configure(&window, &config, events.clone()) {
        tracing::error!(?error, "failed to configure window");
    }
    crate::webview::install_workspace_resume_workaround(&window, &webview_stack, &webview_widgets);
    notifications::install_actions(application, events.clone());
    shortcuts::install(&window, accounts, events.clone());
    bind_app_events(
        application,
        &window,
        config_path.to_path_buf(),
        Arc::clone(&shared_config),
        events.clone(),
    );

    match theme::watch_system_changes(&config.appearance.theme, events.clone()) {
        Ok(Some(watcher)) => {
            window.connect_destroy(move |_| {
                watcher.keep_alive();
            });
        }
        Ok(None) => {}
        Err(error) => tracing::warn!(?error, "system theme watcher unavailable"),
    }

    if config.usability.tray_enabled {
        match tray::start(
            accounts,
            events.clone(),
            config.usability.tray_icon_path.clone(),
        ) {
            Ok(tray) => {
                window.connect_destroy(move |_| {
                    tray.keep_alive();
                });
            }
            Err(error) => tracing::warn!(?error, "tray unavailable"),
        }
    }

    window::install_geometry_persistence(&window, config_path.to_path_buf(), shared_config);
    window.present();
    let _subscriber_count = events.emit(AppEvent::AppReady);
}

fn bind_app_events(
    application: &gtk::Application,
    window: &gtk::ApplicationWindow,
    config_path: PathBuf,
    shared_config: Arc<Mutex<AppConfig>>,
    events: EventBus,
) {
    let mut receiver = events.subscribe();
    let application = application.clone();
    let window = window.clone();

    gtk::glib::MainContext::default().spawn_local(async move {
        loop {
            match receiver.recv().await {
                Ok(AppEvent::WindowShow) => window.present(),
                Ok(AppEvent::WindowHide) => window.set_visible(false),
                Ok(AppEvent::SettingsOpen) => crate::settings::show_dialog(
                    &window,
                    config_path.clone(),
                    Arc::clone(&shared_config),
                    events.clone(),
                ),
                Ok(AppEvent::NewAccountRequested) => {
                    tracing::info!("new account shortcut requested");
                }
                Ok(AppEvent::NotificationReceived {
                    account_id,
                    title,
                    body,
                }) => notifications::send(&application, account_id, &title, &body),
                Ok(AppEvent::Quit) => application.quit(),
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });
}

fn snapshot_config(shared_config: &Arc<Mutex<AppConfig>>) -> AppConfig {
    match shared_config.lock() {
        Ok(config) => config.clone(),
        Err(error) => {
            tracing::error!(?error, "config lock poisoned");
            AppConfig::default()
        }
    }
}

fn application_id() -> &'static str {
    APPLICATION_ID
}

fn install_app_icon() {
    gtk::Window::set_default_icon_name(APP_ICON_NAME);

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::IconTheme::for_display(&display).add_search_path(APP_ICON_DIR);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_id_is_reverse_dns() {
        assert!(application_id().contains('.'));
    }

    #[test]
    fn app_icon_name_is_stable() {
        assert_eq!(APP_ICON_NAME, "openwhatsapp");
    }
}
