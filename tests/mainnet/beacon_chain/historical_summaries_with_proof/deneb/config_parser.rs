use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/mydb".to_string(),
                max_connections: 10,
                min_connections: 2,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("app.log".to_string()),
                enable_console: true,
            },
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_str = fs::read_to_string(path)?;
    let config: AppConfig = toml::from_str(&config_str)?;
    validate_config(&config)?;
    Ok(config)
}

pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<(), Box<dyn std::error::Error>> {
    let config_str = toml::to_string_pretty(config)?;
    fs::write(path, config_str)?;
    Ok(())
}

fn validate_config(config: &AppConfig) -> Result<(), String> {
    if config.server.port == 0 {
        return Err("Server port cannot be 0".to_string());
    }
    
    if config.database.max_connections < config.database.min_connections {
        return Err("Max connections must be greater than or equal to min connections".to_string());
    }
    
    let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
    if !valid_log_levels.contains(&config.logging.level.as_str()) {
        return Err(format!("Invalid log level: {}", config.logging.level));
    }
    
    Ok(())
}

pub fn create_default_config<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let default_config = AppConfig::default();
    save_config(&default_config, path)
}