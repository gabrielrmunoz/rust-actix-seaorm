use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TblRefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TblRefreshTokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TblRefreshTokens::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblRefreshTokens::RefreshToken)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(TblRefreshTokens::CreatedOn)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblRefreshTokens::ExpiresOn)
                            .timestamp()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TblRefreshTokens::RevokedOn)
                            .timestamp()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_refresh_token_user")
                            .from(TblRefreshTokens::Table, TblRefreshTokens::UserId)
                            .to(TblUsers::Table, TblUsers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TblRefreshTokens::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TblRefreshTokens {
    Table,
    Id,
    UserId,
    RefreshToken,
    CreatedOn,
    ExpiresOn,
    RevokedOn,
}

#[derive(DeriveIden)]
enum TblUsers {
    Table,
    Id,
}
