use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
    pub features: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut config_map = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                config_map.insert(key, value);
            }
        }

        Self::from_map(&config_map)
    }

    pub fn from_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let database_url = Self::get_value(map, "DATABASE_URL")
            .or_else(|| env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "postgres://localhost:5432/app".to_string());

        let port = Self::get_value(map, "PORT")
            .or_else(|| env::var("PORT").ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(8080);

        let log_level = Self::get_value(map, "LOG_LEVEL")
            .or_else(|| env::var("LOG_LEVEL").ok())
            .unwrap_or_else(|| "info".to_string());

        let features = Self::get_value(map, "FEATURES")
            .map(|s| s.split(',').map(|f| f.trim().to_string()).collect())
            .unwrap_or_default();

        Ok(Config {
            database_url,
            port,
            log_level,
            features,
        })
    }

    fn get_value(map: &HashMap<String, String>, key: &str) -> Option<String> {
        map.get(key).cloned()
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.database_url.is_empty() {
            errors.push("DATABASE_URL cannot be empty".to_string());
        }

        if self.port == 0 {
            errors.push("PORT must be greater than 0".to_string());
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            errors.push(format!(
                "LOG_LEVEL must be one of: {}",
                valid_log_levels.join(", ")
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_map() {
        let mut map = HashMap::new();
        map.insert("DATABASE_URL".to_string(), "postgres://test".to_string());
        map.insert("PORT".to_string(), "3000".to_string());
        map.insert("LOG_LEVEL".to_string(), "debug".to_string());

        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://test");
        assert_eq!(config.port, 3000);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_defaults() {
        let map = HashMap::new();
        let config = Config::from_map(&map).unwrap();
        assert_eq!(config.database_url, "postgres://localhost:5432/app");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.features.is_empty());
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            database_url: "".to_string(),
            port: 0,
            log_level: "invalid".to_string(),
            features: vec![],
        };

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut values = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(value: &str) -> String {
        let mut result = String::new();
        let mut chars = value.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Skip '{'
                let mut var_name = String::new();
                
                while let Some(ch) = chars.next() {
                    if ch == '}' {
                        break;
                    }
                    var_name.push(ch);
                }
                
                if let Ok(env_value) = env::var(&var_name) {
                    result.push_str(&env_value);
                }
            } else {
                result.push(ch);
            }
        }
        
        result
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key).map(|s| s.as_str()).unwrap_or(default).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "mysecret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=${APP_SECRET}").unwrap();
        writeln!(file, "HOST=localhost:${PORT}").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY").unwrap(), "mysecret123");
        assert_eq!(config.get("HOST").unwrap(), "localhost:");
    }
}use serde::{Deserialize, Serialize};
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
    pub connect_timeout: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: String,
    pub max_files: usize,
    pub max_file_size_mb: u64,
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
                max_connections: 20,
                min_connections: 5,
                connect_timeout: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_files: 10,
                max_file_size_mb: 100,
            },
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
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path_ref = path.as_ref();
        
        if !path_ref.exists() {
            return Err(ConfigError::FileNotFound(
                path_ref.to_string_lossy().to_string()
            ));
        }

        let content = fs::read_to_string(path_ref)?;
        
        let mut config: AppConfig = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        config.validate()?;
        Ok(config)
    }

    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to load config: {:?}, using defaults", e);
                Self::default()
            }
        }
    }

    pub fn validate(&mut self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError(
                "Server port cannot be 0".to_string()
            ));
        }

        if self.database.max_connections < self.database.min_connections {
            return Err(ConfigError::ValidationError(
                "Max connections must be greater than or equal to min connections".to_string()
            ));
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::ValidationError(
                format!("Invalid log level: {}", self.logging.level)
            ));
        }

        if self.logging.max_file_size_mb == 0 {
            return Err(ConfigError::ValidationError(
                "Max file size must be greater than 0".to_string()
            ));
        }

        Ok(())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let yaml = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        fs::write(path, yaml)?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.logging.level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server.port = 0;
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_save_and_load() {
        let config = AppConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        config.save_to_file(temp_file.path()).unwrap();
        let loaded_config = AppConfig::load_from_file(temp_file.path()).unwrap();
        
        assert_eq!(config.server.port, loaded_config.server.port);
        assert_eq!(config.database.url, loaded_config.database.url);
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = AppConfig::load_from_file("/nonexistent/path/config.yaml");
        assert!(matches!(result, Err(ConfigError::FileNotFound(_))));
    }
}