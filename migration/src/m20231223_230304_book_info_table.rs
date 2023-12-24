use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BookInfo::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BookInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BookInfo::BookHash).string().not_null())
                    .col(ColumnDef::new(BookInfo::Title).string().not_null())
                    .col(ColumnDef::new(BookInfo::Creator).string().not_null())
                    .col(ColumnDef::new(BookInfo::CoverMime).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BookInfo::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum BookInfo {
    Table,
    Id,
    BookHash,
    Title,
    Creator,
    CoverMime,
}
