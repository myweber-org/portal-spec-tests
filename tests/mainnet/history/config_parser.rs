
use std::collections::HashMap;
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
        if value.starts_with('$') {
            let var_name = &value[1..];
            env::var(var_name).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
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
        writeln!(file, "HOST=localhost").unwrap();
        writeln!(file, "PORT=8080").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "TIMEOUT=30").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"8080".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("MISSING"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_PASSWORD", "secret123");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PASSWORD=$DB_PASSWORD").unwrap();
        writeln!(file, "OTHER=plain_value").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PASSWORD"), Some(&"secret123".to_string()));
        assert_eq!(config.get("OTHER"), Some(&"plain_value".to_string()));
    }

    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING=value1").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING", "default"), "value1");
        assert_eq!(config.get_or_default("MISSING", "default"), "default");
    }
}use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse config: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for field {0}: {1}")]
    InvalidValue(String, String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default = "default_database_name")]
    pub database: String,
}

fn default_database_name() -> String {
    "app_db".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default)]
    pub enable_compression: bool,
}

fn default_port() -> u16 {
    8080
}

fn default_timeout() -> u64 {
    30
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

        if self.server.port == 0 {
            return Err(ConfigError::InvalidValue(
                "server.port".to_string(),
                "Port cannot be zero".to_string(),
            ));
        }

        if self.server.timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue(
                "server.timeout_seconds".to_string(),
                "Timeout must be greater than zero".to_string(),
            ));
        }

        Ok(())
    }

    pub fn get_feature_flag(&self, feature: &str) -> bool {
        self.features.get(feature).copied().unwrap_or(false)
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: None,
                database: default_database_name(),
            },
            server: ServerConfig {
                address: "0.0.0.0".to_string(),
                port: default_port(),
                timeout_seconds: default_timeout(),
                enable_compression: false,
            },
            features: HashMap::new(),
            metadata: HashMap::new(),
        }
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
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.timeout_seconds, 30);
    }

    #[test]
    fn test_from_valid_file() {
        let toml_content = r#"
            [database]
            host = "db.example.com"
            port = 5432
            username = "admin"
            password = "secret"
            database = "production"

            [server]
            address = "127.0.0.1"
            port = 3000
            timeout_seconds = 60
            enable_compression = true

            [features]
            caching = true
            logging = false

            [metadata]
            environment = "production"
            version = "1.0.0"
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database.host, "db.example.com");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.server.enable_compression, true);
        assert_eq!(config.get_feature_flag("caching"), true);
        assert_eq!(config.get_feature_flag("logging"), false);
        assert_eq!(config.get_metadata("environment"), Some(&"production".to_string()));
    }

    #[test]
    fn test_validation() {
        let invalid_toml = r#"
            [database]
            host = ""
            port = 5432
            username = "admin"

            [server]
            address = "127.0.0.1"
            port = 3000
            timeout_seconds = 60
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), invalid_toml).unwrap();
        
        let result = AppConfig::from_file(file.path().to_str().unwrap());
        assert!(result.is_err());
        
        if let Err(ConfigError::MissingField(field)) = result {
            assert_eq!(field, "database.host");
        } else {
            panic!("Expected MissingField error");
        }
    }

    #[test]
    fn test_partial_config() {
        let partial_toml = r#"
            [database]
            host = "localhost"
            port = 5432
            username = "user"

            [server]
            address = "0.0.0.0"
        "#;

        let mut file = NamedTempFile::new().unwrap();
        std::fs::write(file.path(), partial_toml).unwrap();
        
        let config = AppConfig::from_file(file.path().to_str().unwrap()).unwrap();
        
        assert_eq!(config.database.database, "app_db");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.timeout_seconds, 30);
        assert_eq!(config.server.enable_compression, false);
    }
}