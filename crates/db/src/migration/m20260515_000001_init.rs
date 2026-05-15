use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Workspaces::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Workspaces::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Workspaces::Name).text().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Projects::WorkspaceId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Projects::Path)
                            .text()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_projects_workspace_id")
                            .from(Projects::Table, Projects::WorkspaceId)
                            .to(Workspaces::Table, Workspaces::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_projects_workspace_id")
                    .table(Projects::Table)
                    .col(Projects::WorkspaceId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Workspaces::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Workspaces {
    Table,
    Id,
    Name,
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    WorkspaceId,
    Path,
    Name,
}

#[cfg(test)]
mod tests {
    use sea_orm::{
        ConnectOptions, ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, Statement,
    };
    use sea_orm_migration::MigratorTrait;

    use crate::migration::Migrator;

    /// A raw SeaORM connection to a fresh `:memory:` DB with no migrations
    /// applied. Used to drive `Migrator::up`/`down` directly.
    async fn raw_db() -> DatabaseConnection {
        let mut opts = ConnectOptions::new("sqlite::memory:".to_string());
        opts.max_connections(1).sqlx_logging(false);
        Database::connect(opts)
            .await
            .expect("connect to in-memory sqlite")
    }

    async fn schema_object_exists(conn: &DatabaseConnection, kind: &str, name: &str) -> bool {
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master WHERE type = ? AND name = ?",
            [kind.into(), name.into()],
        );
        conn.query_one(stmt)
            .await
            .expect("query sqlite_master")
            .is_some()
    }

    #[tokio::test]
    async fn up_creates_workspaces_and_projects_tables() {
        let conn = raw_db().await;
        Migrator::up(&conn, None).await.unwrap();
        assert!(schema_object_exists(&conn, "table", "workspaces").await);
        assert!(schema_object_exists(&conn, "table", "projects").await);
    }

    #[tokio::test]
    async fn up_creates_projects_workspace_id_index() {
        let conn = raw_db().await;
        Migrator::up(&conn, None).await.unwrap();
        assert!(schema_object_exists(&conn, "index", "idx_projects_workspace_id").await);
    }

    #[tokio::test]
    async fn up_is_idempotent() {
        let conn = raw_db().await;
        Migrator::up(&conn, None).await.unwrap();
        // Running again should be a no-op: SeaORM skips already-applied migrations.
        Migrator::up(&conn, None).await.unwrap();
    }

    #[tokio::test]
    async fn down_drops_tables() {
        let conn = raw_db().await;
        Migrator::up(&conn, None).await.unwrap();
        Migrator::down(&conn, None).await.unwrap();
        assert!(!schema_object_exists(&conn, "table", "workspaces").await);
        assert!(!schema_object_exists(&conn, "table", "projects").await);
    }

    #[tokio::test]
    async fn up_down_up_round_trips() {
        let conn = raw_db().await;
        Migrator::up(&conn, None).await.unwrap();
        Migrator::down(&conn, None).await.unwrap();
        Migrator::up(&conn, None).await.unwrap();
        assert!(schema_object_exists(&conn, "table", "workspaces").await);
        assert!(schema_object_exists(&conn, "table", "projects").await);
    }
}
