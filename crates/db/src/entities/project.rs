use std::path::Path;

use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, QueryOrder, Set};

use crate::error::Error;
use crate::path::derive_name;

/// A project attached to a [`super::workspace::Model`]. `path` is an absolute
/// filesystem path (usually a git repository); `name` is derived from the
/// path's basename at insertion time.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub workspace_id: i64,
    #[sea_orm(unique)]
    pub path: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::workspace::Entity",
        from = "Column::WorkspaceId",
        to = "super::workspace::Column::Id",
        on_delete = "Cascade"
    )]
    Workspace,
}

impl Related<super::workspace::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Workspace.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Add a project to a workspace. `path` must be an absolute UTF-8 path;
    /// the project's display `name` is derived from the path's basename.
    pub async fn add<C: ConnectionTrait>(
        conn: &C,
        workspace_id: i64,
        path: &Path,
    ) -> Result<Self, Error> {
        let name = derive_name(path)?;
        // Safe: `derive_name` already validated UTF-8.
        let path_str = path.to_str().expect("validated UTF-8").to_string();
        let active = ActiveModel {
            workspace_id: Set(workspace_id),
            path: Set(path_str),
            name: Set(name),
            ..Default::default()
        };
        Ok(active.insert(conn).await?)
    }

    /// List all projects attached to the given workspace, ordered by name.
    pub async fn list_for_workspace<C: ConnectionTrait>(
        conn: &C,
        workspace_id: i64,
    ) -> Result<Vec<Self>, Error> {
        Ok(Entity::find()
            .filter(Column::WorkspaceId.eq(workspace_id))
            .order_by_asc(Column::Name)
            .all(conn)
            .await?)
    }

    /// Remove a project. Returns [`Error::NotFound`] if `id` doesn't match a row.
    pub async fn remove<C: ConnectionTrait>(conn: &C, id: i64) -> Result<(), Error> {
        let res = Entity::delete_by_id(id).exec(conn).await?;
        if res.rows_affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    /// Reassign a project to a different workspace. Returns
    /// [`Error::NotFound`] if `project_id` doesn't match a row.
    pub async fn move_to<C: ConnectionTrait>(
        conn: &C,
        project_id: i64,
        target_workspace_id: i64,
    ) -> Result<Self, Error> {
        let existing = Entity::find_by_id(project_id)
            .one(conn)
            .await?
            .ok_or(Error::NotFound)?;
        let mut active: ActiveModel = existing.into();
        active.workspace_id = Set(target_workspace_id);
        Ok(active.update(conn).await?)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::error::Error;
    use crate::test_support::{project_path, test_db};
    use crate::{Project, Workspace};

    #[tokio::test]
    async fn add_derives_name_from_basename() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        let path = project_path("gesttalt");
        let p = Project::add(db.connection(), ws.id, &path).await.unwrap();
        assert_eq!(p.name, "gesttalt");
        assert_eq!(p.path, path.to_str().unwrap());
        assert_eq!(p.workspace_id, ws.id);
    }

    #[tokio::test]
    async fn add_rejects_relative_path() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        let err = Project::add(db.connection(), ws.id, &PathBuf::from("relative/path"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::PathNotAbsolute(_)));
    }

    #[tokio::test]
    async fn list_for_workspace_is_scoped() {
        let db = test_db().await;
        let a = Workspace::create(db.connection(), "a").await.unwrap();
        let b = Workspace::create(db.connection(), "b").await.unwrap();
        Project::add(db.connection(), a.id, &project_path("in-a"))
            .await
            .unwrap();
        Project::add(db.connection(), b.id, &project_path("in-b"))
            .await
            .unwrap();

        let in_a = Project::list_for_workspace(db.connection(), a.id)
            .await
            .unwrap();
        assert_eq!(in_a.len(), 1);
        assert_eq!(in_a[0].name, "in-a");
    }

    #[tokio::test]
    async fn move_between_workspaces() {
        let db = test_db().await;
        let a = Workspace::create(db.connection(), "a").await.unwrap();
        let b = Workspace::create(db.connection(), "b").await.unwrap();
        let p = Project::add(db.connection(), a.id, &project_path("mover"))
            .await
            .unwrap();

        let moved = Project::move_to(db.connection(), p.id, b.id).await.unwrap();
        assert_eq!(moved.workspace_id, b.id);
        assert!(
            Project::list_for_workspace(db.connection(), a.id)
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            Project::list_for_workspace(db.connection(), b.id)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn move_missing_errors() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        let err = Project::move_to(db.connection(), 999, ws.id)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn remove_drops_project() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        let p = Project::add(db.connection(), ws.id, &project_path("doomed"))
            .await
            .unwrap();
        Project::remove(db.connection(), p.id).await.unwrap();
        assert!(
            Project::list_for_workspace(db.connection(), ws.id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn remove_missing_errors() {
        let db = test_db().await;
        let err = Project::remove(db.connection(), 999).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn duplicate_path_rejected() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        let path = project_path("dup");
        Project::add(db.connection(), ws.id, &path).await.unwrap();
        let err = Project::add(db.connection(), ws.id, &path)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::SeaOrm(_)));
    }
}
