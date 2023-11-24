pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_user_table;
mod m20231124_193135_create_book_table;
mod m20231124_193703_create_email_table;
mod m20231124_194004_create_filetype_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user_table::Migration),
            Box::new(m20231124_193135_create_book_table::Migration),
            Box::new(m20231124_193703_create_email_table::Migration),
            Box::new(m20231124_194004_create_filetype_table::Migration),
        ]
    }
}
