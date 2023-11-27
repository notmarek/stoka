use crate::auth::Claims;
use crate::config::Config;
use argon2::{self, hash_encoded, verify_encoded, Config as ArgonConf, Variant, Version};
// use hex_literal::hex;
use crate::AuthData;
use crate::ErrorResponse;
use entity::book::Model as BookModel;
use entity::file_type::Model as FTModel;
use actix_multipart::{
    form::{
        tempfile::{TempFile, TempFileConfig},
        MultipartForm,
    },
    Multipart,
};
use actix_web::{get, post, put};
use actix_web::{web, HttpResponse};
use entity::user::{self, ActiveModel, Entity};
use hex::encode;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::io::{self, Read, Write};
use std::path::Path;

pub fn hash_password(password: String, salt: String) -> String {
    let config = ArgonConf {
        variant: Variant::Argon2id,
        version: Version::Version13,
        mem_cost: 65536,
        time_cost: 4,
        lanes: 4,
        secret: &[],
        ad: &[],
        hash_length: 32,
    };
    hash_encoded(password.as_bytes(), salt.as_bytes(), &config).unwrap()
}

#[derive(Debug, MultipartForm)]
struct UploadForm {
    #[multipart(rename = "file")]
    book: TempFile,
}

#[put("/book")]
async fn upload(
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl actix_web::Responder {
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
        Err(_) => std::fs::File::create(&new_path).unwrap().write_all(&buf).unwrap()
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

    let ft = FTModel {
        id:0,
        name: extension.to_lowercase(),
    };
    let book = BookModel {   
        id:0,
        title,
        hash,
        user_id: 0,
        file_tyoe: 0,
        email_id: 0,        
    };
    // todo save to db :)
    format!("{:#?}. {:#?}", book, ft)
}


pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(upload);
}
