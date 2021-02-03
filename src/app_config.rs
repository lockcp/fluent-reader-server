use config::ConfigError;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: i32,
    pub secret: String,
    pub token_time: i64,
    pub salt: String,
}

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub pg: deadpool_postgres::Config,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into()
    }
}
