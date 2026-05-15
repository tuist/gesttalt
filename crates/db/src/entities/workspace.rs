use sea_orm::entity::prelude::*;
use sea_orm::{ConnectionTrait, QueryOrder, Set};

use crate::error::Error;

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

impl Model {
    /// Insert a new workspace.
    pub async fn create<C: ConnectionTrait>(conn: &C, name: &str) -> Result<Self, Error> {
        let active = ActiveModel {
            name: Set(name.to_string()),
            ..Default::default()
        };
        Ok(active.insert(conn).await?)
    }

    /// List all workspaces, ordered by name.
    pub async fn list<C: ConnectionTrait>(conn: &C) -> Result<Vec<Self>, Error> {
        Ok(Entity::find().order_by_asc(Column::Name).all(conn).await?)
    }

    /// Rename an existing workspace. Returns [`Error::NotFound`] if `id`
    /// doesn't match a row.
    pub async fn rename<C: ConnectionTrait>(conn: &C, id: i64, name: &str) -> Result<Self, Error> {
        let existing = Entity::find_by_id(id)
            .one(conn)
            .await?
            .ok_or(Error::NotFound)?;
        let mut active: ActiveModel = existing.into();
        active.name = Set(name.to_string());
        Ok(active.update(conn).await?)
    }

    /// Delete a workspace and all of its projects (cascading FK). Returns
    /// [`Error::NotFound`] if `id` doesn't match a row.
    pub async fn delete<C: ConnectionTrait>(conn: &C, id: i64) -> Result<(), Error> {
        let res = Entity::delete_by_id(id).exec(conn).await?;
        if res.rows_affected == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::test_support::{project_path, test_db};
    use crate::{Project, Workspace};

    #[tokio::test]
    async fn create_and_list() {
        let db = test_db().await;
        let a = Workspace::create(db.connection(), "alpha").await.unwrap();
        let b = Workspace::create(db.connection(), "beta").await.unwrap();
        assert_ne!(a.id, b.id);

        let listed = Workspace::list(db.connection()).await.unwrap();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].name, "alpha");
        assert_eq!(listed[1].name, "beta");
    }

    #[tokio::test]
    async fn rename_persists() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "old").await.unwrap();
        let renamed = Workspace::rename(db.connection(), ws.id, "new")
            .await
            .unwrap();
        assert_eq!(renamed.id, ws.id);
        assert_eq!(renamed.name, "new");
        let listed = Workspace::list(db.connection()).await.unwrap();
        assert_eq!(listed[0].name, "new");
    }

    #[tokio::test]
    async fn rename_missing_errors() {
        let db = test_db().await;
        let err = Workspace::rename(db.connection(), 999, "x")
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn delete_missing_errors() {
        let db = test_db().await;
        let err = Workspace::delete(db.connection(), 999).await.unwrap_err();
        assert!(matches!(err, Error::NotFound));
    }

    #[tokio::test]
    async fn delete_cascades_to_projects() {
        let db = test_db().await;
        let ws = Workspace::create(db.connection(), "ws").await.unwrap();
        Project::add(db.connection(), ws.id, &project_path("proj-a"))
            .await
            .unwrap();
        Project::add(db.connection(), ws.id, &project_path("proj-b"))
            .await
            .unwrap();

        Workspace::delete(db.connection(), ws.id).await.unwrap();
        let projects = Project::list_for_workspace(db.connection(), ws.id)
            .await
            .unwrap();
        assert!(projects.is_empty());
    }
}
