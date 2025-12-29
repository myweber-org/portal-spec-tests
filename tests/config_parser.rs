use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
    pub enable_metrics: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost/app_db"),
            log_level: String::from("info"),
            cache_size: 1000,
            enable_metrics: true,
        }
    }
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        
        let mut config: AppConfig = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config: {}", e))?;
        
        config.validate()?;
        Ok(config)
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.server_port == 0 {
            return Err(String::from("Server port cannot be 0"));
        }
        
        if self.database_url.is_empty() {
            return Err(String::from("Database URL cannot be empty"));
        }
        
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log_level.as_str()) {
            return Err(format!("Invalid log level: {}", self.log_level));
        }
        
        if self.cache_size > 100_000 {
            return Err(String::from("Cache size cannot exceed 100,000"));
        }
        
        Ok(())
    }
    
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.enable_metrics);
    }
    
    #[test]
    fn test_valid_config_parsing() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost/test_db"
            log_level = "debug"
            cache_size = 500
            enable_metrics = false
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "debug");
        assert!(!config.enable_metrics);
    }
    
    #[test]
    fn test_invalid_log_level() {
        let toml_content = r#"
            server_port = 3000
            database_url = "postgresql://localhost/test_db"
            log_level = "invalid_level"
            cache_size = 500
            enable_metrics = true
        "#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();
        
        let config = AppConfig::from_file(temp_file.path());
        assert!(config.is_err());
        assert!(config.unwrap_err().contains("Invalid log level"));
    }
    
    #[test]
    fn test_serialization() {
        let config = AppConfig::default();
        let toml_str = config.to_toml();
        assert!(toml_str.is_ok());
        let toml_str = toml_str.unwrap();
        assert!(toml_str.contains("server_port = 8080"));
        assert!(toml_str.contains("log_level = \"info\""));
    }
}