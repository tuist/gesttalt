use sea_orm::entity::prelude::*;

/// A named grouping of [`super::project::Model`]s.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "workspaces")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project::Entity")]
    Project,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::error::Error;
    use crate::test_support::test_db;

    #[tokio::test]
    async fn create_and_list_workspaces() {
        let db = test_db().await;
        let a = db.create_workspace("alpha").await.unwrap();
        let b = db.create_workspace("beta").await.unwrap();
        assert_ne!(a.id, b.id);

        let listed = db.list_workspaces().await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "alpha");
        assert_eq!(listed[1].name, "beta");
    }

    #[tokio::test]
    async fn rename_persists() {
        let db = test_db().await;
        let ws = db.create_workspace("old").await.unwrap();
        let renamed = db.rename_workspace(ws.id, "new").await.unwrap();
        assert_eq!(renamed.id, ws.id);
        assert_eq!(renamed.name, "new");
        let listed = db.list_workspaces().await.unwrap();
        assert_eq!(listed[0].name, "new");
    }

    #[tokio::test]
    async fn rename_missing_errors() {
        let db = test_db().await;
        let err = db.rename_workspace(999, "x").await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn delete_missing_errors() {
        let db = test_db().await;
        let err = db.delete_workspace(999).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn delete_cascades_to_projects() {
        let db = test_db().await;
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
}
