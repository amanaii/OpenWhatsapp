//! Account persistence.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::profile::Account;

const MIGRATION_VERSION: i64 = 1;

/// SQLite-backed account store.
pub(crate) struct AccountStore {
    connection: Mutex<Connection>,
}

impl AccountStore {
    /// Opens an account store and runs pending migrations.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create account DB dir {}", parent.display()))?;
        }

        let connection = Connection::open(path)
            .with_context(|| format!("failed to open account DB {}", path.display()))?;
        let store = Self {
            connection: Mutex::new(connection),
        };
        store.migrate()?;

        Ok(store)
    }

    /// Lists accounts ordered by creation time.
    pub(crate) fn list_accounts(&self) -> Result<Vec<Account>> {
        let connection = self.lock_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, display_name, avatar_path, created_at, custom_download_dir, \
                 customization_override FROM accounts ORDER BY created_at ASC, display_name ASC",
            )
            .context("failed to prepare account list query")?;
        let accounts = statement
            .query_map([], row_to_account)
            .context("failed to query accounts")?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read account rows")?;

        Ok(accounts)
    }

    /// Inserts a new account.
    #[allow(dead_code)]
    pub(crate) fn add_account(&self, account: &Account) -> Result<()> {
        let connection = self.lock_connection()?;
        connection
            .execute(
                "INSERT INTO accounts (
                    id, display_name, avatar_path, created_at, custom_download_dir,
                    customization_override
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    account.id.to_string(),
                    account.display_name,
                    optional_path_to_db(&account.avatar_path),
                    account.created_at,
                    optional_path_to_db(&account.custom_download_dir),
                    account.customization_override,
                ],
            )
            .context("failed to insert account")?;

        Ok(())
    }

    /// Returns an account by ID.
    #[allow(dead_code)]
    pub(crate) fn get_account(&self, id: Uuid) -> Result<Option<Account>> {
        let connection = self.lock_connection()?;
        connection
            .query_row(
                "SELECT id, display_name, avatar_path, created_at, custom_download_dir,
                    customization_override
                 FROM accounts WHERE id = ?1",
                params![id.to_string()],
                row_to_account,
            )
            .optional()
            .context("failed to get account")
    }

    /// Updates an existing account.
    #[allow(dead_code)]
    pub(crate) fn update_account(&self, account: &Account) -> Result<bool> {
        let connection = self.lock_connection()?;
        let changed = connection
            .execute(
                "UPDATE accounts SET
                    display_name = ?2,
                    avatar_path = ?3,
                    created_at = ?4,
                    custom_download_dir = ?5,
                    customization_override = ?6
                 WHERE id = ?1",
                params![
                    account.id.to_string(),
                    account.display_name,
                    optional_path_to_db(&account.avatar_path),
                    account.created_at,
                    optional_path_to_db(&account.custom_download_dir),
                    account.customization_override,
                ],
            )
            .context("failed to update account")?;

        Ok(changed > 0)
    }

    /// Removes an account by ID.
    #[allow(dead_code)]
    pub(crate) fn remove_account(&self, id: Uuid) -> Result<bool> {
        let connection = self.lock_connection()?;
        let changed = connection
            .execute(
                "DELETE FROM accounts WHERE id = ?1",
                params![id.to_string()],
            )
            .context("failed to remove account")?;

        Ok(changed > 0)
    }

    fn migrate(&self) -> Result<()> {
        let connection = self.lock_connection()?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;

                 CREATE TABLE IF NOT EXISTS schema_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    applied_at INTEGER NOT NULL DEFAULT (unixepoch())
                 );

                 CREATE TABLE IF NOT EXISTS accounts (
                    id TEXT PRIMARY KEY NOT NULL,
                    display_name TEXT NOT NULL,
                    avatar_path TEXT,
                    created_at INTEGER NOT NULL,
                    custom_download_dir TEXT,
                    customization_override INTEGER NOT NULL DEFAULT 0
                 );",
            )
            .context("failed to migrate account store")?;
        connection
            .execute(
                "INSERT OR IGNORE INTO schema_migrations (version) VALUES (?1)",
                params![MIGRATION_VERSION],
            )
            .context("failed to record account migration")?;

        Ok(())
    }

    fn lock_connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|error| anyhow::anyhow!("account store lock poisoned: {error}"))
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self> {
        let store = Self {
            connection: Mutex::new(
                Connection::open_in_memory().context("failed to open memory DB")?,
            ),
        };
        store.migrate()?;

        Ok(store)
    }
}

fn row_to_account(row: &Row<'_>) -> rusqlite::Result<Account> {
    let id_text: String = row.get("id")?;
    let id = Uuid::parse_str(&id_text).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(0, Type::Text, Box::new(error))
    })?;

    Ok(Account {
        id,
        display_name: row.get("display_name")?,
        avatar_path: optional_path_from_db(row.get("avatar_path")?),
        created_at: row.get("created_at")?,
        custom_download_dir: optional_path_from_db(row.get("custom_download_dir")?),
        customization_override: row.get("customization_override")?,
    })
}

fn optional_path_to_db(path: &Option<PathBuf>) -> Option<String> {
    path.as_ref()
        .map(|path| path.to_string_lossy().into_owned())
}

fn optional_path_from_db(path: Option<String>) -> Option<PathBuf> {
    path.map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_store_runs_migration() {
        let store = AccountStore::in_memory().unwrap();

        assert!(store.list_accounts().unwrap().is_empty());
    }

    #[test]
    fn account_store_crud_round_trip() {
        let store = AccountStore::in_memory().unwrap();
        let mut account = Account::new("Personal").unwrap();

        store.add_account(&account).unwrap();
        assert_eq!(
            store.get_account(account.id).unwrap(),
            Some(account.clone())
        );

        account.display_name = "Work".to_string();
        account.customization_override = true;
        assert!(store.update_account(&account).unwrap());

        let listed = store.list_accounts().unwrap();
        assert_eq!(listed, vec![account.clone()]);

        assert!(store.remove_account(account.id).unwrap());
        assert_eq!(store.get_account(account.id).unwrap(), None);
    }

    #[test]
    fn optional_paths_round_trip_through_db_text() {
        let path = Some(PathBuf::from("/tmp/avatar.png"));

        assert_eq!(
            optional_path_from_db(optional_path_to_db(&path)),
            Some(PathBuf::from("/tmp/avatar.png"))
        );
    }
}
