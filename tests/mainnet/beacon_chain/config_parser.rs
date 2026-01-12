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
                let processed_value = Self::process_value(value.trim());
                values.insert(key.trim().to_string(), processed_value);
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
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
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
                max_connections: 10,
                pool_timeout_seconds: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_path: None,
                max_file_size_mb: 100,
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(&path)
            .map_err(|e| ConfigError::FileRead(path.as_ref().to_path_buf(), e))?;
        
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e))?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e))?;
        
        fs::write(&path, toml_string)
            .map_err(|e| ConfigError::FileWrite(path.as_ref().to_path_buf(), e))
    }
    
    fn validate(&mut self) -> Result<(), ConfigError> {
        if self.server.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be zero".to_string()));
        }
        
        if self.database.max_connections == 0 {
            self.database.max_connections = 5;
        }
        
        if self.logging.level.is_empty() {
            self.logging.level = "info".to_string();
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileRead(std::path::PathBuf, std::io::Error),
    FileWrite(std::path::PathBuf, std::io::Error),
    ParseError(toml::de::Error),
    SerializeError(toml::ser::Error),
    ValidationError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::FileRead(path, err) => 
                write!(f, "Failed to read config file {:?}: {}", path, err),
            ConfigError::FileWrite(path, err) => 
                write!(f, "Failed to write config file {:?}: {}", path, err),
            ConfigError::ParseError(err) => 
                write!(f, "Failed to parse config: {}", err),
            ConfigError::SerializeError(err) => 
                write!(f, "Failed to serialize config: {}", err),
            ConfigError::ValidationError(msg) => 
                write!(f, "Config validation failed: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}