use std::path::Path;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, ConnectionTrait, Database as SeaDatabase,
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use sea_orm_migration::MigratorTrait;

use crate::entities::{project, workspace};
use crate::error::Error;
use crate::migration::Migrator;
use crate::path::derive_name;

/// Async handle to the gesttalt SQLite database. Cheap to clone (wraps a
/// SeaORM `DatabaseConnection`, which is internally an `Arc` over a pool).
#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    /// Open (or create) a database at the given filesystem path. Runs any
    /// pending migrations on connect.
    pub async fn open(path: &Path) -> Result<Self, Error> {
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

    /// Borrow the underlying connection for ad-hoc queries from callers that
    /// need to compose with this crate's API.
    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    // ---- Workspaces ----

    pub async fn create_workspace(&self, name: &str) -> Result<crate::Workspace, Error> {
        let active = workspace::ActiveModel {
            name: Set(name.to_string()),
            ..Default::default()
        };
        Ok(active.insert(&self.conn).await?)
    }

    pub async fn list_workspaces(&self) -> Result<Vec<crate::Workspace>, Error> {
        Ok(workspace::Entity::find()
            .order_by_asc(workspace::Column::Name)
            .all(&self.conn)
            .await?)
    }

    pub async fn rename_workspace(
        &self,
        id: i64,
        name: &str,
    ) -> Result<crate::Workspace, Error> {
        let existing = workspace::Entity::find_by_id(id)
            .one(&self.conn)
            .await?
            .ok_or(Error::NotFound)?;
        let mut active: workspace::ActiveModel = existing.into();
        active.name = Set(name.to_string());
        Ok(active.update(&self.conn).await?)
    }

    pub async fn delete_workspace(&self, id: i64) -> Result<(), Error> {
        let res = workspace::Entity::delete_by_id(id).exec(&self.conn).await?;
        if res.rows_affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    // ---- Projects ----

    pub async fn add_project(
        &self,
        workspace_id: i64,
        path: &Path,
    ) -> Result<crate::Project, Error> {
        let name = derive_name(path)?;
        // Safe: `derive_name` already validated UTF-8.
        let path_str = path.to_str().expect("validated UTF-8").to_string();
        let active = project::ActiveModel {
            workspace_id: Set(workspace_id),
            path: Set(path_str),
            name: Set(name),
            ..Default::default()
        };
        Ok(active.insert(&self.conn).await?)
    }

    pub async fn list_projects(&self, workspace_id: i64) -> Result<Vec<crate::Project>, Error> {
        Ok(project::Entity::find()
            .filter(project::Column::WorkspaceId.eq(workspace_id))
            .order_by_asc(project::Column::Name)
            .all(&self.conn)
            .await?)
    }

    pub async fn remove_project(&self, id: i64) -> Result<(), Error> {
        let res = project::Entity::delete_by_id(id).exec(&self.conn).await?;
        if res.rows_affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    pub async fn move_project(
        &self,
        project_id: i64,
        target_workspace_id: i64,
    ) -> Result<crate::Project, Error> {
        let existing = project::Entity::find_by_id(project_id)
            .one(&self.conn)
            .await?
            .ok_or(Error::NotFound)?;
        let mut active: project::ActiveModel = existing.into();
        active.workspace_id = Set(target_workspace_id);
        Ok(active.update(&self.conn).await?)
    }
}
