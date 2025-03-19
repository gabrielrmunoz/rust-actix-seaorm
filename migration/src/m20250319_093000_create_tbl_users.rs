use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblUsers::Table)
                    .if_not_exists()
                    .col(pk_auto(TblUsers::Id))
                    .col(string(TblUsers::Username).not_null())
                    .col(string(TblUsers::FirstName).null())
                    .col(string(TblUsers::LastName).null())
                    .col(string(TblUsers::Email).not_null())
                    .col(string(TblUsers::Phone).null())
                    .col(timestamp(TblUsers::CreatedOn).not_null())
                    .col(timestamp(TblUsers::UpdatedOn).not_null())
                    .col(timestamp(TblUsers::DeletedOn).null())
                    .index(
                        Index::create()
                            .name("idx_username")
                            .col(TblUsers::Username)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("idx_email")
                            .col(TblUsers::Email)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum TblUsers {
    Table,
    Id,
    Username,
    FirstName,
    LastName,
    Email,
    Phone,
    CreatedOn,
    UpdatedOn,
    DeletedOn,
}
