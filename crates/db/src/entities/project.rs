use sea_orm::entity::prelude::*;

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::error::Error;
    use crate::test_support::test_db;

    #[tokio::test]
    async fn add_derives_name_from_basename() {
        let db = test_db().await;
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
    async fn add_rejects_relative_path() {
        let db = test_db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let err = db
            .add_project(ws.id, &PathBuf::from("relative/path"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::PathNotAbsolute(_)));
    }

    #[tokio::test]
    async fn list_is_scoped_to_workspace() {
        let db = test_db().await;
        let a = db.create_workspace("a").await.unwrap();
        let b = db.create_workspace("b").await.unwrap();
        db.add_project(a.id, &PathBuf::from("/tmp/in-a"))
            .await
            .unwrap();
        db.add_project(b.id, &PathBuf::from("/tmp/in-b"))
            .await
            .unwrap();

        let in_a = db.list_projects(a.id).await.unwrap();
        assert_eq!(in_a.len(), 1);
        assert_eq!(in_a[0].name, "in-a");
    }

    #[tokio::test]
    async fn move_between_workspaces() {
        let db = test_db().await;
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
    async fn move_missing_errors() {
        let db = test_db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let err = db.move_project(999, ws.id).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn remove_drops_project() {
        let db = test_db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        let p = db
            .add_project(ws.id, &PathBuf::from("/tmp/doomed"))
            .await
            .unwrap();
        db.remove_project(p.id).await.unwrap();
        assert!(db.list_projects(ws.id).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn remove_missing_errors() {
        let db = test_db().await;
        let err = db.remove_project(999).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn duplicate_path_rejected() {
        let db = test_db().await;
        let ws = db.create_workspace("ws").await.unwrap();
        db.add_project(ws.id, &PathBuf::from("/tmp/dup"))
            .await
            .unwrap();
        let err = db
            .add_project(ws.id, &PathBuf::from("/tmp/dup"))
            .await
            .unwrap_err();
        assert!(matches!(err, Error::SeaOrm(_)));
    }
}
