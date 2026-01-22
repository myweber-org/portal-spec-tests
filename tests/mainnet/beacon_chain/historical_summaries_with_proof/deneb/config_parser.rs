use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database_name: String,
    pub pool_size: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub enable_ssl: bool,
    pub max_connections: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub log_level: String,
    pub cache_ttl: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            database: DatabaseConfig {
                host: "localhost".to_string(),
                port: 5432,
                username: "postgres".to_string(),
                password: "".to_string(),
                database_name: "app_db".to_string(),
                pool_size: 10,
            },
            server: ServerConfig {
                address: "0.0.0.0".to_string(),
                port: 8080,
                enable_ssl: false,
                max_connections: 100,
            },
            log_level: "info".to_string(),
            cache_ttl: 300,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.database.port == 0 {
            return Err("Database port cannot be zero".to_string());
        }
        
        if self.server.port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        
        if self.database.pool_size == 0 {
            return Err("Database pool size cannot be zero".to_string());
        }
        
        if self.server.max_connections == 0 {
            return Err("Max connections cannot be zero".to_string());
        }
        
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        Ok(())
    }
    
    pub fn to_yaml(&self) -> Result<String, Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(self)?;
        Ok(yaml)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.database.port, 5432);
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.database.port = 0;
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("database:"));
        assert!(yaml.contains("server:"));
    }
    
    #[test]
    fn test_config_from_file() {
        let yaml_content = r#"
database:
  host: "db.example.com"
  port: 5432
  username: "user"
  password: "pass"
  database_name: "test_db"
  pool_size: 5
server:
  address: "127.0.0.1"
  port: 3000
  enable_ssl: true
  max_connections: 50
log_level: "debug"
cache_ttl: 600
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), yaml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.database.host, "db.example.com");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.log_level, "debug");
    }
}use std::collections::HashMap;
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
        write!(
            temp_file,
            "host=localhost\nport=8080\n# This is a comment\n\ntimeout=60"
        )
        .unwrap();

        let result = config.load_from_file(temp_file.path());
        assert!(result.is_ok());
        assert_eq!(config.get("host"), Some(&"localhost".to_string()));
        assert_eq!(config.get("port"), Some(&"8080".to_string()));
        assert_eq!(config.get("timeout"), Some(&"60".to_string()));
        assert_eq!(config.get("retries"), Some(&"3".to_string()));
    }

    #[test]
    fn test_validation() {
        let config = Config::new();
        let required = vec!["timeout", "retries", "missing_key"];
        let result = config.validate_required(&required);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert_eq!(missing, vec!["missing_key".to_string()]);
    }

    #[test]
    fn test_fallback_value() {
        let config = Config::new();
        assert_eq!(config.get_with_fallback("unknown", "default"), "default");
        assert_eq!(config.get_with_fallback("timeout", "99"), "30");
    }
}