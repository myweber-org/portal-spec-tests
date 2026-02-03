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
    pub timeout_seconds: u32,
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
    pub file_path: String,
    pub max_file_size_mb: u32,
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
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: "./logs/app.log".to_string(),
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database.max_connections < self.database.min_connections {
            return Err("Max connections must be greater than or equal to min connections".to_string());
        }
        if self.logging.max_file_size_mb == 0 {
            return Err("Log file size must be greater than zero".to_string());
        }
        Ok(())
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}use std::collections::HashMap;
use std::env;
use std::fs;

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
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
        if value.starts_with("${") && value.ends_with('}') {
            let env_var = &value[2..value.len() - 1];
            env::var(env_var).unwrap_or_else(|_| value.to_string())
        } else {
            value.to_string()
        }
    }
    
    pub fn get(&self, key: &str) -> Option<&String> {
        self.values.get(key)
    }
    
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values.get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
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
        writeln!(file, "DATABASE_HOST=localhost").unwrap();
        writeln!(file, "DATABASE_PORT=5432").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "  TIMEOUT = 30  ").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("DATABASE_PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("TIMEOUT"), Some(&"30".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }
    
    #[test]
    fn test_env_substitution() {
        env::set_var("APP_SECRET", "my_secret_key");
        
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "SECRET_KEY=${APP_SECRET}").unwrap();
        writeln!(file, "OTHER_KEY=static_value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("SECRET_KEY"), Some(&"my_secret_key".to_string()));
        assert_eq!(config.get("OTHER_KEY"), Some(&"static_value".to_string()));
    }
    
    #[test]
    fn test_get_or_default() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "EXISTING_KEY=actual_value").unwrap();
        
        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING_KEY", "default"), "actual_value");
        assert_eq!(config.get_or_default("MISSING_KEY", "default_value"), "default_value");
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
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let processed_value = Self::process_value(value.trim());
                values.insert(key, processed_value);
            }
        }

        Ok(Config { values })
    }

    fn process_value(raw: &str) -> String {
        let mut result = String::new();
        let mut chars = raw.chars().peekable();

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
                let env_value = env::var(&var_name).unwrap_or_else(|_| String::new());
                result.push_str(&env_value);
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
        writeln!(file, "APP_NAME=MyApp").unwrap();
        writeln!(file, "VERSION=1.0.0").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "EMPTY=").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("APP_NAME"), Some(&"MyApp".to_string()));
        assert_eq!(config.get("VERSION"), Some(&"1.0.0".to_string()));
        assert_eq!(config.get("EMPTY"), Some(&"".to_string()));
        assert_eq!(config.get("NONEXISTENT"), None);
    }

    #[test]
    fn test_env_substitution() {
        env::set_var("DB_HOST", "localhost");
        env::set_var("DB_PORT", "5432");

        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "HOST=${DB_HOST}").unwrap();
        writeln!(file, "PORT=${DB_PORT}").unwrap();
        writeln!(file, "URL=${DB_HOST}:${DB_PORT}").unwrap();
        writeln!(file, "MISSING=${NONEXISTENT_VAR}").unwrap();

        let config = Config::from_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("HOST"), Some(&"localhost".to_string()));
        assert_eq!(config.get("PORT"), Some(&"5432".to_string()));
        assert_eq!(config.get("URL"), Some(&"localhost:5432".to_string()));
        assert_eq!(config.get("MISSING"), Some(&"".to_string()));
    }
}use std::env;
use std::fs;
use std::collections::HashMap;

pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let mut settings = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let mut value = value.trim().to_string();

                if value.starts_with("${") && value.ends_with('}') {
                    let env_var = &value[2..value.len() - 1];
                    value = env::var(env_var).unwrap_or_else(|_| value.clone());
                }

                settings.insert(key, value);
            }
        }

        Ok(Config { settings })
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.settings.get(key)
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.settings.get(key).cloned().unwrap_or(default.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_parsing() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(temp_file, "# This is a comment").unwrap();
        writeln!(temp_file, "MAX_CONNECTIONS=100").unwrap();
        writeln!(temp_file, "").unwrap();

        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("DATABASE_URL").unwrap(), "postgres://localhost/db");
        assert_eq!(config.get("MAX_CONNECTIONS").unwrap(), "100");
        assert!(config.get("NONEXISTENT").is_none());
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("APP_PORT", "8080");
        
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "PORT=${APP_PORT}").unwrap();
        writeln!(temp_file, "HOST=localhost").unwrap();

        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get("PORT").unwrap(), "8080");
        assert_eq!(config.get("HOST").unwrap(), "localhost");
    }

    #[test]
    fn test_get_or_default() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "EXISTING_KEY=value").unwrap();

        let config = Config::new(temp_file.path().to_str().unwrap()).unwrap();
        assert_eq!(config.get_or_default("EXISTING_KEY", "default"), "value");
        assert_eq!(config.get_or_default("MISSING_KEY", "default_value"), "default_value");
    }
}use std::fs;
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
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub enable_console: bool,
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

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if self.database.max_connections == 0 {
            return Err(ConfigError::ValidationError("Max connections must be greater than zero".to_string()));
        }

        Ok(())
    }

    pub fn get_env_overrides(&self) -> HashMap<String, String> {
        let mut overrides = HashMap::new();
        
        if let Ok(env_host) = std::env::var("APP_HOST") {
            overrides.insert("server.host".to_string(), env_host);
        }
        
        if let Ok(env_port) = std::env::var("APP_PORT") {
            overrides.insert("server.port".to_string(), env_port);
        }

        overrides
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ConfigError::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: Some("/var/log/app.log".to_string()),
                enable_console: true,
            },
        };

        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        config.to_file(path).unwrap();
        let loaded = AppConfig::from_file(path).unwrap();

        assert_eq!(loaded.server.host, config.server.host);
        assert_eq!(loaded.server.port, config.server.port);
    }

    #[test]
    fn test_config_validation() {
        let invalid_config = AppConfig {
            server: ServerConfig {
                host: "localhost".to_string(),
                port: 0,
                enable_ssl: false,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost/mydb".to_string(),
                max_connections: 10,
                timeout_seconds: 30,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                enable_console: true,
            },
        };

        assert!(invalid_config.validate().is_err());
    }
}