use super::m20220101_000001_create_user_table::User;
use super::m20231124_193703_create_email_table::Email;
use super::m20231124_194004_create_filetype_table::FileType;
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Book::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Book::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Book::Title).string().not_null())
                    .col(ColumnDef::new(Book::Hash).string().not_null())
                    .col(ColumnDef::new(Book::UserId).integer().not_null())
                    .col(ColumnDef::new(Book::FileTyoe).integer().not_null())
                    .col(ColumnDef::new(Book::EmailId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book-user_id")
                            .from(Book::Table, Book::UserId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book-email_id")
                            .from(Book::Table, Book::EmailId)
                            .to(Email::Table, Email::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-book-filetype")
                            .from(Book::Table, Book::FileTyoe)
                            .to(FileType::Table, FileType::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Book::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Book {
    Table,
    Id,
    Title,
    Hash,
    UserId,
    FileTyoe,
    EmailId,
}
