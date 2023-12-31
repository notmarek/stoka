use std::fmt::Display;

use actix_web::FromRequest;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::ResponseError;
use futures::future::{ready, Ready};
use log::error;
use serde::Serialize;
use thiserror::Error;
pub mod api;
pub mod auth;
pub mod config;

#[derive(Debug, Serialize)]
pub struct Response<T: Serialize> {
    status: String,
    data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    status: String,
    error: String,
}
impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        serde_json::to_string(self).unwrap().fmt(f)
    }
}
#[derive(Clone, Debug)]
pub struct AuthData(pub entity::user::Model);

impl FromRequest for AuthData {
    type Error = Unauthorized;

    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _payload: &mut actix_http::Payload) -> Self::Future {
        ready(req.extensions().get().map(Self::clone).ok_or(Unauthorized))
    }
}

#[derive(Error, Debug)]
#[error("unauthorized")]
pub struct Unauthorized;

impl ResponseError for Unauthorized {
    fn status_code(&self) -> actix_http::StatusCode {
        actix_http::StatusCode::UNAUTHORIZED
    }
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(actix_web::http::header::ContentType::json())
            .body(format!(r#"{{ "status": "error", "error": "{self}" }}"#))
    }
}
