use actix_web::{middleware::Compat, web};
use actix_web_httpauth::middleware::HttpAuthentication;

pub mod user;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let auth = HttpAuthentication::bearer(crate::auth::validator);

    cfg.service(
        web::scope("/api")
            .wrap(Compat::new(auth))
            .configure(user::configure),
    );
}

pub fn configure_no_auth(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/na")
            .configure(user::configure_na)
            .service(health),
    );
}

#[actix_web::get("/health")]
async fn health() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body("OK")
}
