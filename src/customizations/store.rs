//! Customization persistence.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use rusqlite::types::Type;
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

use super::{CustomizationEntry, CustomizationKind, CustomizationSource};

const MIGRATION_VERSION: i64 = 1;

/// SQLite-backed customization store.
pub(crate) struct CustomizationStore {
    connection: Mutex<Connection>,
}

impl CustomizationStore {
    /// Opens a customization store and runs migrations.
    pub(crate) fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create customization DB dir {}", parent.display())
            })?;
        }

        let connection = Connection::open(path)
            .with_context(|| format!("failed to open customization DB {}", path.display()))?;
        let store = Self {
            connection: Mutex::new(connection),
        };
        store.migrate()?;

        Ok(store)
    }

    /// Lists enabled global entries followed by account-specific entries.
    pub(crate) fn enabled_for_account(
        &self,
        account_id: Option<Uuid>,
    ) -> Result<Vec<CustomizationEntry>> {
        let connection = self.lock_connection()?;
        let mut entries = query_entries(&connection, None)?;

        if let Some(account_id) = account_id {
            entries.extend(query_entries(&connection, Some(account_id))?);
        }

        Ok(entries)
    }

    /// Inserts or replaces customization metadata.
    #[allow(dead_code)]
    pub(crate) fn upsert(&self, entry: &CustomizationEntry) -> Result<()> {
        let connection = self.lock_connection()?;
        connection
            .execute(
                "INSERT INTO customizations (
                    id, account_id, path, kind, enabled, source, url, etag
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                ON CONFLICT(id) DO UPDATE SET
                    account_id = excluded.account_id,
                    path = excluded.path,
                    kind = excluded.kind,
                    enabled = excluded.enabled,
                    source = excluded.source,
                    url = excluded.url,
                    etag = excluded.etag",
                params![
                    entry.id.to_string(),
                    entry.account_id.map(|id| id.to_string()),
                    entry.path.to_string_lossy(),
                    kind_to_db(entry.kind),
                    entry.enabled,
                    source_to_db(entry.source),
                    entry.url,
                    entry.etag,
                ],
            )
            .context("failed to upsert customization")?;

        Ok(())
    }

    /// Enables or disables a customization entry.
    #[allow(dead_code)]
    pub(crate) fn set_enabled(&self, id: Uuid, enabled: bool) -> Result<bool> {
        let connection = self.lock_connection()?;
        let changed = connection
            .execute(
                "UPDATE customizations SET enabled = ?2 WHERE id = ?1",
                params![id.to_string(), enabled],
            )
            .context("failed to update customization enabled flag")?;

        Ok(changed > 0)
    }

    fn migrate(&self) -> Result<()> {
        let connection = self.lock_connection()?;
        connection
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS customization_migrations (
                    version INTEGER PRIMARY KEY NOT NULL,
                    applied_at INTEGER NOT NULL DEFAULT (unixepoch())
                 );

                 CREATE TABLE IF NOT EXISTS customizations (
                    id TEXT PRIMARY KEY NOT NULL,
                    account_id TEXT,
                    path TEXT NOT NULL,
                    kind TEXT NOT NULL CHECK (kind IN ('css', 'js')),
                    enabled INTEGER NOT NULL DEFAULT 1,
                    source TEXT NOT NULL CHECK (source IN ('file', 'url')),
                    url TEXT,
                    etag TEXT
                 );",
            )
            .context("failed to migrate customization store")?;
        connection
            .execute(
                "INSERT OR IGNORE INTO customization_migrations (version) VALUES (?1)",
                params![MIGRATION_VERSION],
            )
            .context("failed to record customization migration")?;

        Ok(())
    }

    fn lock_connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>> {
        self.connection
            .lock()
            .map_err(|error| anyhow::anyhow!("customization store lock poisoned: {error}"))
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

fn query_entries(
    connection: &Connection,
    account_id: Option<Uuid>,
) -> Result<Vec<CustomizationEntry>> {
    let (sql, account_id_text) = match account_id {
        Some(account_id) => (
            "SELECT id, account_id, path, kind, enabled, source, url, etag
             FROM customizations WHERE enabled = 1 AND account_id = ?1 ORDER BY path ASC",
            account_id.to_string(),
        ),
        None => (
            "SELECT id, account_id, path, kind, enabled, source, url, etag
             FROM customizations WHERE enabled = 1 AND account_id IS NULL ORDER BY path ASC",
            String::new(),
        ),
    };
    let mut statement = connection
        .prepare(sql)
        .context("failed to prepare customization query")?;
    let entries = if account_id.is_some() {
        statement
            .query_map(params![account_id_text], row_to_entry)
            .context("failed to query account customizations")?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read account customization rows")?
    } else {
        statement
            .query_map([], row_to_entry)
            .context("failed to query global customizations")?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read global customization rows")?
    };

    Ok(entries)
}

fn row_to_entry(row: &Row<'_>) -> rusqlite::Result<CustomizationEntry> {
    let id_text: String = row.get("id")?;
    let id = parse_uuid_column(&id_text, 0)?;
    let account_id_text: Option<String> = row.get("account_id")?;
    let account_id = account_id_text
        .as_deref()
        .map(|value| parse_uuid_column(value, 1))
        .transpose()?;

    Ok(CustomizationEntry {
        id,
        account_id,
        path: PathBuf::from(row.get::<_, String>("path")?),
        kind: kind_from_db(row.get::<_, String>("kind")?.as_str()),
        enabled: row.get("enabled")?,
        source: source_from_db(row.get::<_, String>("source")?.as_str()),
        url: row.get("url")?,
        etag: row.get("etag")?,
    })
}

fn parse_uuid_column(value: &str, column: usize) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
    })
}

fn kind_to_db(kind: CustomizationKind) -> &'static str {
    match kind {
        CustomizationKind::Css => "css",
        CustomizationKind::Js => "js",
    }
}

fn kind_from_db(kind: &str) -> CustomizationKind {
    match kind {
        "js" => CustomizationKind::Js,
        _ => CustomizationKind::Css,
    }
}

fn source_to_db(source: CustomizationSource) -> &'static str {
    match source {
        CustomizationSource::File => "file",
        CustomizationSource::Url => "url",
    }
}

fn source_from_db(source: &str) -> CustomizationSource {
    match source {
        "url" => CustomizationSource::Url,
        _ => CustomizationSource::File,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_round_trips_enabled_entry() {
        let store = CustomizationStore::in_memory().unwrap();
        let entry = CustomizationEntry {
            id: Uuid::new_v4(),
            account_id: None,
            path: PathBuf::from("/tmp/a.css"),
            kind: CustomizationKind::Css,
            enabled: true,
            source: CustomizationSource::File,
            url: None,
            etag: None,
        };

        store.upsert(&entry).unwrap();

        assert_eq!(store.enabled_for_account(None).unwrap(), vec![entry]);
    }
}
