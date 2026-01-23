use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("app.log".to_string()),
                enable_console: true,
            },
        }
    }

    pub fn to_env_map(&self) -> HashMap<String, String> {
        let mut env_map = HashMap::new();
        env_map.insert("SERVER_HOST".to_string(), self.server.host.clone());
        env_map.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        env_map.insert("DB_URL".to_string(), self.database.url.clone());
        env_map.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        env_map
    }
}

pub fn validate_config(config: &AppConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if config.server.port == 0 {
        errors.push("Server port cannot be zero".to_string());
    }

    if config.database.max_connections == 0 {
        errors.push("Database max connections cannot be zero".to_string());
    }

    let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        errors.push(format!("Invalid log level: {}", config.logging.level));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}