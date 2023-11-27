use crate::config::Config;
// use hex_literal::hex;
use crate::{AuthData, ErrorResponse, Response};
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::http::header::ContentDisposition;
use actix_web::{delete, get, put};
use actix_web::{error, web, HttpResponse};

use entity::book::ActiveModel as BookActiveModel;
use entity::book::Column as BookCol;

use entity::file_type::ActiveModel as FTActiveModel;
use entity::file_type::Column as FTCol;
use entity::file_type::Model as FTModel;
use entity::prelude::Book;
use entity::prelude::FileType;
// use entity::user::{self, ActiveModel, Entity};
use actix_files::NamedFile;
use hex::encode;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
#[derive(Deserialize)]
struct BookId {
    book_id: i32,
}

#[derive(Serialize)]
struct FullBook {
    pub id: i32,
    pub title: String,
    pub hash: String,
    pub user_id: i32,
    pub file_type: FTModel,
}

impl BookId {
    pub async fn get(&self, uid: i32, pool: &DatabaseConnection) -> Result<FullBook, String> {
        match Book::find_by_id(self.book_id)
            .filter(BookCol::UserId.eq(uid))
            .one(pool)
            .await
        {
            Ok(Some(book)) => {
                let ft = FileType::find_by_id(book.file_tyoe)
                    .one(pool)
                    .await
                    .unwrap()
                    .unwrap();
                Ok(FullBook {
                    file_type: ft,
                    id: book.id,
                    title: book.title,
                    hash: book.hash,
                    user_id: book.user_id,
                })
            }
            Ok(None) => Err("No such book found.".to_string()),
            Err(e) => Err(e.to_string()),
        }
    }
}
#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    book: TempFile,
}

#[get("/book/{book_id}")]
async fn book_info(
    bookid: web::Path<BookId>,
    db: web::Data<DatabaseConnection>,
    AuthData(user): AuthData,
) -> actix_web::Result<impl actix_web::Responder> {
    match bookid.get(user.id, &db).await {
        Ok(b) => Ok(HttpResponse::Ok().json(Response {
            status: "ok".to_string(),
            data: b,
        })),
        Err(e) => Err(error::ErrorNotFound(ErrorResponse {
            status: "error".to_string(),
            error: e,
        })),
    }
}

#[get("/book/{book_id}/dl")]
async fn download(
    bookid: web::Path<BookId>,
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    AuthData(user): AuthData,
) -> NamedFile {
    let book = bookid.get(user.id, &db).await.unwrap(); // we dont care just fail lol
    let fp = format!("{}/{}.bin", config.filepath, book.hash);
    NamedFile::open_async(fp)
        .await
        .unwrap()
        .set_content_disposition(ContentDisposition::attachment(format!(
            "{}.{}",
            book.title, book.file_type.name
        )))
}

#[delete("/book/{id}")]
async fn remove(
    _config: web::Data<Config>,
    _db: web::Data<DatabaseConnection>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    todo!("make it work");
    ""
}

#[get("/books")]
async fn list(
    _config: web::Data<Config>,
    _db: web::Data<DatabaseConnection>,
    AuthData(_user): AuthData,
) -> impl actix_web::Responder {
    todo!("make it work");
    ""
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
    cfg.service(upload)
        .service(download)
        .service(remove)
        .service(list)
        .service(book_info);
}
