use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub enable_cache: bool,
}

impl AppConfig {
    pub fn from_file(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(config_path)?;
        let mut config: AppConfig = toml::from_str(&config_content)?;

        config.apply_environment_overrides();
        config.validate()?;

        Ok(config)
    }

    fn apply_environment_overrides(&mut self) {
        if let Ok(db_host) = env::var("DB_HOST") {
            self.database.host = db_host;
        }
        if let Ok(db_port) = env::var("DB_PORT") {
            if let Ok(port) = db_port.parse() {
                self.database.port = port;
            }
        }
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.log_level = log_level;
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }
        if self.server.max_connections == 0 {
            return Err("Max connections must be greater than zero".to_string());
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }

        Ok(())
    }
}

pub fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = env::var("CONFIG_PATH")
        .unwrap_or_else(|_| "config.toml".to_string());

    if !Path::new(&config_path).exists() {
        return Err(format!("Configuration file not found: {}", config_path).into());
    }

    AppConfig::from_file(&config_path)
}