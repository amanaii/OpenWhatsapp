//! Download routing.

use std::path::PathBuf;

use anyhow::Result;

use crate::accounts::Account;
use crate::config::DownloadConfig;
use crate::ipc::{AppEvent, EventBus};

pub(crate) mod custom_folder;
pub(crate) mod temp_folder;

/// Prepares the runtime temporary download folder.
pub(crate) fn prepare_temp_folder() -> Result<PathBuf> {
    temp_folder::prepare()
}

/// Cleans the runtime temporary download folder.
pub(crate) fn cleanup_temp_folder() -> Result<()> {
    temp_folder::cleanup()
}

/// Attaches download routing to a WebKit network session.
pub(crate) fn attach_to_network_session(
    session: &webkit6::NetworkSession,
    account: &Account,
    config: &DownloadConfig,
    events: EventBus,
) {
    let account = account.clone();
    let config = config.clone();

    session.connect_download_started(move |_, download| {
        let account = account.clone();
        let config = config.clone();
        let events = events.clone();

        download.set_allow_overwrite(false);
        download.connect_decide_destination(move |download, suggested_filename| {
            match custom_folder::destination_for(&config, &account, suggested_filename) {
                Ok(destination) => match webkit6::glib::filename_to_uri(&destination, None) {
                    Ok(uri) => {
                        download.set_destination(uri.as_str());
                        let _subscriber_count = events.emit(AppEvent::DownloadStarted {
                            account_id: account.id,
                            path: destination,
                        });
                        true
                    }
                    Err(error) => {
                        tracing::error!(?error, "failed to build download destination URI");
                        false
                    }
                },
                Err(error) => {
                    tracing::error!(?error, "failed to choose download destination");
                    false
                }
            }
        });
    });
}
