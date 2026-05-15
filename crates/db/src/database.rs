use std::path::{Path, PathBuf};

use sea_orm::{ConnectOptions, ConnectionTrait, Database as SeaDatabase, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

use crate::error::Error;
use crate::migration::Migrator;

/// Async handle to the gesttalt SQLite database. Cheap to clone (wraps a
/// SeaORM `DatabaseConnection`, which is internally an `Arc` over a pool).
///
/// Per-entity queries live on the entity types themselves
/// (e.g. [`crate::Workspace::create`], [`crate::Project::add`]); this struct
/// only owns the connection lifecycle.
#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    /// The default filesystem location for the gesttalt database, following
    /// the platform's data-directory convention:
    /// - Linux: `$XDG_DATA_HOME/gesttalt/gesttalt.db`
    ///   (defaults to `~/.local/share/gesttalt/gesttalt.db`)
    /// - macOS: `~/Library/Application Support/gesttalt/gesttalt.db`
    /// - Windows: `%APPDATA%\gesttalt\gesttalt.db`
    pub fn default_path() -> Result<PathBuf, Error> {
        let data = dirs::data_dir().ok_or(Error::NoDataDir)?;
        Ok(data.join("gesttalt").join("gesttalt.db"))
    }

    /// Open (or create) the database at [`Self::default_path`].
    pub async fn open_default() -> Result<Self, Error> {
        Self::open(&Self::default_path()?).await
    }

    /// Open (or create) a database at the given filesystem path. Any missing
    /// parent directories are created. Runs pending migrations on connect.
    pub async fn open(path: &Path) -> Result<Self, Error> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let url = format!("sqlite://{}?mode=rwc", path.display());
        Self::connect(url).await
    }

    /// Open an ephemeral in-memory database. Primarily for tests.
    pub async fn in_memory() -> Result<Self, Error> {
        Self::connect("sqlite::memory:".to_string()).await
    }

    async fn connect(url: String) -> Result<Self, Error> {
        // SQLite serializes writes through a single global lock, so a desktop
        // app gains nothing from a multi-connection pool — and using one would
        // require running `PRAGMA foreign_keys = ON` per connection. Holding a
        // single connection sidesteps that and also makes `:memory:` usable
        // (each connection there gets its own private database).
        let mut opts = ConnectOptions::new(url);
        opts.max_connections(1).sqlx_logging(false);
        let conn = SeaDatabase::connect(opts).await?;
        conn.execute_unprepared("PRAGMA foreign_keys = ON").await?;
        Migrator::up(&conn, None).await?;
        Ok(Self { conn })
    }

    /// Borrow the underlying connection. Pass this to entity methods like
    /// [`crate::Workspace::create`] or to compose ad-hoc SeaORM queries.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_path_lives_under_os_data_dir() {
        let path = Database::default_path().expect("data dir available on this platform");
        let data = dirs::data_dir().expect("data dir available on this platform");
        assert!(path.starts_with(&data), "expected {path:?} under {data:?}");
        assert_eq!(path.file_name().unwrap(), "gesttalt.db");
        assert_eq!(path.parent().unwrap().file_name().unwrap(), "gesttalt");
    }

    #[tokio::test]
    async fn open_creates_missing_parent_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nested/created/gesttalt.db");
        let _db = Database::open(&path).await.unwrap();
        assert!(path.exists());
        assert!(path.parent().unwrap().is_dir());
    }
}
