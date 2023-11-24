use actix_cors::Cors;
use actix_web::{http::header, middleware, web::Data, App, HttpServer};
use ksync::{api, config::Config};
use log::{debug, info};
use migration::MigratorTrait;
use sea_orm::{Database, DatabaseConnection};
use std::{env, str::FromStr};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(
            log::LevelFilter::from_str(
                &env::var("RUST_LOG").unwrap_or_else(|_| String::from("info")),
            )
            .unwrap_or(log::LevelFilter::Trace),
        )
        .filter_module("sqlx::query", log::LevelFilter::Warn)
        .init();
    debug!("Initalized logger!");
    let conf_path = "config.json";
    info!("Looking for config.json in current directory.");
    let config: Config = {
        let conf = std::fs::read_to_string(conf_path)?;
        serde_json::from_str(&conf)?
    };
    let db_string = config.db.connection_string.clone();
    let cors = config.cors.clone();
    let port = config.port;
    let address = config.address.clone();
    let db: DatabaseConnection = Database::connect(db_string)
        .await
        .expect("Failed to create a database connection.");
    migration::Migrator::up(&db, None).await.unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(config.clone()))
            .app_data(Data::new(db.clone()))
            .wrap({
                if let Some(cors_conf) = &cors {
                    let cors = Cors::default()
                        .supports_credentials()
                        .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "PUT"])
                        .allowed_headers(vec![
                            header::ACCEPT,
                            header::AUTHORIZATION,
                            header::CONTENT_TYPE,
                        ])
                        .max_age(3600);
                    let cors = cors_conf
                        .origins
                        .iter()
                        .fold(cors, |cors, origin| cors.allowed_origin(origin));
                    cors
                } else {
                    Cors::permissive()
                }
            })
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .configure(api::configure)
            .configure(api::configure_no_auth)
    })
    .bind((address, port))?
    .run()
    .await
}
