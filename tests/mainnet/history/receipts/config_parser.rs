use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub debug_mode: bool,
    pub api_keys: HashMap<String, String>,
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
                config.parse_key_value(key, value);
            }
        }
        
        Ok(config)
    }
    
    fn parse_key_value(&mut self, key: &str, value: &str) {
        match key {
            "DATABASE_URL" => self.database_url = Self::resolve_env_var(value).to_string(),
            "PORT" => {
                if let Ok(port) = value.parse() {
                    self.port = port;
                }
            }
            "DEBUG_MODE" => self.debug_mode = value.parse().unwrap_or(false),
            key if key.starts_with("API_KEY_") => {
                let service = key.trim_start_matches("API_KEY_").to_lowercase();
                self.api_keys.insert(service, Self::resolve_env_var(value).to_string());
            }
            _ => {}
        }
    }
    
    fn resolve_env_var(value: &str) -> String {
        if value.starts_with("${") && value.ends_with('}') {
            let var_name = &value[2..value.len()-1];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn default() -> Self {
        Self {
            database_url: "postgres://localhost:5432/mydb".to_string(),
            port: 8080,
            debug_mode: false,
            api_keys: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_parsing() {
        let config_content = r#"
            DATABASE_URL=postgres://user:pass@localhost/db
            PORT=3000
            DEBUG_MODE=true
            API_KEY_WEATHER=${WEATHER_API_KEY}
        "#;
        
        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), config_content).unwrap();
        
        env::set_var("WEATHER_API_KEY", "test_key_123");
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.port, 3000);
        assert_eq!(config.debug_mode, true);
        assert_eq!(config.api_keys.get("weather"), Some(&"test_key_123".to_string()));
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

#[derive(Debug, PartialEq)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub debug_mode: bool,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let mut variables = HashMap::new();
        for (key, value) in env::vars() {
            variables.insert(key, value);
        }

        let mut config_map = HashMap::new();
        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid config line: {}", line));
            }

            let key = parts[0].trim().to_string();
            let mut value = parts[1].trim().to_string();

            for (var_name, var_value) in &variables {
                let placeholder = format!("${{{}}}", var_name);
                value = value.replace(&placeholder, var_value);
            }

            config_map.insert(key, value);
        }

        let database_url = config_map
            .get("DATABASE_URL")
            .ok_or("Missing DATABASE_URL")?
            .clone();

        let max_connections = config_map
            .get("MAX_CONNECTIONS")
            .ok_or("Missing MAX_CONNECTIONS")?
            .parse::<u32>()
            .map_err(|e| format!("Invalid MAX_CONNECTIONS: {}", e))?;

        let debug_mode = config_map
            .get("DEBUG_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Config {
            database_url,
            max_connections,
            debug_mode,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/test").unwrap();
        writeln!(file, "MAX_CONNECTIONS=10").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "DEBUG_MODE=true").unwrap();

        env::set_var("CUSTOM_VAR", "replaced_value");

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database_url, "postgres://localhost/test");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.debug_mode, true);
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for field {0}: {1}")]
    InvalidValue(String, String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_database_name")]
    pub database: String,
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
}

fn default_database_name() -> String {
    "app_db".to_string()
}

fn default_pool_size() -> u32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default)]
    pub cors_allowed_origins: Vec<String>,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    #[serde(default)]
    pub features: HashMap<String, bool>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database.host.is_empty() {
            return Err(ConfigError::MissingField("database.host".to_string()));
        }

        if self.database.port == 0 {
            return Err(ConfigError::InvalidValue(
                "database.port".to_string(),
                "Port cannot be zero".to_string(),
            ));
        }

        if self.database.username.is_empty() {
            return Err(ConfigError::MissingField("database.username".to_string()));
        }

        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue(
                "server.port".to_string(),
                "Port cannot be zero".to_string(),
            ));
        }

        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.server.log_level.as_str()) {
            return Err(ConfigError::InvalidValue(
                "server.log_level".to_string(),
                format!(
                    "Must be one of: {}",
                    valid_log_levels.join(", ")
                ),
            ));
        }

        Ok(())
    }

    pub fn default() -> Self {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: None,
                database: default_database_name(),
                max_connections: default_pool_size(),
            },
            server: ServerConfig {
                host: default_host(),
                port: default_port(),
                tls_enabled: false,
                cors_allowed_origins: Vec::new(),
                log_level: default_log_level(),
            },
            features: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.host, "localhost");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.log_level, "info");
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.database.host = String::new();
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.database.port = 0;
        assert!(config.validate().is_err());

        let mut config = AppConfig::default();
        config.server.log_level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_from_file() {
        let toml_content = r#"
            [database]
            host = "db.example.com"
            port = 5432
            username = "app_user"
            password = "secret"
            database = "production_db"
            max_connections = 20

            [server]
            host = "0.0.0.0"
            port = 8443
            tls_enabled = true
            log_level = "debug"
            cors_allowed_origins = ["https://example.com"]

            [features]
            experimental = true
            maintenance = false

            [metadata]
            environment = "production"
            version = "1.0.0"
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.database.host, "db.example.com");
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.database.username, "app_user");
        assert_eq!(config.database.password, Some("secret".to_string()));
        assert_eq!(config.database.database, "production_db");
        assert_eq!(config.database.max_connections, 20);
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8443);
        assert!(config.server.tls_enabled);
        assert_eq!(config.server.log_level, "debug");
        assert_eq!(config.features.get("experimental"), Some(&true));
        assert_eq!(config.metadata.get("environment"), Some(&"production".to_string()));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let config = AppConfig::default();
        let toml_str = config.to_toml().unwrap();
        let parsed_config: AppConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.database.host, parsed_config.database.host);
        assert_eq!(config.server.port, parsed_config.server.port);
    }
}