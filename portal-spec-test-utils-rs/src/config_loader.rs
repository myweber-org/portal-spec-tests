use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub log_level: String,
    pub features: HashMap<String, bool>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                config.apply_setting(key, value);
            }
        }
        
        Ok(config)
    }
    
    fn apply_setting(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = Self::resolve_env_var(value),
            "SERVER_PORT" => {
                if let Ok(port) = value.parse() {
                    self.server_port = port;
                }
            }
            "LOG_LEVEL" => self.log_level = value.to_string(),
            _ if key.starts_with("FEATURE_") => {
                let feature_name = key.trim_start_matches("FEATURE_").to_lowercase();
                let enabled = value.eq_ignore_ascii_case("true");
                self.features.insert(feature_name, enabled);
            }
            _ => {}
        }
    }
    
    fn resolve_env_var(value: &str) -> String {
        if let Some(var_name) = value.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: String::from("postgres://localhost:5432/db"),
            server_port: 8080,
            log_level: String::from("info"),
            features: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_loading() {
        let config_content = r#"
            DATABASE_URL=postgres://user:pass@localhost:5432/app
            SERVER_PORT=3000
            LOG_LEVEL=debug
            FEATURE_API_V2=true
            FEATURE_CACHE=false
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::io::write(&mut file, config_content).unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database_url, "postgres://user:pass@localhost:5432/app");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert!(config.is_feature_enabled("api_v2"));
        assert!(!config.is_feature_enabled("cache"));
    }
    
    #[test]
    fn test_env_var_resolution() {
        env::set_var("DB_HOST", "database.server.com");
        
        let config_content = "DATABASE_URL=postgres://${DB_HOST}:5432/db";
        let mut file = NamedTempFile::new().unwrap();
        std::io::write(&mut file, config_content).unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://database.server.com:5432/db");
        
        env::remove_var("DB_HOST");
    }
}
use serde::Deserialize;
use std::env;

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
    pub host: String,
    pub port: u16,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub debug_mode: bool,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_config = DatabaseConfig {
            host: env::var("DB_HOST")
                .map_err(|_| ConfigError::MissingVariable("DB_HOST".to_string()))?,
            port: env::var("DB_PORT")
                .map_err(|_| ConfigError::MissingVariable("DB_PORT".to_string()))?
                .parse()
                .map_err(|_| ConfigError::InvalidValue("DB_PORT".to_string()))?,
            username: env::var("DB_USER")
                .map_err(|_| ConfigError::MissingVariable("DB_USER".to_string()))?,
            password: env::var("DB_PASSWORD")
                .map_err(|_| ConfigError::MissingVariable("DB_PASSWORD".to_string()))?,
            database_name: env::var("DB_NAME")
                .map_err(|_| ConfigError::MissingVariable("DB_NAME".to_string()))?,
        };

        let server_config = ServerConfig {
            host: env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("SERVER_PORT".to_string()))?,
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("MAX_CONNECTIONS".to_string()))?,
            timeout_seconds: env::var("TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("TIMEOUT_SECONDS".to_string()))?,
        };

        let debug_mode = env::var("DEBUG_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("DEBUG_MODE".to_string()))?;

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        Ok(AppConfig {
            database: database_config,
            server: server_config,
            debug_mode,
            log_level,
        })
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::InvalidValue("DB_HOST".to_string()));
        }
        if self.database.port == 0 {
            return Err(ConfigError::InvalidValue("DB_PORT".to_string()));
        }
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("SERVER_PORT".to_string()));
        }
        if self.server.max_connections == 0 {
            return Err(ConfigError::InvalidValue("MAX_CONNECTIONS".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingVariable(String),
    InvalidValue(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingVariable(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidValue(var) => write!(f, "Invalid value for variable: {}", var),
            ConfigError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}