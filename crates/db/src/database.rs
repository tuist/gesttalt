use std::path::Path;

use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use crate::error::Error;
use crate::project::{Project, derive_name};
use crate::workspace::Workspace;

/// Async handle to the gesttalt SQLite database. Cheap to clone (wraps a pool).
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open (or create) a database at the given filesystem path. Runs any
    /// pending migrations on connect.
    pub async fn open(path: &Path) -> Result<Self, Error> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true);
        Self::connect_with(SqlitePoolOptions::new(), options).await
    }

    /// Open an ephemeral in-memory database. Primarily for tests.
    pub async fn in_memory() -> Result<Self, Error> {
        let options = SqliteConnectOptions::new()
            .in_memory(true)
            .foreign_keys(true);
        // Each connection to `:memory:` gets its own private database, so the
        // pool must hold exactly one connection or migrations and queries land
        // on different DBs.
        let pool_options = SqlitePoolOptions::new().max_connections(1);
        Self::connect_with(pool_options, options).await
    }

    async fn connect_with(
        pool_options: SqlitePoolOptions,
        connect_options: SqliteConnectOptions,
    ) -> Result<Self, Error> {
        let pool = pool_options.connect_with(connect_options).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    /// Borrow the underlying pool. Useful for ad-hoc queries from callers that
    /// need to compose with this crate's API.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // ---- Workspaces ----

    pub async fn create_workspace(&self, name: &str) -> Result<Workspace, Error> {
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO workspaces (name) VALUES (?) RETURNING id")
                .bind(name)
                .fetch_one(&self.pool)
                .await?;
        Ok(Workspace {
            id,
            name: name.to_string(),
        })
    }

    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>, Error> {
        let rows = sqlx::query_as::<_, Workspace>(
            "SELECT id, name FROM workspaces ORDER BY name COLLATE NOCASE",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn rename_workspace(&self, id: i64, name: &str) -> Result<Workspace, Error> {
        let affected = sqlx::query("UPDATE workspaces SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(Workspace {
            id,
            name: name.to_string(),
        })
    }

    pub async fn delete_workspace(&self, id: i64) -> Result<(), Error> {
        let affected = sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    // ---- Projects ----

    pub async fn add_project(&self, workspace_id: i64, path: &Path) -> Result<Project, Error> {
        let name = derive_name(path)?;
        // Safe: `derive_name` already validated UTF-8.
        let path_str = path.to_str().expect("validated UTF-8").to_string();
        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO projects (workspace_id, path, name) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(workspace_id)
        .bind(&path_str)
        .bind(&name)
        .fetch_one(&self.pool)
        .await?;
        Ok(Project {
            id,
            workspace_id,
            path: path_str,
            name,
        })
    }

    pub async fn list_projects(&self, workspace_id: i64) -> Result<Vec<Project>, Error> {
        let rows = sqlx::query_as::<_, Project>(
            "SELECT id, workspace_id, path, name FROM projects \
             WHERE workspace_id = ? ORDER BY name COLLATE NOCASE",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    pub async fn remove_project(&self, id: i64) -> Result<(), Error> {
        let affected = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    pub async fn move_project(
        &self,
        project_id: i64,
        target_workspace_id: i64,
    ) -> Result<Project, Error> {
        let row = sqlx::query_as::<_, Project>(
            "UPDATE projects SET workspace_id = ? WHERE id = ? \
             RETURNING id, workspace_id, path, name",
        )
        .bind(target_workspace_id)
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        row.ok_or(Error::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    async fn db() -> Database {
        Database::in_memory().await.expect("open in-memory db")
    }

    #[tokio::test]
    async fn create_and_list_workspaces() {
        let db = db().await;
        let a = db.create_workspace("alpha").await.unwrap();
        let b = db.create_workspace("beta").await.unwrap();
        assert_ne!(a.id, b.id);

        let listed = db.list_workspaces().await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "alpha");
        assert_eq!(listed[1].name, "beta");
    }

    #[tokio::test]
    async fn rename_workspace_persists() {
        let db = db().await;
        let ws = db.create_workspace("old").await.unwrap();
        let renamed = db.rename_workspace(ws.id, "new").await.unwrap();
        assert_eq!(renamed.name, "new");
        let listed = db.list_workspaces().await.unwrap();
        assert_eq!(listed[0].name, "new");
    }

    #[tokio::test]
    async fn rename_missing_workspace_errors() {
        let db = db().await;
        let err = db.rename_workspace(999, "x").await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn delete_workspace_cascades_to_projects() {
        let db = db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        db.add_project(ws.id, &PathBuf::from("/tmp/proj-a"))
            .await
            .unwrap();
        db.add_project(ws.id, &PathBuf::from("/tmp/proj-b"))
            .await
            .unwrap();

        db.delete_workspace(ws.id).await.unwrap();
        let projects = db.list_projects(ws.id).await.unwrap();
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn add_project_derives_name_from_basename() {
        let db = db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let p = db
            .add_project(ws.id, &PathBuf::from("/Users/me/code/gesttalt"))
            .await
            .unwrap();
        assert_eq!(p.name, "gesttalt");
        assert_eq!(p.path, "/Users/me/code/gesttalt");
        assert_eq!(p.workspace_id, ws.id);
    }

    #[tokio::test]
    async fn add_project_rejects_relative_path() {
        let db = db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let err = db
            .add_project(ws.id, &PathBuf::from("relative/path"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::PathNotAbsolute(_)));
    }

    #[tokio::test]
    async fn list_projects_scoped_to_workspace() {
        let db = db().await;
        let a = db.create_workspace("a").await.unwrap();
        let b = db.create_workspace("b").await.unwrap();
        db.add_project(a.id, &PathBuf::from("/tmp/in-a")).await.unwrap();
        db.add_project(b.id, &PathBuf::from("/tmp/in-b")).await.unwrap();

        let in_a = db.list_projects(a.id).await.unwrap();
        assert_eq!(in_a.len(), 1);
        assert_eq!(in_a[0].name, "in-a");
    }

    #[tokio::test]
    async fn move_project_between_workspaces() {
        let db = db().await;
        let a = db.create_workspace("a").await.unwrap();
        let b = db.create_workspace("b").await.unwrap();
        let p = db
            .add_project(a.id, &PathBuf::from("/tmp/mover"))
            .await
            .unwrap();

        let moved = db.move_project(p.id, b.id).await.unwrap();
        assert_eq!(moved.workspace_id, b.id);
        assert!(db.list_projects(a.id).await.unwrap().is_empty());
        assert_eq!(db.list_projects(b.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn remove_project_drops_it() {
        let db = db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let p = db
            .add_project(ws.id, &PathBuf::from("/tmp/doomed"))
            .await
            .unwrap();
        db.remove_project(p.id).await.unwrap();
        assert!(db.list_projects(ws.id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn duplicate_project_path_rejected() {
        let db = db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        db.add_project(ws.id, &PathBuf::from("/tmp/dup")).await.unwrap();
        let err = db
            .add_project(ws.id, &PathBuf::from("/tmp/dup"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Sqlx(_)));
    }
}
