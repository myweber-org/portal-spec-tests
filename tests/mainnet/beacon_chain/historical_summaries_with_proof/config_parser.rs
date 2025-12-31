use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub settings: HashMap<String, String>,
    pub defaults: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config {
            settings: HashMap::new(),
            defaults: HashMap::from([
                ("timeout".to_string(), "30".to_string()),
                ("retries".to_string(), "3".to_string()),
                ("log_level".to_string(), "info".to_string()),
            ]),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.parse_content(&content)
    }

    fn parse_content(&mut self, content: &str) -> Result<(), String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid line format: {}", line));
            }

            let key = parts[0].trim().to_string();
            let value = parts[1].trim().to_string();

            if value.is_empty() {
                return Err(format!("Empty value for key: {}", key));
            }

            self.settings.insert(key, value);
        }
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key).or_else(|| self.defaults.get(key))
    }

    pub fn get_with_fallback(&self, key: &str, fallback: &str) -> String {
        self.get(key).map(|s| s.as_str()).unwrap_or(fallback).to_string()
    }

    pub fn validate_required(&self, required_keys: &[&str]) -> Result<(), Vec<String>> {
        let mut missing = Vec::new();
        for key in required_keys {
            if !self.settings.contains_key(*key) && !self.defaults.contains_key(*key) {
                missing.push(key.to_string());
            }
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    pub fn merge(&mut self, other: Config) {
        for (key, value) in other.settings {
            self.settings.insert(key, value);
        }
        for (key, value) in other.defaults {
            self.defaults.entry(key).or_insert(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut config = Config::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "host=localhost").unwrap();
        writeln!(temp_file, "port=8080").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();

        assert!(config.load_from_file(temp_file.path()).is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"30".to_string()));
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let required = vec!["timeout", "nonexistent"];
        let result = config.validate_required(&required);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["nonexistent"]);
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub max_files: usize,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
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
                connect_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("/var/log/app.log".to_string()),
                max_files: 5,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .map_err(|_| ConfigError::FileNotFound(path.to_string()))?;
        
        let mut config: AppConfig = serde_json::from_str(&content)?;
        config.validate()?;
        
        Ok(config)
    }
    
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = AppConfig::default();
        
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port_str) = std::env::var("SERVER_PORT") {
            config.server.port = port_str.parse()
                .map_err(|e| ConfigError::InvalidConfig(format!("Invalid port: {}", e)))?;
        }
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::InvalidConfig("Port cannot be 0".to_string()));
        }
        
        if self.database.max_connections < self.database.min_connections {
            return Err(ConfigError::InvalidConfig(
                "max_connections must be >= min_connections".to_string()
            ));
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidConfig(
                format!("Invalid log level: {}", self.logging.level)
            ));
        }
        
        Ok(())
    }
    
    pub fn to_env_vars(&self) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        
        vars.insert("SERVER_HOST".to_string(), self.server.host.clone());
        vars.insert("SERVER_PORT".to_string(), self.server.port.to_string());
        vars.insert("DATABASE_URL".to_string(), self.database.url.clone());
        
        vars
    }
}

pub fn load_config(config_path: Option<&str>) -> Result<AppConfig, ConfigError> {
    match config_path {
        Some(path) => AppConfig::from_file(path),
        None => AppConfig::from_env(),
    }
}