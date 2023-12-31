use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub address: String,
    pub filepath: String,
    pub port: u16,
    pub jwt: JWTConfig,
    #[serde(default)]
    pub cors: Option<CORSConfig>,
    pub db: DBConfig,
}

#[derive(Deserialize, Clone)]
pub struct JWTConfig {
    pub valid_for: i64,
    pub private_key: PathBuf,
    pub public_key: PathBuf,
}

#[derive(Deserialize, Clone)]
pub struct CORSConfig {
    #[serde(rename = "allowed_origins")]
    pub origins: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct DBConfig {
    pub connection_string: String,
    pub connections: u32,
}
