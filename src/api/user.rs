use crate::auth::Claims;
use crate::config::Config;
use argon2::{self, hash_encoded, verify_encoded, Config as ArgonConf, Variant, Version};

use crate::AuthData;
use crate::ErrorResponse;
use actix_web::{get, post, put};
use actix_web::{web, HttpResponse};
use entity::user::{self, ActiveModel, Entity};
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
#[derive(ToSchema, Serialize, Deserialize)]
pub struct Tokens {
    status: String,
    token_type: String,
    token: String,
    refresh_token: String,
    expiration: i64,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct UserRequest {
    pub username: Option<String>,
    pub password: Option<String>,
    pub refresh_token: Option<String>,
    pub identifier: String,
    pub captcha: Option<String>,     // TODO: captcha integration
    pub invite: Option<String>,      // TODO: invite system
    pub newpassword: Option<String>, // TODO: password changing
    pub private: Option<String>,     // TODO: account privacy settings
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct EditUser {
    username: Option<String>,
    password: Option<String>,
    permissions: Option<String>,
}

#[derive(ToSchema, Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LimitQuery {
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[put("/user")]
async fn register(
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    req_data: web::Json<RegisterRequest>,
) -> impl actix_web::Responder {
    let user = ActiveModel {
        id: ActiveValue::NotSet,
        username: ActiveValue::Set(req_data.username.clone()),
        password: ActiveValue::Set(hash_password(
            req_data.password.clone(),
            "very7898952salty:)".to_string(), // Todo: make the salt reanomd
        )),
        admin: ActiveValue::Set(false),
    };
    let con: &DatabaseConnection = &db;

    match Entity::insert(user).exec(con).await {
        Ok(u) => {
            let claims = Claims::new(u.last_insert_id, config.jwt.valid_for);
            HttpResponse::Ok().json(Tokens {
                status: "ok".to_string(),
                token_type: "Bearer".to_string(),
                token: claims.create_token(&config.jwt.private_key).unwrap(),
                refresh_token: claims
                    .create_refresh_token(&config.jwt.private_key)
                    .unwrap(),
                expiration: claims.exp,
            })
        }
        Err(e) => HttpResponse::NotAcceptable().json(ErrorResponse {
            status: "error".to_string(),
            error: e.to_string(),
        }),
    }
}

#[post("/user")]
async fn login(
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
    req_data: web::Json<UserRequest>,
) -> impl actix_web::Responder {
    let con: &DatabaseConnection = &db;
    match req_data.identifier.as_str() {
        "password" => {
            match Entity::find()
                .filter(user::Column::Username.eq(req_data.username.clone().unwrap()))
                .one(con)
                .await
            {
                Ok(Some(u)) => {
                    match verify_encoded(&u.password, req_data.password.clone().unwrap().as_bytes())
                        .unwrap()
                    {
                        true => {
                            let claims = Claims::new(u.id, config.jwt.valid_for);
                            HttpResponse::Ok().json(Tokens {
                                status: "ok".to_string(),
                                token_type: "Bearer".to_string(),
                                token: claims.create_token(&config.jwt.private_key).unwrap(),
                                refresh_token: claims
                                    .create_refresh_token(&config.jwt.private_key)
                                    .unwrap(),
                                expiration: claims.exp,
                            })
                        }
                        false => HttpResponse::Unauthorized().json(ErrorResponse {
                            status: "error".to_string(),
                            error: "Wrong password".to_string(),
                        }),
                    }
                }
                Ok(None) => HttpResponse::Unauthorized().json(ErrorResponse {
                    status: "error".to_string(),
                    error: "User not found".to_string(),
                }),
                Err(e) => HttpResponse::Unauthorized().json(ErrorResponse {
                    status: "error".to_string(),
                    error: e.to_string(),
                }),
            }
        }
        "refresh_token" => {
            todo!("refresh token lol - imagine")
        }
        _ => HttpResponse::Unauthorized().json(ErrorResponse {
            status: "error".to_string(),
            error: "unknown_identifier".to_string(),
        }),
    }
}

#[get("/user/@me")]
async fn me(AuthData(user): AuthData) -> actix_web::Result<impl actix_web::Responder> {
    Ok(HttpResponse::Ok().json(user))
}

pub fn configure_na(cfg: &mut web::ServiceConfig) {
    cfg.service(register).service(login);
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(me);
}
