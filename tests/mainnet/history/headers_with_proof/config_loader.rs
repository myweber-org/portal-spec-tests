use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    pub settings: HashMap<String, String>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                settings.insert(key, processed_value);
            }
        }

        Ok(Config { settings })
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
        self.settings.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_loading() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/mydb").unwrap();
        writeln!(file, "API_KEY=${SECRET_KEY}").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        env::set_var("SECRET_KEY", "abc123");

        let config = Config::load(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/mydb");
        assert_eq!(config.get("API_KEY").unwrap(), "abc123");
        assert_eq!(config.get("TIMEOUT").unwrap(), "30");
        assert!(config.get("NONEXISTENT").is_none());
    }
}use std::collections::HashMap;
use std::env;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        for (key, value) in env::vars() {
            if key.starts_with("APP_") {
                values.insert(key.to_lowercase(), value);
            }
        }
        
        Config { values }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        let formatted_key = format!("app_{}", key.to_lowercase());
        self.values.get(&formatted_key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|| default.to_string())
    }
}

pub fn load_config() -> Config {
    Config::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_loading() {
        env::set_var("APP_DATABASE_URL", "postgres://localhost:5432");
        env::set_var("APP_API_KEY", "secret123");
        env::set_var("OTHER_VAR", "should_be_ignored");
        
        let config = Config::new();
        
        assert_eq!(config.get("database_url"), Some(&"postgres://localhost:5432".to_string()));
        assert_eq!(config.get("api_key"), Some(&"secret123".to_string()));
        assert_eq!(config.get("other_var"), None);
    }
    
    #[test]
    fn test_get_or_default() {
        let config = Config::new();
        
        assert_eq!(config.get_or_default("missing_key", "default_value"), "default_value");
    }
}use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_ssl: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub environment: String,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;
        
        config.apply_environment_overrides()?;
        config.validate()?;
        
        Ok(config)
    }
    
    fn apply_environment_overrides(&mut self) -> Result<(), ConfigError> {
        if let Ok(env) = env::var("APP_ENVIRONMENT") {
            self.environment = env;
        }
        
        if let Ok(host) = env::var("DATABASE_HOST") {
            self.database.host = host;
        }
        
        if let Ok(port_str) = env::var("DATABASE_PORT") {
            self.database.port = port_str.parse()
                .map_err(|_| ConfigError::InvalidValue("DATABASE_PORT".to_string(), port_str))?;
        }
        
        if let Ok(log_level) = env::var("LOG_LEVEL") {
            self.logging.level = log_level;
        }
        
        Ok(())
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::MissingField("database.host".to_string()));
        }
        
        if self.database.port == 0 {
            return Err(ConfigError::InvalidValue("database.port".to_string(), "0".to_string()));
        }
        
        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue("server.port".to_string(), "0".to_string()));
        }
        
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.as_str()) {
            return Err(ConfigError::InvalidValue(
                "logging.level".to_string(), 
                self.logging.level.clone()
            ));
        }
        
        Ok(())
    }
    
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
    
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.database.username,
            self.database.password,
            self.database.host,
            self.database.port,
            self.database.database
        )
    }
}

pub fn load_default_config() -> Result<AppConfig, ConfigError> {
    let default_paths = vec![
        "config.toml",
        "config/local.toml",
        "/etc/app/config.toml",
    ];
    
    for path in default_paths {
        if let Ok(config) = AppConfig::from_file(path) {
            return Ok(config);
        }
    }
    
    Err(ConfigError::MissingField("No configuration file found".to_string()))
}