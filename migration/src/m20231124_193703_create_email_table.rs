use super::m20220101_000001_create_user_table::User;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Email::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Email::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Email::Email).string().not_null())
                    .col(ColumnDef::new(Email::UserId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-post-user_id")
                            .from(Email::Table, Email::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Email::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Email {
    Table,
    Id,
    Email,
    UserId,
}
