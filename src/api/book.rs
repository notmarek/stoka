use crate::config::Config;
// use hex_literal::hex;
use crate::AuthData;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::put;
use actix_web::web;
use entity::book::ActiveModel as BookActiveModel;
use entity::file_type::ActiveModel as FTActiveModel;
use entity::file_type::Column as FTCol;
use entity::prelude::Book;
use entity::prelude::FileType;
// use entity::user::{self, ActiveModel, Entity};
use hex::encode;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    book: TempFile,
}

#[put("/book")]
async fn upload(
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    AuthData(user): AuthData,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl actix_web::Responder {
    let db: &DatabaseConnection = &db;
    let mut book: TempFile = form.book;
    let mut hasher = Sha256::new();
    let mut buf: Vec<u8> = vec![];
    match book.file.read_to_end(&mut buf) {
        Ok(_size) => hasher.update(buf.clone()),
        Err(e) => return e.to_string(),
    }
    let hash = encode(hasher.finalize());
    let new_path = format!("{}/{}.bin", config.filepath, hash);
    match std::fs::metadata(&new_path) {
        Ok(_) => println!("File already exists!"),
        Err(_) => std::fs::File::create(&new_path)
            .unwrap()
            .write_all(&buf)
            .unwrap(),
    };
    let filename_string = book.file_name.unwrap_or("unk.epub".to_string());
    let filename = Path::new(&filename_string);

    let mut extension = ".epub".to_string();
    if let Some(ext) = filename.extension() {
        extension = ext.to_string_lossy().to_string();
    }

    let mut title = "unk".to_string();
    if let Some(name) = filename.file_stem() {
        title = name.to_string_lossy().to_string();
    }
    println!("{:#?}", book.file);

    let ft_id = match FileType::find()
        .filter(FTCol::Name.eq(extension.to_lowercase()))
        .one(db)
        .await
    {
        Ok(Some(ft)) => ft.id,
        Ok(None) => match FileType::insert(FTActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(extension.to_lowercase()),
        })
        .exec(db)
        .await
        {
            Ok(res) => res.last_insert_id,
            Err(_) => 0,
        },
        Err(_) => 0,
    };

    let new_book = BookActiveModel {
        id: ActiveValue::NotSet,
        title: ActiveValue::Set(title),
        hash: ActiveValue::Set(hash),
        user_id: ActiveValue::Set(user.id),
        file_tyoe: ActiveValue::Set(ft_id),
    };

    Book::insert(new_book).exec(db).await.unwrap();
    "ok".to_string()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload);
}
