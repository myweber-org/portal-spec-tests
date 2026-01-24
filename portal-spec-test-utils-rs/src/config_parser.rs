use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_https: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "".to_string(),
            database: "app_db".to_string(),
            pool_size: 10,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            address: "0.0.0.0".to_string(),
            port: 8080,
            enable_https: false,
            cert_path: None,
            key_path: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            log_level: "info".to_string(),
            cache_ttl: 300,
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileNotFound(String),
    ParseError(String),
    ValidationError(String),
    IoError(std::io::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(ConfigError::FileNotFound(
                path_ref.to_string_lossy().to_string()
            ));
        }

        let content = fs::read_to_string(path_ref)?;
        
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::ValidationError(
                "Database host cannot be empty".to_string()
            ));
        }

        if self.database.port == 0 {
            return Err(ConfigError::ValidationError(
                "Database port cannot be zero".to_string()
            ));
        }

        if self.database.pool_size == 0 {
            return Err(ConfigError::ValidationError(
                "Database pool size cannot be zero".to_string()
            ));
        }

        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port cannot be zero".to_string()
            ));
        }

        if self.server.enable_https {
            if self.server.cert_path.is_none() || self.server.key_path.is_none() {
                return Err(ConfigError::ValidationError(
                    "HTTPS requires both certificate and key paths".to_string()
                ));
            }
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}", self.log_level)
            ));
        }

        Ok(())
    }

    pub fn merge_defaults(&mut self) {
        let default = AppConfig::default();
        
        if self.database.host.is_empty() {
            self.database.host = default.database.host;
        }
        
        if self.database.port == 0 {
            self.database.port = default.database.port;
        }
        
        if self.database.username.is_empty() {
            self.database.username = default.database.username;
        }
        
        if self.database.database.is_empty() {
            self.database.database = default.database.database;
        }
        
        if self.database.pool_size == 0 {
            self.database.pool_size = default.database.pool_size;
        }
        
        if self.server.address.is_empty() {
            self.server.address = default.server.address;
        }
        
        if self.server.port == 0 {
            self.server.port = default.server.port;
        }
        
        if self.log_level.is_empty() {
            self.log_level = default.log_level;
        }
        
        if self.cache_ttl == 0 {
            self.cache_ttl = default.cache_ttl;
        }
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let toml_content = self.to_toml()?;
        fs::write(path, toml_content)?;
        Ok(())
    }
}

pub fn create_default_config<P: AsRef<Path>>(path: P) -> Result<(), ConfigError> {
    let config = AppConfig::default();
    config.save_to_file(path)
}