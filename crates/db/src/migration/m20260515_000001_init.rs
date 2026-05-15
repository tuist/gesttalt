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
                    .col(ColumnDef::new(Projects::WorkspaceId).big_integer().not_null())
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
