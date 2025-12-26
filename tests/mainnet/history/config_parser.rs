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
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub pool_timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub rotation: String,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    pub fn to_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.server.port == 0 {
            errors.push("Server port cannot be 0".to_string());
        }

        if self.server.timeout_seconds > 3600 {
            errors.push("Server timeout cannot exceed 1 hour".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Database max connections cannot be 0".to_string());
        }

        if !["trace", "debug", "info", "warn", "error"].contains(&self.logging.level.as_str()) {
            errors.push(format!("Invalid log level: {}", self.logging.level));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn get_env_overrides(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();
        
        env_vars.insert("APP_HOST".to_string(), self.server.host.clone());
        env_vars.insert("APP_PORT".to_string(), self.server.port.to_string());
        env_vars.insert("DB_URL".to_string(), self.database.url.clone());
        env_vars.insert("LOG_LEVEL".to_string(), self.logging.level.clone());
        
        env_vars
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(Vec<String>),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::ValidationError(errors) => {
                write!(f, "Validation errors: {}", errors.join(", "))
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_config_with_fallback(paths: &[&str]) -> Result<AppConfig, ConfigError> {
    for path in paths {
        match AppConfig::from_file(path) {
            Ok(config) => {
                if let Err(validation_errors) = config.validate() {
                    return Err(ConfigError::ValidationError(validation_errors));
                }
                return Ok(config);
            }
            Err(e) if path == paths.last().unwrap() => return Err(e),
            _ => continue,
        }
    }
    
    Err(ConfigError::IoError("No configuration files found".to_string()))
}