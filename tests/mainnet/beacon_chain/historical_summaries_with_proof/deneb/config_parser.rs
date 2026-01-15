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
}