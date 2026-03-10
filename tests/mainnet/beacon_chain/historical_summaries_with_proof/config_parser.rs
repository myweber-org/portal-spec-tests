use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server_port: u16,
    pub database_url: String,
    pub log_level: String,
    pub cache_size: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            server_port: 8080,
            database_url: String::from("postgresql://localhost:5432/app_db"),
            log_level: String::from("info"),
            cache_size: 100,
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
        if self.server_port == 0 {
            return Err("Server port cannot be zero".to_string());
        }
        if self.database_url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        if self.cache_size > 10000 {
            return Err("Cache size exceeds maximum limit".to_string());
        }
        Ok(())
    }

    pub fn merge_with_defaults(mut self) -> Self {
        let default = AppConfig::default();
        if self.server_port == 0 {
            self.server_port = default.server_port;
        }
        if self.database_url.is_empty() {
            self.database_url = default.database_url;
        }
        if self.log_level.is_empty() {
            self.log_level = default.log_level;
        }
        if self.cache_size == 0 {
            self.cache_size = default.cache_size;
        }
        self
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
        assert_eq!(config.database_url, "postgresql://localhost:5432/app_db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_size, 100);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        config.server_port = 0;
        assert!(config.validate().is_err());

        config.server_port = 8080;
        config.database_url = String::new();
        assert!(config.validate().is_err());

        config.database_url = String::from("valid_url");
        config.cache_size = 20000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut config = AppConfig {
            server_port: 0,
            database_url: String::new(),
            log_level: String::new(),
            cache_size: 0,
        };
        config = config.merge_with_defaults();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.database_url, "postgresql://localhost:5432/app_db");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.cache_size, 100);
    }

    #[test]
    fn test_from_file() {
        let toml_content = r#"
            server_port = 9000
            database_url = "postgresql://localhost:5432/test_db"
            log_level = "debug"
            cache_size = 500
        "#;

        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), toml_content).unwrap();

        let config = AppConfig::from_file(temp_file.path()).unwrap();
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.database_url, "postgresql://localhost:5432/test_db");
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.cache_size, 500);
    }
}